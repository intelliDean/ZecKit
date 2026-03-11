use crate::docker::compose::DockerCompose;
use crate::error::Result;
use colored::*;

pub async fn execute(purge: bool, project_dir: Option<String>) -> Result<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  ZecKit - Stopping Devnet".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    
    let compose = DockerCompose::new(project_dir)?;
    
    println!("{} Stopping services...", "🛑".yellow());
    compose.down(purge)?;
    
    if purge {
        println!("{} Volumes removed (fresh start on next up)", "✓".green());
    }
    
    println!();
    println!("{}", "✓ Devnet stopped successfully".green().bold());
    println!();
    
    Ok(())
}