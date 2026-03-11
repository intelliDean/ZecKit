use crate::docker::compose::DockerCompose;
use crate::docker::health::HealthChecker;
use crate::error::{Result, ZecKitError};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use tokio::time::{sleep, Duration};

const MAX_WAIT_SECONDS: u64 = 60000;

// Known transparent address from default seed "abandon abandon abandon..."
const DEFAULT_FAUCET_ADDRESS: &str = "tmBsTi2xWTjUdEXnuTceL7fecEQKeWaPDJd";

pub async fn execute(backend: String, fresh: bool, timeout: u64, action_mode: bool, project_dir: Option<String>) -> Result<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  ZecKit - Starting Devnet".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    
    let compose = DockerCompose::new(project_dir.clone())?;
    
    if fresh {
        println!("{}", "🧹 Cleaning up old data (fresh start)...".yellow());
        compose.down(true)?;
    }
    
    let (services, profile) = match backend.as_str() {
        "lwd" => (vec!["zebra-miner", "zebra-sync", "lightwalletd", "faucet-lwd"], "lwd"),
        "zaino" => (vec!["zebra-miner", "zebra-sync", "zaino", "faucet-zaino"], "zaino"),
        "none" => (vec!["zebra-miner", "zebra-sync"], "none"),
        _ => {
            return Err(ZecKitError::Config(format!(
                "Invalid backend: {}. Use 'lwd', 'zaino', or 'none'", 
                backend
            )));
        }
    };
    
    println!("Starting services: {}", services.join(", "));
    println!();
    
    // ========================================================================
    // STEP 1: Pre-configure zebra.toml BEFORE starting any containers
    // ========================================================================
    println!("📝 Configuring Zebra mining address...");
    
    match update_zebra_config_file(DEFAULT_FAUCET_ADDRESS, project_dir.clone()) {
        Ok(_) => {
            println!("✓ Updated docker/configs/zebra.toml");
            println!("  Mining to: {}", DEFAULT_FAUCET_ADDRESS);
        }
        Err(e) => {
            println!("{}", format!("Warning: Could not update zebra.toml: {}", e).yellow());
            println!("  Using existing config");
        }
    }
    println!();
    
    // ========================================================================
    // STEP 2: Build and start services (smart build - only when needed)
    // ========================================================================
    if backend == "lwd" || backend == "zaino" {
        compose.up_with_profile(profile, fresh)?;
        println!();
    } else {
        compose.up(&services)?;
    }
    
    println!("Starting services...");
    println!();
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    
    // ========================================================================
    // STEP 3: Wait for Zebra
    // ========================================================================
    let checker = HealthChecker::new();
    let start = std::time::Instant::now();
    
    // Wait for Miner
    println!("Waiting for Zebra Miner node to initialize...");
    let mut last_error_miner = String::new();
    let mut last_error_sync = String::new();
    let mut last_error_print = std::time::Instant::now();

    loop {
        pb.tick();
        match checker.check_zebra_miner_ready().await {
            Ok(_) => {
                println!("\n[1.1/3] Zebra Miner ready");
                break;
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str != last_error_miner || last_error_print.elapsed().as_secs() > 10 {
                    println!("  Miner: {}", err_str);
                    last_error_miner = err_str;
                    last_error_print = std::time::Instant::now();
                }
                
                if start.elapsed().as_secs() > timeout * 60 {
                    let _ = save_faucet_stats_artifact(action_mode, project_dir.clone()).await;
                    return Err(ZecKitError::ServiceNotReady(format!("Zebra Miner not ready after {} minutes: {}", timeout, e)));
                }
            }
        }
        sleep(Duration::from_secs(2)).await;
    }

    // Wait for Sync Node
    println!("Waiting for Zebra Sync node to initialize and peer...");
    let start_sync = std::time::Instant::now();
    let mut last_error_print = std::time::Instant::now();

    loop {
        pb.tick();
        match checker.check_zebra_sync_ready().await {
            Ok(_) => {
                println!("\n[1.2/3] Zebra Sync Node ready");
                break;
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str != last_error_sync || last_error_print.elapsed().as_secs() > 10 {
                    println!("  Sync Node: {}", err_str);
                    last_error_sync = err_str;
                    last_error_print = std::time::Instant::now();
                }

                if start_sync.elapsed().as_secs() > timeout * 60 {
                    let _ = save_faucet_stats_artifact(action_mode, project_dir.clone()).await;
                    return Err(ZecKitError::ServiceNotReady(format!("Zebra Sync Node not ready after {} minutes: {}", timeout, e)));
                }
            }
        }
        sleep(Duration::from_secs(2)).await;
    }
    println!("[1/3] Zebra Cluster ready (100%)");
    println!();
    
    // ========================================================================
    // STEP 4: Wait for Backend (if using lwd or zaino)
    // ========================================================================
    if backend == "lwd" || backend == "zaino" {
        let backend_name = if backend == "lwd" { "Lightwalletd" } else { "Zaino" };
        let start = std::time::Instant::now();
        
        loop {
            pb.tick();
            
            if checker.wait_for_backend(&backend, &pb).await.is_ok() {
                println!("[2/3] {} ready (100%)", backend_name);
                break;
            }
            
            let elapsed = start.elapsed().as_secs();
            if elapsed < 180 {
                let progress = (elapsed as f64 / 180.0 * 100.0).min(99.0) as u32;
                print!("\r[2/3] Starting {}... {}%", backend_name, progress);
                io::stdout().flush().ok();
                sleep(Duration::from_secs(1)).await;
            } else {
                return Err(ZecKitError::ServiceNotReady(format!("{} not ready", backend_name)));
            }
        }
        println!();
    }
    
    // ========================================================================
    // STEP 5: Wait for Faucet
    // ========================================================================
    let start = std::time::Instant::now();
    loop {
        pb.tick();
        
        if checker.wait_for_faucet(&pb).await.is_ok() {
            println!("[3/3] Faucet ready (100%)");
            break;
        }
        
        let elapsed = start.elapsed().as_secs();
        if elapsed < 120 {
            let progress = (elapsed as f64 / 120.0 * 100.0).min(99.0) as u32;
            print!("\r[3/3] Starting Faucet... {}%", progress);
            io::stdout().flush().ok();
            sleep(Duration::from_secs(1)).await;
        } else {
            return Err(ZecKitError::ServiceNotReady("Faucet not ready".into()));
        }
    }
    println!();
    
    pb.finish_and_clear();
    
    // ========================================================================
    // STEP 6: Verify wallet address matches configured address
    // ========================================================================
    println!();
    println!("🔍 Verifying wallet configuration...");
    
    match get_wallet_transparent_address_from_faucet().await {
        Ok(addr) => {
            println!("✓ Faucet wallet address: {}", addr);
            if addr != DEFAULT_FAUCET_ADDRESS {
                println!("{}", format!("⚠ Warning: Address mismatch!").yellow());
                println!("{}", format!("  Expected: {}", DEFAULT_FAUCET_ADDRESS).yellow());
                println!("{}", format!("  Got:      {}", addr).yellow());
                println!("{}", "  This may cause funds to be lost!".yellow());
            } else {
                println!("✓ Address matches Zebra mining configuration");
            }
        }
        Err(e) => {
            println!("{}", format!("Warning: Could not verify wallet address: {}", e).yellow());
        }
    }
    println!();
    
    // ========================================================================
    // STEP 7: Mine initial blocks for maturity
    // ========================================================================
    println!();
    
    let current_blocks = get_block_count(&Client::new()).await.unwrap_or(0);
    let target_blocks = 101;
    
    if current_blocks < target_blocks {
        let needed = (target_blocks - current_blocks) as u32;
        println!("Mining {} initial blocks for full maturity...", needed);
        mine_additional_blocks(needed).await?;
    }
    
    // ========================================================================
    // STEP 8: Ensure blocks are fully synced
    // ========================================================================
    wait_for_mined_blocks(&pb, target_blocks).await?;
    
    // ========================================================================
    // STEP 9: Wait for blocks to propagate
    // ========================================================================
    println!();
    println!("Waiting for blocks to propagate...");
    sleep(Duration::from_secs(10)).await;
    
    // ========================================================================
    // STEP 10: Generate UA fixtures from faucet API
    // ========================================================================
    println!();
    println!("Generating ZIP-316 Unified Address fixtures...");
    
    match generate_ua_fixtures_from_faucet().await {
        Ok(address) => {
            println!("Generated UA: {}...", &address[..20]);
        }
        Err(e) => {
            println!("{}", format!("Warning: Could not generate UA fixture ({})", e).yellow());
        }
    }
    
    // ========================================================================
    // STEP 11: Sync wallet through faucet API
    // ========================================================================
    println!();
    println!("Syncing wallet with blockchain...");
    
    // Give wallet time to catch up with mined blocks
    sleep(Duration::from_secs(5)).await;
    
    if let Err(e) = sync_wallet_via_faucet().await {
        println!("{}", format!("Wallet sync warning: {}", e).yellow());
        println!("  Will retry after waiting...");
        sleep(Duration::from_secs(10)).await;
        
        // Retry once
        if let Err(e) = sync_wallet_via_faucet().await {
            println!("{}", format!("Wallet sync still failing: {}", e).yellow());
        } else {
            println!("✓ Wallet synced on retry");
        }
    } else {
        println!("✓ Wallet synced with blockchain");
    }
    
    // Wait for sync to complete
    sleep(Duration::from_secs(5)).await;
    
    // ========================================================================
    // STEP 12: Check balance BEFORE shielding
    // ========================================================================
    println!();
    println!("Checking transparent balance...");
    match check_wallet_balance().await {
        Ok((transparent, orchard, total)) => {
            println!("  Transparent: {} ZEC", transparent);
            println!("  Orchard: {} ZEC", orchard);
            println!("  Total: {} ZEC", total);
            
            if transparent == 0.0 && total == 0.0 {
                println!();
                println!("{}", "⚠ WARNING: Wallet has no funds!".yellow().bold());
                println!("{}", "  This means Zebra did NOT mine to the faucet wallet address.".yellow());
                println!("{}", "  Possible causes:".yellow());
                println!("{}", "    1. Zebra config wasn't updated properly".yellow());
                println!("{}", "    2. Wallet seed mismatch".yellow());
                println!("{}", "  The devnet will still work, but the faucet won't have funds.".yellow());
            }
        }
        Err(e) => {
            println!("{}", format!("Could not check balance: {}", e).yellow());
        }
    }
    
    // ========================================================================
    // STEP 13: Shield transparent funds to orchard
    // ========================================================================
    println!();
    if let Err(e) = shield_transparent_funds().await {
        println!("{}", format!("Shield operation: {}", e).yellow());
    } else {
        // Sync again after shielding
        println!("Re-syncing after shielding...");
        sleep(Duration::from_secs(15)).await;
        
        if let Err(e) = sync_wallet_via_faucet().await {
            println!("{}", format!("Warning: Post-shield sync failed: {}", e).yellow());
        } else {
            println!("✓ Post-shield sync complete");
        }
        
        sleep(Duration::from_secs(5)).await;
    }
    
    // ========================================================================
    // STEP 14: Final balance check
    // ========================================================================
    println!();
    println!("Final wallet balance:");
    match check_wallet_balance().await {
        Ok((transparent, orchard, total)) => {
            println!("  Transparent: {} ZEC", transparent);
            println!("  Orchard: {} ZEC", orchard);
            println!("  Total: {} ZEC", total);
            
            if total > 0.0 {
                println!();
                println!("{}", "✓ Faucet wallet funded and ready!".green().bold());
            }
        }
        Err(e) => {
            println!("{}", format!("Could not check balance: {}", e).yellow());
        }
    }
    
    // ========================================================================
    // STEP 15: Start background miner
    // ========================================================================
    println!();
    println!("Starting continuous background miner (1 block every 15s)...");
    start_background_miner().await?;
    
    print_connection_info(&backend);
    print_mining_info().await?;
    
    println!();
    println!("{}", "✓ Devnet is running with continuous mining".green().bold());
    println!("{}", "   New blocks will be mined every 15 seconds".green());
    println!("{}", "   Press Ctrl+C to stop".green());
    
    // Save artifacts if in action mode
    if action_mode {
        let _ = save_faucet_stats_artifact(action_mode, project_dir.clone()).await;
    }
    
    Ok(())
}

