use crate::error::{Result, ZecKitError};
use std::process::{Command, Stdio};
use std::fs;
use crate::assets::{ConfigAssets, ComposeAsset};

#[derive(Clone)]
pub struct DockerCompose {
    project_dir: String,
    image_prefix: Option<String>,
}

impl DockerCompose {
    pub fn new(project_dir_override: Option<String>, image_prefix: Option<String>) -> Result<Self> {
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
                println!("ZECKIT_ALLOW_BUILD is set, keeping build blocks in docker-compose.yml");
                
                if let Ok(src_path) = std::env::var("ZECKIT_SRC_PATH") {
                   println!("ZECKIT_SRC_PATH is set to {}, remapping build contexts", src_path);
                   content = content.replace("./docker/", &format!("{}/docker/", src_path));
                   content = content.replace("./zeckit-faucet", &format!("{}/zeckit-faucet", src_path));
                }
            }
            
            fs::write(project_dir.join("docker-compose.yml"), content)?;
        }

        // Extract configs
        let configs_dir = project_dir.join("docker").join("configs");
        fs::create_dir_all(&configs_dir)?;
        
        for file in ConfigAssets::iter() {
            if let Some(embedded_file) = ConfigAssets::get(file.as_ref()) {
                let target = configs_dir.join(file.as_ref());
                fs::write(&target, embedded_file.data.as_ref())?;
            }
        }

        Ok(Self {
            project_dir: project_dir.to_string_lossy().to_string(),
            image_prefix,
        })
    }

    fn create_command(&self) -> Command {
        let mut cmd = Command::new("docker");
        cmd.arg("compose");
        cmd.current_dir(&self.project_dir);
        
        if let Some(ref prefix) = self.image_prefix {
            cmd.env("IMAGE_PREFIX", prefix);
        }
        
        cmd
    }

    pub fn up(&self, services: &[&str]) -> Result<()> {
        let allow_build = std::env::var("ZECKIT_ALLOW_BUILD").map(|v| v == "true" || v == "1").unwrap_or(false);
        let mut cmd = self.create_command();
        cmd.arg("up")
            .arg("-d");
        
        if allow_build {
            cmd.arg("--build");
        }
        
        cmd.current_dir(&self.project_dir);

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
        let output = self.create_command()
            .arg("--profile")
            .arg(profile)
            .arg("config")
            .arg("--images")
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
        let allow_build = std::env::var("ZECKIT_ALLOW_BUILD").map(|v| v == "true" || v == "1").unwrap_or(false);
        let needs_pull = !self.images_exist(profile);
        
        if needs_pull && !allow_build {
            println!("Pulling Docker images for profile '{}'...", profile);
            println!("(This may take a few minutes)");
            println!();
            
            // Pull with LIVE output instead of silent
            let pull_status = self.create_command()
                .arg("--profile")
                .arg(profile)
                .arg("pull")
                .status()  // This shows output in real-time!
                .map_err(|e| ZecKitError::Docker(format!("Failed to start pull: {}", e)))?;

            if !pull_status.success() {
                return Err(ZecKitError::Docker("Image pull failed".into()));
            }

            println!("✓ Images pulled successfully");
            println!();
        }

        // Start services with live output
        let allow_build = std::env::var("ZECKIT_ALLOW_BUILD").map(|v| v == "true" || v == "1").unwrap_or(false);
        let mut cmd = self.create_command();
        cmd.arg("--profile")
            .arg(profile)
            .arg("up")
            .arg("-d");
        
        if allow_build {
            cmd.arg("--build");
        }
        
        cmd.status()?
            .success()
            .then_some(())
            .ok_or_else(|| ZecKitError::Docker("Failed to start containers".into()))?;

        Ok(())
    }

    pub fn down(&self, volumes: bool) -> Result<()> {
        let mut cmd = self.create_command();
        cmd.arg("--profile")
            .arg("zaino")
            .arg("--profile")
            .arg("lwd")
            .arg("down");

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
        let output = self.create_command()
            .arg("ps")
            .arg("--format")
            .arg("table")
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
        let output = self.create_command()
            .arg("logs")
            .arg("--tail")
            .arg(tail.to_string())
            .arg(service)
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
        let mut cmd = self.create_command();
        cmd.arg("exec")
            .arg("-T") // Non-interactive
            .arg(service);

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
        self.create_command()
            .arg("ps")
            .arg("-q")
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false)
    }
}
