use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use zcash_protocol::value::Zatoshis;

mod config;
mod wallet;
mod api;
mod validation;
mod error;

use config::Config;
use wallet::WalletManager;

#[derive(Clone)]
pub struct AppState {
    pub wallet: Arc<RwLock<WalletManager>>,
    pub config: Arc<Config>,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

/// Health check for Zaino - uses lightweight gRPC ping instead of full sync
async fn wait_for_zaino(uri: &str, max_attempts: u32) -> anyhow::Result<u64> {
    use zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient;
    use zcash_client_backend::proto::service::ChainSpec;
    
    info!(" Waiting for Zaino at {} to be ready...", uri);
    
    for attempt in 1..=max_attempts {
        let ping_result = tokio::time::timeout(
            Duration::from_secs(5),
            async {
                let channel = Channel::from_shared(uri.to_string())?
                    .connect_timeout(Duration::from_secs(3))
                    .connect()
                    .await?;
                
                let mut client = CompactTxStreamerClient::new(channel);
                let response = client.get_latest_block(ChainSpec {}).await?;
                let block = response.into_inner();
                
                Ok::<u64, anyhow::Error>(block.height)
            }
        ).await;
        
        match ping_result {
            Ok(Ok(height)) => {
                info!(" Zaino ready at block height {} (took {}s)", height, attempt * 5);
                return Ok(height);
            }
            Ok(Err(e)) => {
                if attempt % 6 == 0 {  // Log every 30 seconds
                    info!(" Still waiting for Zaino... ({}s elapsed)", attempt * 5);
                    tracing::debug!("Zaino error: {}", e);
                } else {
                    tracing::debug!("Zaino not ready (attempt {}): {}", attempt, e);
                }
            }
            Err(_) => {
                if attempt % 6 == 0 {
                    info!(" Still waiting for Zaino... ({}s elapsed) - connection timeout", attempt * 5);
                } else {
                    tracing::debug!("Zaino connection timeout (attempt {})", attempt);
                }
            }
        }
        
        if attempt < max_attempts {
            sleep(Duration::from_secs(5)).await;
        }
    }
    
    Err(anyhow::anyhow!("Zaino not ready after {} seconds", max_attempts * 5))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ═══════════════════════════════════════════════════════════
    // STEP 1: Initialize Tracing
    // ═══════════════════════════════════════════════════════════
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zeckit_faucet=debug,zingolib=debug,zingo_sync=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting ZecKit Faucet v0.3.0");

    // ═══════════════════════════════════════════════════════════
    // STEP 2: Load Configuration
    // ═══════════════════════════════════════════════════════════
    let config = Config::load()?;
    info!("📋 Configuration loaded");
    info!("  Network: regtest");
    info!("  Backend: {}", if config.lightwalletd_uri.contains("lightwalletd") { "lightwalletd" } else { "zaino" }); 
    info!("  LightwalletD URI: {}", config.lightwalletd_uri);
    info!("  Data dir: {}", config.zingo_data_dir.display());

    // ═══════════════════════════════════════════════════════════
    // STEP 3: Wait for Zaino Backend
    // ═══════════════════════════════════════════════════════════
    let chain_height = wait_for_zaino(&config.lightwalletd_uri, 60).await?;
    info!("🔗 Connected to Zaino at block {}", chain_height);
    // Extra grace period: give Zaino a moment to fully index the chain before we sync
    info!("⏳ Allowing Zaino indexer to stabilize (10s)...");
    sleep(Duration::from_secs(10)).await;

    // ═══════════════════════════════════════════════════════════
    // STEP 4: Initialize Wallet
    // ═══════════════════════════════════════════════════════════
    info!("💼 Initializing wallet...");
    let wallet = WalletManager::new(
        config.zingo_data_dir.clone(),
        config.lightwalletd_uri.clone(),
    ).await?;

    let wallet = Arc::new(RwLock::new(wallet));

    // Get wallet address
    let address = wallet.read().await.get_unified_address().await?;
    info!(" Wallet initialized");
    info!("  Address: {}", address);

    // ═══════════════════════════════════════════════════════════
    // STEP 5: Initial Sync (Retrying with reinit on connection errors)
    // ═══════════════════════════════════════════════════════════
    info!("🔄 Performing initial wallet sync...");
    
    let mut sync_attempts = 0u32;
    let max_sync_attempts = 8;
    
    loop {
        sync_attempts += 1;
        info!("  [Attempt #{}/{}] Syncing wallet...", sync_attempts, max_sync_attempts);
        
        let sync_result = {
            let mut wallet_guard = wallet.write().await;
            tokio::time::timeout(
                Duration::from_secs(300),
                wallet_guard.sync()
            ).await
        };
        
        match sync_result {
            Ok(Ok(_)) => {
                info!(" ✓ Initial sync completed successfully");
                break;
            }
            Ok(Err(e)) => {
                let err_str = e.to_string();
                let is_connection_err = err_str.contains("HTTP Request Error")
                    || err_str.contains("connection refused")
                    || err_str.contains("transport error")
                    || err_str.contains("sync mode error"); // stuck lock
                
                if is_connection_err && sync_attempts < max_sync_attempts {
                    tracing::warn!("  ⚠ Sync #{} failed (connection/lock error): {} — reinitializing wallet client...", sync_attempts, e);
                    // CRITICAL FIX: Reinitialize WalletManager to clear Zingolib's stuck sync flag
                    sleep(Duration::from_secs(15)).await;
                    match WalletManager::new(config.zingo_data_dir.clone(), config.lightwalletd_uri.clone()).await {
                        Ok(new_wallet) => {
                            let mut w = wallet.write().await;
                            *w = new_wallet;
                            drop(w);
                        }
                        Err(reinit_err) => {
                            tracing::warn!("  Failed to reinitialize wallet: {} (will retry sync anyway)", reinit_err);
                        }
                    }
                } else if sync_attempts >= max_sync_attempts {
                    tracing::error!(" ❌ Sync failed after {} attempts: {} (continuing with 0 balance)", sync_attempts, e);
                    break;
                } else {
                    tracing::error!(" ❌ Initial sync failed (non-connection error): {} (continuing anyway)", e);
                    break;
                }
            }
            Err(_) => {
                tracing::warn!("  ⏱ Sync #{} timed out locally (reinitializing wallet client...)", sync_attempts);
                if sync_attempts < max_sync_attempts {
                    sleep(Duration::from_secs(10)).await;
                    match WalletManager::new(config.zingo_data_dir.clone(), config.lightwalletd_uri.clone()).await {
                        Ok(new_wallet) => {
                            let mut w = wallet.write().await;
                            *w = new_wallet;
                            drop(w);
                        }
                        Err(reinit_err) => {
                            tracing::warn!("  Failed to reinitialize wallet: {} (will retry sync anyway)", reinit_err);
                        }
                    }
                } else {
                    tracing::error!(" ❌ Sync timed out after {} attempts (continuing with 0 balance)", sync_attempts);
                    break;
                }
            }
        }
    }

    // Check balance after sync
    match wallet.read().await.get_balance().await {
        Ok(balance) => {
            info!("💰 Initial balance: {} ZEC", balance.total_zec());
            if balance.transparent > Zatoshis::ZERO {
                info!("  Transparent: {} ZEC", balance.transparent_zec());
            }
            if balance.sapling > Zatoshis::ZERO {
                info!("  Sapling: {} ZEC", balance.sapling_zec());
            }
            if balance.orchard > Zatoshis::ZERO {
                info!("  Orchard: {} ZEC", balance.orchard_zec());
            }
        }
        Err(e) => {
            tracing::warn!("⚠ Could not read balance: {}", e);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // STEP 6: Build Application State
    // ═══════════════════════════════════════════════════════════
    let state = AppState {
        wallet: wallet.clone(),
        config: Arc::new(config.clone()),
        start_time: chrono::Utc::now(),
    };

    // ═══════════════════════════════════════════════════════════
    // STEP 7: Start Background Sync Task 
    // ═══════════════════════════════════════════════════════════
    let sync_wallet = wallet.clone();
    tokio::spawn(async move {
        // Wait before starting to avoid collision with initial sync
        sleep(Duration::from_secs(10)).await;
        
        info!("🔄 Starting background wallet sync (every 120 seconds)");
        
        let mut interval = tokio::time::interval(Duration::from_secs(120));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        let mut sync_count = 0u64;
        
        loop {
            interval.tick().await;
            sync_count += 1;
            
            tracing::debug!("🔄 Background sync attempt #{}", sync_count);
            
            // Try to acquire write lock with reasonable timeout
            let lock_result = tokio::time::timeout(
                Duration::from_secs(5),  // ← Increased from 2s to 5s
                sync_wallet.write()
            ).await;
            
            match lock_result {
                Ok(mut wallet_guard) => {
                    // Perform sync_and_await with generous timeout
                    let sync_result = tokio::time::timeout(
                        Duration::from_secs(90), 
                        wallet_guard.sync()
                    ).await;
                    
                    match sync_result {
                        Ok(Ok(result)) => {
                            // Sync completed successfully
                            tracing::debug!("Sync result: {:?}", result);
                            
                            // Release write lock before reading balance
                            drop(wallet_guard);
                            
                            match sync_wallet.read().await.get_balance().await {
                                Ok(balance) => {
                                    info!("✓ Sync #{} complete - Balance: {} ZEC", sync_count, balance.total_zec());
                                }
                                Err(e) => {
                                    tracing::warn!("✓ Sync #{} complete (balance check failed: {})", sync_count, e);
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            tracing::warn!("⚠ Sync #{} failed: {} (will retry in 60s)", sync_count, e);
                        }
                        Err(_) => {
                            tracing::error!("⏱ Sync #{} timed out after 90s (will retry in 60s)", sync_count);
                        }
                    }
                }
                Err(_) => {
                    tracing::debug!("⏭ Sync #{} skipped - couldn't acquire lock (wallet busy)", sync_count);
                }
            }
        }
    });

    // ═══════════════════════════════════════════════════════════
    // STEP 8: Build and Start Web Server
    // ═══════════════════════════════════════════════════════════
    let app = Router::new()
        .route("/", get(api::root))
        .route("/health", get(api::health::health_check))
        .route("/stats", get(api::stats::get_stats))
        .route("/history", get(api::stats::get_history))
        .route("/request", post(api::faucet::request_funds))
        .route("/address", get(api::wallet::get_addresses))
        .route("/sync", post(api::wallet::sync_wallet))
        .route("/shield", post(api::wallet::shield_funds)) 
        .route("/send", post(api::wallet::send_shielded))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("🌐 Server ready on {}", addr);
    info!("📡 Background sync: Active (120s interval)");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