async fn save_faucet_stats_artifact(action_mode: bool, project_dir_override: Option<String>) -> Result<()> {
    if !action_mode {
        return Ok(());
    }

    let project_dir = if let Some(dir) = project_dir_override {
        std::path::PathBuf::from(dir)
    } else {
        let current_dir = std::env::current_dir()?;
        if current_dir.ends_with("cli") {
            current_dir.parent().unwrap().to_path_buf()
        } else {
            current_dir
        }
    };

    let log_dir = project_dir.join("logs");
    fs::create_dir_all(&log_dir).ok();
    
    match Client::new().get("http://127.0.0.1:8080/stats").send().await {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let stats_path = log_dir.join("faucet-stats.json");
                fs::write(
                    &stats_path,
                    serde_json::to_string_pretty(&json)?
                ).ok();
                println!("✓ Saved {:?}", stats_path);
            }
        }
        Err(e) => println!("  Warning: Could not get faucet stats for artifact: {}", e),
    }

    Ok(())
}


// ============================================================================
// NEW FUNCTION: Update zebra.toml on host before starting containers
// ============================================================================
fn update_zebra_config_file(address: &str, project_dir_override: Option<String>) -> Result<()> {
    use regex::Regex;
    
    // Get project root
    let project_dir = if let Some(dir) = project_dir_override {
        std::path::PathBuf::from(dir)
    } else {
        let current_dir = std::env::current_dir()?;
        if current_dir.ends_with("cli") {
            current_dir.parent().unwrap().to_path_buf()
        } else {
            current_dir
        }
    };
    
    let config_path = project_dir.join("docker/configs/zebra.toml");
    
    // Read current config
    let config = fs::read_to_string(&config_path)
        .map_err(|e| ZecKitError::Config(format!("Could not read {:?}: {}", config_path, e)))?;
    
    // Update miner address using regex
    let updated = if config.contains("miner_address") {
        // Replace existing miner_address
        let re = Regex::new(r#"miner_address\s*=\s*"[^"]*""#)
            .map_err(|e| ZecKitError::Config(format!("Regex error: {}", e)))?;
        re.replace(&config, format!("miner_address = \"{}\"", address)).to_string()
    } else {
        // Add miner_address to [mining] section
        if config.contains("[mining]") {
            config.replace(
                "[mining]",
                &format!("[mining]\nminer_address = \"{}\"", address)
            )
        } else {
            // Add entire [mining] section at the end
            format!("{}\n\n[mining]\nminer_address = \"{}\"\n", config, address)
        }
    };
    
    // Write back to file
    fs::write(&config_path, updated)
        .map_err(|e| ZecKitError::Config(format!("Could not write {:?}: {}", config_path, e)))?;
    
    Ok(())
}

// ============================================================================
// Helper Functions (keep all your existing functions below)
// ============================================================================

async fn wait_for_mined_blocks(_pb: &ProgressBar, min_blocks: u64) -> Result<()> {
    let client = Client::new();
    let start = std::time::Instant::now();
    
    println!("Mining initial blocks...");
    
    loop {
        match get_block_count(&client).await {
            Ok(height) if height >= min_blocks => {
                println!("✓ Mined {} blocks", height);
                println!();
                return Ok(());
            }
            Ok(height) => {
                let progress = (height as f64 / min_blocks as f64 * 100.0) as u64;
                print!("\r  Block {} / {} ({}%)", height, min_blocks, progress);
                io::stdout().flush().ok();
            }
            Err(_) => {}
        }
        
        if start.elapsed().as_secs() > MAX_WAIT_SECONDS {
            return Err(ZecKitError::ServiceNotReady(
                "Internal miner timeout - blocks not reaching maturity".into()
            ));
        }
        
        sleep(Duration::from_secs(2)).await;
    }
}

async fn mine_additional_blocks(count: u32) -> Result<()> {
    let client = Client::new();
    
    println!("Mining {} additional blocks...", count);
    
    let mut successful_mines = 0;
    while successful_mines < count {
        let res = client
            .post("http://127.0.0.1:8232")
            .json(&json!({
                "jsonrpc": "2.0",
                "id": "generate",
                "method": "generate",
                "params": [1]
            }))
            .timeout(Duration::from_secs(10))
            .send()
            .await;
            
        match res {
            Ok(resp) if resp.status().is_success() => {
                successful_mines += 1;
                if successful_mines % 10 == 0 || successful_mines == count {
                    print!("\r  Mined {} / {} blocks", successful_mines, count);
                    io::stdout().flush().ok();
                }
            }
            Ok(resp) => {
                // Not success status
                sleep(Duration::from_millis(500)).await;
            }
            Err(_) => {
                // Connection or timeout error
                sleep(Duration::from_millis(500)).await;
            }
        }
    }
    
    println!("\n✓ Mined {} additional blocks", count);
    Ok(())
}

async fn start_background_miner() -> Result<()> {
    tokio::spawn(async {
        let client = Client::new();
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        
        loop {
            interval.tick().await;
            
            let _ = client
                .post("http://127.0.0.1:8232")
                .json(&json!({
                    "jsonrpc": "2.0",
                    "id": "bgminer",
                    "method": "generate",
                    "params": [1]
                }))
                .timeout(Duration::from_secs(10))
                .send()
                .await;
        }
    });
    
    Ok(())
}

async fn shield_transparent_funds() -> Result<()> {
    let client = Client::new();
    
    println!("Shielding transparent funds to Orchard...");
    
    let resp = client
        .post("http://127.0.0.1:8080/shield")
        .timeout(Duration::from_secs(60))
        .send()
        .await?;
    
    let json: serde_json::Value = resp.json().await?;
    
    if json["status"] == "no_funds" {
        return Err(ZecKitError::HealthCheck("No transparent funds to shield".into()));
    }
    
    if let Some(txid) = json.get("txid").and_then(|v| v.as_str()) {
        println!("✓ Shielded {} ZEC", json["transparent_amount"].as_f64().unwrap_or(0.0));
        println!("  Transaction ID: {}", txid);
        println!("  Waiting for confirmation...");
        sleep(Duration::from_secs(20)).await;
        return Ok(());
    }
    
    Err(ZecKitError::HealthCheck("Shield transaction failed".into()))
}

async fn get_block_count(client: &Client) -> Result<u64> {
    // Check miner first
    let resp = client
        .post("http://127.0.0.1:8232")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "blockcount",
            "method": "getblockcount",
            "params": []
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    let json: serde_json::Value = resp.json().await?;
    
    let miner_height = json.get("result")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ZecKitError::HealthCheck("Invalid miner block count".into()))?;

    // Check sync node parity
    if let Ok(resp_sync) = client
        .post("http://127.0.0.1:18232")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "blockcount",
            "method": "getblockcount",
            "params": []
        }))
        .timeout(Duration::from_secs(2))
        .send()
        .await {
            if let Ok(json_sync) = resp_sync.json::<serde_json::Value>().await {
                if let Some(sync_height) = json_sync.get("result").and_then(|v| v.as_u64()) {
                    if sync_height < miner_height {
                        // Just log for now, don't fail yet as sync takes time
                    }
                }
            }
        }

    Ok(miner_height)
}

