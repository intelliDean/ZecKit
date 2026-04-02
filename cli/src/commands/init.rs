use crate::error::{Result, ZecKitError};
use colored::*;
use std::fs;
use std::path::PathBuf;

const WORKFLOW_TEMPLATE: &str = r#"name: ZecKit E2E CI

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]
  workflow_dispatch:

jobs:
  zeckit-e2e:
    name: ZecKit E2E
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: 🚀 Start ZecKit Devnet
        uses: {repo}@{branch}
        with:
          backend: '{backend}'
          startup_timeout_minutes: '15'
"#;

fn detect_github_repo() -> String {
    use std::process::Command;
    
    // Try to get the remote URL from git
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output();
        
    if let Ok(out) = output {
        let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if url.contains("github.com") {
            // Handle both HTTPS and SSH URLs
            // HTTPS: https://github.com/owner/repo.git
            // SSH: git@github.com:owner/repo.git
            let parts: Vec<&str> = if url.contains("https://") {
                url.trim_start_matches("https://github.com/").trim_end_matches(".git").split('/').collect()
            } else {
                url.trim_start_matches("git@github.com:").trim_end_matches(".git").split('/').collect()
            };
            
            if parts.len() >= 2 {
                return format!("{}/{}", parts[0], parts[1]);
            }
        }
    }
    
    // Fallback to original repo if detection fails
    "intelliDean/ZecKit".to_string()
}

fn detect_git_branch() -> String {
    use std::process::Command;
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output();
        
    if let Ok(out) = output {
        let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !branch.is_empty() && branch != "HEAD" {
            return branch;
        }
    }
    
    "main".to_string()
}

pub async fn execute(
    backend: String,
    force: bool,
    output: Option<String>,
    _project_dir: Option<String>,
) -> Result<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("  {}", "ZecKit - Workflow Generator".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());

    // 1. Determine target path
    let target_path = if let Some(out) = output {
        PathBuf::from(out)
    } else {
        // Default to .github/workflows/zeckit-e2e.yml in the current dir
        // Note: We ignore project_dir here because 'init' should target the user's project,
        // while project_dir points to the toolkit resources.
        let base_dir = std::env::current_dir().map_err(|e| ZecKitError::Io(e))?;
        base_dir.join(".github").join("workflows").join("zeckit-e2e.yml")
    };

    // 2. Check if file exists
    if target_path.exists() && !force {
        println!("{} Workflow file already exists at {:?}", "Warning:".yellow().bold(), target_path);
        println!("Use --force to overwrite it.");
        return Ok(());
    }

    // 3. Create parent directories
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| ZecKitError::Io(e))?;
    }

    // 4. Generate content
    let repo = detect_github_repo();
    let branch = detect_git_branch();
    let content = WORKFLOW_TEMPLATE
        .replace("{backend}", &backend)
        .replace("{repo}", &repo)
        .replace("{branch}", &branch);

    // 5. Write file
    fs::write(&target_path, content).map_err(|e| ZecKitError::Io(e))?;

    println!("{} Successfully initialized ZecKit workflow!", "✓".green().bold());
    println!("File created at: {}", target_path.to_string_lossy().cyan());
    println!("\nNext steps:");
    println!("  1. Commit the new workflow file.");
    println!("  2. Push to GitHub to trigger your first ZecKit-powered CI run.");
    println!("\nHappy private coding! 🛡️");

    Ok(())
}
