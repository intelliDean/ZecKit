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
          image_prefix: 'ghcr.io/intellidean/zeckit'
          startup_timeout_minutes: '20'

      # Add your own test steps below!
      # Example:
      # - name: Run Custom Smoke Test
      #   run: |
      #     chmod +x ./smoke_test.sh
      #     ./smoke_test.sh
"#;

fn find_toolkit_root() -> Option<PathBuf> {
    let mut curr = std::env::current_dir().ok()?;
    loop {
        if curr.join("action.yml").exists() {
            return Some(curr);
        }
        if !curr.pop() {
            break;
        }
    }
    None
}

fn detect_github_repo() -> String {
    use std::process::Command;
    
    // Try to get the remote URL from git
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output();
        
    if let Ok(out) = output {
        let mut url = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if url.contains("github.com") {
            // Handle both HTTPS and SSH URLs
            // HTTPS: https://github.com/owner/repo.git OR https://token@github.com/owner/repo.git
            // SSH: git@github.com:owner/repo.git
            let parts: Vec<&str> = if url.contains("https://") {
                // If there's a token, it will be in the form https://token@github.com/owner/repo.git
                if let Some(pos) = url.find('@') {
                    url = format!("https://{}", &url[pos+1..]);
                }
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
    project_dir: Option<String>,
) -> Result<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("  {}", "ZecKit - Workflow Generator".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());

    // 1. Determine target path
    let target_base = if let Some(ref dir) = project_dir {
        PathBuf::from(dir)
    } else {
        std::env::current_dir().map_err(|e| ZecKitError::Io(e))?
    };

    let target_path = if let Some(out) = output {
        PathBuf::from(out)
    } else {
        // Default to .github/workflows/zeckit-e2e.yml in the target project dir
        target_base.join(".github").join("workflows").join("zeckit-e2e.yml")
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
    // Detection logic:
    // If we're in the ZecKit toolkit repo (action.yml exists in the root), use detected local repo/branch (Contributor mode)
    // If we're in an external app project (no action.yml), default to intelliDean/ZecKit@main (User mode)
    let is_toolkit_repo = find_toolkit_root().is_some();

    let (repo, branch) = if is_toolkit_repo {
        (detect_github_repo(), detect_git_branch())
    } else {
        ("intelliDean/ZecKit".to_string(), "main".to_string())
    };

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
