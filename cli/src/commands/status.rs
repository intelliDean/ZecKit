use crate::docker::compose::DockerCompose;
use crate::error::Result;
use colored::*;
use reqwest::Client;
use serde_json::Value;

pub async fn execute(project_dir: Option<String>) -> Result<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  ZecKit - Devnet Status".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    
    let compose = DockerCompose::new(project_dir, None)?;
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
    
    // Zebra Miner
    print_zebra_status(&client, "Zebra Miner", "http://127.0.0.1:8232").await;

    // Zebra Sync
    print_zebra_status(&client, "Zebra Sync ", "http://127.0.0.1:18232").await;
    
    // Faucet
    print_faucet_status(&client, "Faucet", "http://127.0.0.1:8080/stats").await;
    
    println!();
    Ok(())
}

async fn print_zebra_status(client: &Client, name: &str, url: &str) {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "status",
        "method": "getblockcount",
        "params": []
    });

    match client.post(url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(json) = resp.json::<Value>().await {
                let height = json["result"].as_u64().unwrap_or(0);
                println!("  {} {} - Height: {}", "✓".green(), name.bold(), height);
            } else {
                println!("  {} {} - {}", "✓".green(), name.bold(), "OK");
            }
        }
        _ => {
            println!("  {} {} - {}", "✗".red(), name.bold(), "Not responding");
        }
    }
}

async fn print_faucet_status(client: &Client, name: &str, url: &str) {
    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(json) = resp.json::<Value>().await {
                let balance = json["current_balance"].as_f64().unwrap_or(0.0);
                println!("  {} {} - Balance: {:.2} ZEC", "✓".green(), name.bold(), balance);
            } else {
                println!("  {} {} - {}", "✓".green(), name.bold(), "OK");
            }
        }
        _ => {
            println!("  {} {} - {}", "✗".red(), name.bold(), "Not responding");
        }
    }
}