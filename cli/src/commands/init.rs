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
        uses: intelliDean/ZecKit@m3-implementation
        with:
          backend: '{backend}'
          startup_timeout_minutes: '15'
"#;

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
    let content = WORKFLOW_TEMPLATE.replace("{backend}", &backend);

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
