use crate::error::{Result, ZecKitError};
use crate::SnapshotAction;
use crate::docker::compose::DockerCompose;
use std::fs;
use std::path::{Path, PathBuf};
use colored::*;

pub async fn execute(action: SnapshotAction, project_dir_override: Option<String>) -> Result<()> {
    // Resolve project directory
    let project_dir = if let Some(dir) = project_dir_override.clone() {
        PathBuf::from(dir)
    } else {
        dirs::home_dir()
            .ok_or_else(|| ZecKitError::Config("Could not find home directory".into()))?
            .join(".zeckit")
    };
    
    let snapshots_dir = project_dir.join("snapshots");
    fs::create_dir_all(&snapshots_dir)?;
    
    let project_name = get_project_name(&project_dir);
    let volumes = vec![
        format!("{}_zebra-miner-data", project_name),
        format!("{}_zebra-sync-data", project_name),
        format!("{}_lightwalletd-data", project_name),
        format!("{}_zaino-data", project_name),
        format!("{}_zingo-data", project_name),
        format!("{}_faucet-data", project_name),
    ];

    match action {
        SnapshotAction::Create { name } => {
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
            println!("  ZecKit - Creating Snapshot: {}", name.bold());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
            println!();

            // 1. Stop containers
            let compose = DockerCompose::new(project_dir_override, None)?;
            println!("Stopping running containers...");
            compose.down(false)?;

            // 2. Create snapshot directory
            let snap_dir = snapshots_dir.join(&name);
            if snap_dir.exists() {
                fs::remove_dir_all(&snap_dir)?;
            }
            fs::create_dir_all(&snap_dir)?;

            // 3. Backup each volume
            for vol in &volumes {
                backup_volume(vol, &snap_dir)?;
            }

            println!();
            println!("{}", format!("✓ Snapshot '{}' created successfully!", name).green().bold());
            println!("  Path: {:?}", snap_dir);
        }
        SnapshotAction::Restore { name } => {
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
            println!("  ZecKit - Restoring Snapshot: {}", name.bold());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
            println!();

            let snap_dir = snapshots_dir.join(&name);
            if !snap_dir.exists() {
                return Err(ZecKitError::Config(format!("Snapshot '{}' does not exist", name)));
            }

            // 1. Stop containers
            let compose = DockerCompose::new(project_dir_override, None)?;
            println!("Stopping running containers...");
            compose.down(false)?;

            // 2. Restore each volume
            for vol in &volumes {
                restore_volume(vol, &snap_dir)?;
            }

            println!();
            println!("{}", format!("✓ Snapshot '{}' restored successfully!", name).green().bold());
        }
        SnapshotAction::List => {
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
            println!("  ZecKit - Available Snapshots");
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
            println!();

            let mut count = 0;
            if let Ok(entries) = fs::read_dir(&snapshots_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            println!("  • {}", name.bold());
                            count += 1;
                        }
                    }
                }
            }
            
            if count == 0 {
                println!("  No snapshots found.");
            }
        }
        SnapshotAction::Delete { name } => {
            let snap_dir = snapshots_dir.join(&name);
            if !snap_dir.exists() {
                return Err(ZecKitError::Config(format!("Snapshot '{}' does not exist", name)));
            }
            
            fs::remove_dir_all(&snap_dir)?;
            println!("{}", format!("✓ Snapshot '{}' deleted.", name).green());
        }
    }

    Ok(())
}

fn get_project_name(project_dir: &Path) -> String {
    let folder_name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "zeckit".to_string());
    
    let cleaned: String = folder_name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase();
    
    if cleaned.is_empty() {
        "zeckit".to_string()
    } else {
        cleaned
    }
}

fn backup_volume(volume: &str, backup_dir: &Path) -> Result<()> {
    println!("  Backing up volume {}...", volume);
    
    // Check if the volume actually exists first
    let exists = std::process::Command::new("docker")
        .arg("volume")
        .arg("inspect")
        .arg(volume)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    
    if !exists.success() {
        println!("  Volume {} does not exist, skipping.", volume);
        return Ok(());
    }

    let backup_dir_str = backup_dir.to_string_lossy().to_string();
    
    // Run temporary alpine container to copy data out
    let output = std::process::Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/volume", volume))
        .arg("-v")
        .arg(format!("{}:/backup", backup_dir_str))
        .arg("alpine")
        .arg("tar")
        .arg("-cf")
        .arg(format!("/backup/{}.tar", volume))
        .arg("-C")
        .arg("/volume")
        .arg(".")
        .output()?;
        
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(ZecKitError::Docker(format!("Failed to backup volume {}: {}", volume, err)));
    }
    
    Ok(())
}

fn restore_volume(volume: &str, backup_dir: &Path) -> Result<()> {
    println!("  Restoring volume {}...", volume);
    
    let tar_file = backup_dir.join(format!("{}.tar", volume));
    if !tar_file.exists() {
        println!("  Tarball for {} does not exist, skipping.", volume);
        return Ok(());
    }

    // Ensure volume exists or create it
    std::process::Command::new("docker")
        .arg("volume")
        .arg("create")
        .arg(volume)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    let backup_dir_str = backup_dir.to_string_lossy().to_string();
    
    // Run temporary alpine container to copy data back in
    let output = std::process::Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/volume", volume))
        .arg("-v")
        .arg(format!("{}:/backup", backup_dir_str))
        .arg("alpine")
        .arg("tar")
        .arg("-xf")
        .arg(format!("/backup/{}.tar", volume))
        .arg("-C")
        .arg("/volume")
        .output()?;
        
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(ZecKitError::Docker(format!("Failed to restore volume {}: {}", volume, err)));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_project_name() {
        assert_eq!(get_project_name(Path::new("/home/user/.zeckit")), "zeckit");
        assert_eq!(get_project_name(Path::new("/mnt/data/ZecKit-1.0")), "zeckit10");
        assert_eq!(get_project_name(Path::new("")), "zeckit");
    }
}