async fn get_wallet_transparent_address_from_faucet() -> Result<String> {
    let client = Client::new();
    
    let resp = client
        .get("http://127.0.0.1:8080/address")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ZecKitError::HealthCheck(format!("Faucet API call failed: {}", e)))?;
    
    let json: serde_json::Value = resp.json().await?;
    
    json.get("transparent_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ZecKitError::HealthCheck("No transparent address in faucet response".into()))
        .map(|s| s.to_string())
}

async fn generate_ua_fixtures_from_faucet() -> Result<String> {
    let client = Client::new();
    
    let resp = client
        .get("http://127.0.0.1:8080/address")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ZecKitError::HealthCheck(format!("Faucet API call failed: {}", e)))?;
    
    let json: serde_json::Value = resp.json().await?;
    
    let ua_address = json.get("unified_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ZecKitError::HealthCheck("No unified address in faucet response".into()))?;
    
    let fixture = json!({
        "faucet_address": ua_address,
        "type": "unified",
        "receivers": ["orchard"]
    });
    
    fs::create_dir_all("fixtures")?;
    fs::write(
        "fixtures/unified-addresses.json",
        serde_json::to_string_pretty(&fixture)?
    )?;
    
    Ok(ua_address.to_string())
}

async fn sync_wallet_via_faucet() -> Result<()> {
    let client = Client::new();
    
    let resp = client
        .post("http://127.0.0.1:8080/sync")
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| ZecKitError::HealthCheck(format!("Faucet sync failed: {}", e)))?;
    
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(ZecKitError::HealthCheck(
            format!("Wallet sync failed ({}): {}", status, body)
        ));
    }
    
    Ok(())
}

