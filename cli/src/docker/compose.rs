use crate::error::{Result, ZecKitError};
use std::process::{Command, Stdio};

#[derive(Clone)]
pub struct DockerCompose {
    project_dir: String,
}

impl DockerCompose {
    pub fn new() -> Result<Self> {
        // Get project root (go up from cli/ directory)
        let current_dir = std::env::current_dir()?;
        let project_dir = if current_dir.ends_with("cli") {
            current_dir.parent().unwrap().to_path_buf()
        } else {
            current_dir
        };

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
    pub fn up_with_profile(&self, profile: &str, force_build: bool) -> Result<()> {
        let needs_build = force_build || !self.images_exist(profile);
        
        if needs_build {
            println!("Building Docker images for profile '{}'...", profile);
            println!("(This may take 10-20 minutes on first build)");
            println!();
            
            // Build with LIVE output instead of silent
            let build_status = Command::new("docker")
                .arg("compose")
                .arg("--profile")
                .arg(profile)
                .arg("build")
                .current_dir(&self.project_dir)
                .status()  // This shows output in real-time!
                .map_err(|e| ZecKitError::Docker(format!("Failed to start build: {}", e)))?;

            if !build_status.success() {
                return Err(ZecKitError::Docker("Image build failed".into()));
            }

            println!("✓ Images built successfully");
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
            .arg("down")
            .current_dir(&self.project_dir);

        if volumes {
            cmd.arg("-v");
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