use crate::docker::compose::DockerCompose;
use crate::error::Result;
use colored::*;
use reqwest::Client;
use serde_json::Value;

pub async fn execute() -> Result<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  ZecKit - Devnet Status".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    
    let compose = DockerCompose::new()?;
    let containers = compose.ps()?;
    
    // Display container status
    for container in containers {
        let status_color = if container.contains("Up") {
            "green"
        } else {
            "red"
        };
        
        println!("  {}", container.color(status_color));
    }
    
    println!();
    
    // Check service health
    let client = Client::new();
    
    // Zebra
    print_service_status(&client, "Zebra", "http://127.0.0.1:8232").await;
    
    // Faucet
    print_service_status(&client, "Faucet", "http://127.0.0.1:8080/stats").await;
    
    println!();
    Ok(())
}

async fn print_service_status(client: &Client, name: &str, url: &str) {
    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(json) = resp.json::<Value>().await {
                println!("  {} {} - {}", "✓".green(), name.bold(), format_json(&json));
            } else {
                println!("  {} {} - {}", "✓".green(), name.bold(), "OK");
            }
        }
        _ => {
            println!("  {} {} - {}", "✗".red(), name.bold(), "Not responding");
        }
    }
}

fn format_json(json: &Value) -> String {
    if let Some(height) = json.get("zebra_height") {
        format!("Height: {}", height)
    } else if let Some(balance) = json.get("current_balance") {
        format!("Balance: {} ZEC", balance)
    } else {
        "Running".to_string()
    }
}