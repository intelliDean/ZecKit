use crate::error::{Result, ZecKitError};
use std::process::{Command, Stdio};
use std::fs;
use crate::assets::{ConfigAssets, ComposeAsset};

#[derive(Clone)]
pub struct DockerCompose {
    project_dir: String,
}

impl DockerCompose {
    pub fn new(project_dir_override: Option<String>) -> Result<Self> {
        let project_dir = if let Some(dir) = project_dir_override {
            std::path::PathBuf::from(dir)
        } else {
            dirs::home_dir()
                .ok_or_else(|| ZecKitError::Config("Could not find home directory".into()))?
                .join(".zeckit")
        };

        // Create the base directory
        fs::create_dir_all(&project_dir)?;
        
        // Extract Compose file
        if let Some(compose_file) = ComposeAsset::get("docker-compose.yml") {
            let mut content = String::from_utf8_lossy(&compose_file.data).to_string();
            
            // CRITICAL FIX: Only strip build blocks if we are NOT in build-allowed mode (e.g. CI)
            let allow_build = std::env::var("ZECKIT_ALLOW_BUILD").map(|v| v == "true" || v == "1").unwrap_or(false);

            if !allow_build {
                // Strip out build blocks so docker-compose doesn't look for local directories
                let build_blocks = [
                    "    build:\n      context: ./docker/zebra\n      dockerfile: Dockerfile\n",
                    "    build:\n      context: ./docker/lightwalletd\n      dockerfile: Dockerfile\n",
                    "    build:\n      context: ./docker/zaino\n      dockerfile: Dockerfile\n      args:\n        - NO_TLS=true\n        - RUST_VERSION=1.91.1\n",
                    "    build:\n      context: ./docker/zingo\n      dockerfile: Dockerfile\n",
                    "    build:\n      context: ./zeckit-faucet\n      dockerfile: Dockerfile\n",
                ];
                
                for block in build_blocks.iter() {
                    content = content.replace(block, "");
                }
            } else {
                info!("ZECKIT_ALLOW_BUILD is set, keeping build blocks in docker-compose.yml");
            }
            
            fs::write(project_dir.join("docker-compose.yml"), content)?;
        }

        // Extract configs
        let configs_dir = project_dir.join("docker").join("configs");
        fs::create_dir_all(&configs_dir)?;
        
        for file in ConfigAssets::iter() {
            if let Some(embedded_file) = ConfigAssets::get(&file) {
                let target = configs_dir.join(file.as_ref());
                fs::write(&target, embedded_file.data.as_ref())?;
            }
        }

        Ok(Self {
            project_dir: project_dir.to_string_lossy().to_string(),
        })
    }

    pub fn up(&self, services: &[&str]) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose")
            .arg("up")
            .arg("-d")
            .current_dir(&self.project_dir);

        for service in services {
            cmd.arg(service);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecKitError::Docker(error.to_string()));
        }

        Ok(())
    }

    /// Check if Docker images exist for a profile
    pub fn images_exist(&self, profile: &str) -> bool {
        // Get list of images that would be used by this profile
        let output = Command::new("docker")
            .arg("compose")
            .arg("--profile")
            .arg(profile)
            .arg("config")
            .arg("--images")
            .current_dir(&self.project_dir)
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                let images = String::from_utf8_lossy(&out.stdout);
                
                // Check each image exists locally
                for image in images.lines() {
                    let check = Command::new("docker")
                        .arg("image")
                        .arg("inspect")
                        .arg(image.trim())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                    
                    if check.map(|s| !s.success()).unwrap_or(true) {
                        return false; // At least one image missing
                    }
                }
                
                true // All images exist
            }
            _ => false
        }
    }

    /// Start services with profile, building only if needed
    pub fn up_with_profile(&self, profile: &str, _force_build: bool) -> Result<()> {
        let needs_pull = !self.images_exist(profile);
        
        if needs_pull {
            println!("Pulling Docker images for profile '{}'...", profile);
            println!("(This may take a few minutes)");
            println!();
            
            // Pull with LIVE output instead of silent
            let pull_status = Command::new("docker")
                .arg("compose")
                .arg("--profile")
                .arg(profile)
                .arg("pull")
                .current_dir(&self.project_dir)
                .status()  // This shows output in real-time!
                .map_err(|e| ZecKitError::Docker(format!("Failed to start pull: {}", e)))?;

            if !pull_status.success() {
                return Err(ZecKitError::Docker("Image pull failed".into()));
            }

            println!("✓ Images pulled successfully");
            println!();
        }

        // Start services with live output
        println!("Starting containers...");
        Command::new("docker")
            .arg("compose")
            .arg("--profile")
            .arg(profile)
            .arg("up")
            .arg("-d")
            .current_dir(&self.project_dir)
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| ZecKitError::Docker("Failed to start containers".into()))?;

        Ok(())
    }

    pub fn down(&self, volumes: bool) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose")
            .arg("--profile")
            .arg("zaino")
            .arg("--profile")
            .arg("lwd")
            .arg("down")
            .current_dir(&self.project_dir);

        if volumes {
            cmd.arg("-v");
            cmd.arg("--remove-orphans");
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecKitError::Docker(error.to_string()));
        }

        Ok(())
    }

    pub fn ps(&self) -> Result<Vec<String>> {
        let output = Command::new("docker")
            .arg("compose")
            .arg("ps")
            .arg("--format")
            .arg("table")
            .current_dir(&self.project_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecKitError::Docker(error.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<String> = stdout
            .lines()
            .skip(1) // Skip header
            .map(|l| l.to_string())
            .collect();

        Ok(lines)
    }

    #[allow(dead_code)]
    pub fn logs(&self, service: &str, tail: usize) -> Result<Vec<String>> {
        let output = Command::new("docker")
            .arg("compose")
            .arg("logs")
            .arg("--tail")
            .arg(tail.to_string())
            .arg(service)
            .current_dir(&self.project_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecKitError::Docker(error.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();

        Ok(lines)
    }

    #[allow(dead_code)]
    pub fn exec(&self, service: &str, command: &[&str]) -> Result<String> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose")
            .arg("exec")
            .arg("-T") // Non-interactive
            .arg(service)
            .current_dir(&self.project_dir);

        for arg in command {
            cmd.arg(arg);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecKitError::Docker(error.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        Command::new("docker")
            .arg("compose")
            .arg("ps")
            .arg("-q")
            .current_dir(&self.project_dir)
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false)
    }
}