async fn check_wallet_balance() -> Result<(f64, f64, f64)> {
    let client = Client::new();
    let resp = client
        .get("http://127.0.0.1:8080/stats")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    let json: serde_json::Value = resp.json().await?;
    
    let transparent = json["transparent_balance"].as_f64().unwrap_or(0.0);
    let orchard = json["orchard_balance"].as_f64().unwrap_or(0.0);
    let total = json["current_balance"].as_f64().unwrap_or(0.0);
    
    Ok((transparent, orchard, total))
}

async fn print_mining_info() -> Result<()> {
    let client = Client::new();
    
    if let Ok(height) = get_block_count(&client).await {
        println!();
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!("{}", "  Blockchain Status".cyan().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!();
        println!("  Block Height: {}", height);
        println!("  Network: Regtest");
        println!("  Mining: Continuous (1 block / 15s)");
    }
    
    Ok(())
}

fn print_connection_info(backend: &str) {
    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  Services Ready".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    println!("  Zebra RPC: http://127.0.0.1:8232");
    println!("  Faucet API: http://127.0.0.1:8080");
    
    if backend == "lwd" {
        println!("  LightwalletD: http://127.0.0.1:9067");
    } else if backend == "zaino" {
        println!("  Zaino: http://127.0.0.1:9067");
    }
    
    println!();
    println!("Next steps:");
    println!("  • Check balance: curl http://127.0.0.1:8080/stats");
    println!("  • View fixtures: cat fixtures/unified-addresses.json");
    println!("  • Request funds: curl -X POST http://127.0.0.1:8080/request -d '{{\"address\":\"...\"}}'");
    println!();
}