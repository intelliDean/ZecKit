use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::json;
use zcash_protocol::value::Zatoshis;
use crate::{AppState, error::FaucetError};

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub wallet_id: String,
}

/// GET /address - Returns wallet addresses (default faucet wallet)
pub(crate) async fn get_addresses(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;
    
    let unified_address = wallet.get_unified_address(None).await?;
    let transparent_address = wallet.get_transparent_address(None).await?;
    
    Ok(Json(json!({
        "unified_address": unified_address,
        "transparent_address": transparent_address
    })))
}

/// POST /sync - Syncs wallet with blockchain (default faucet wallet)
pub(crate) async fn sync_wallet(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    match wallet.sync(None).await {
        Ok(_) => {
            Ok(Json(json!({
                "status": "synced",
                "message": "Wallet synced with blockchain"
            })))
        },
        Err(e) if e.to_string().contains("sync is already running") => {
            // Log for visibility but return success to avoid blocking the caller
            tracing::info!("Wallet sync requested but already in progress");
            Ok(Json(json!({
                "status": "syncing",
                "message": "Wallet sync is already in progress"
            })))
        },
        Err(e) => Err(e),
    }
}

/// POST /shield - Shields transparent funds to Orchard (default faucet wallet)
pub(crate) async fn shield_funds(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    let balance = wallet.get_balance(None).await?;
    
    if balance.transparent == Zatoshis::ZERO {
        return Ok(Json(json!({
            "status": "no_funds",
            "message": "No transparent funds to shield"
        })));
    }
    
    // Calculate the amount that will actually be shielded (minus fee)
    let fee = 10_000u64; // 0.0001 ZEC
    let fee_zatoshis = Zatoshis::from_u64(fee)
        .expect("Hardcoded fee should always be a valid Zatoshis amount");
    let shield_amount = if balance.transparent > fee_zatoshis {
        (balance.transparent - fee_zatoshis)
            .expect("Checked with > fee, so subtraction cannot underflow")
    } else {
        return Err(FaucetError::Wallet(
            "Insufficient funds to cover transaction fee".to_string()
        ));
    };
    
    let txid = wallet.shield_to_orchard(None).await?;
    
    Ok(Json(json!({
        "status": "shielded",
        "transparent_amount": balance.transparent_zec(),
        "shielded_amount": shield_amount.into_u64() as f64 / 100_000_000.0,
        "fee": fee as f64 / 100_000_000.0,
        "txid": txid,
        "message": format!("Shielded {} ZEC from transparent to orchard (fee: {} ZEC)", 
                          shield_amount.into_u64() as f64 / 100_000_000.0,
                          fee as f64 / 100_000_000.0)
    })))
}

#[derive(Debug, Deserialize)]
pub struct SendRequest {
    pub address: String,
    pub amount: f64,
    pub memo: Option<String>,
}

/// POST /send - Send shielded funds to another address (default faucet wallet)
pub(crate) async fn send_shielded(
    State(state): State<AppState>,
    Json(payload): Json<SendRequest>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    let balance = wallet.get_balance(None).await?;
    
    // Check if we have enough in Orchard pool
    let amount_zatoshis = Zatoshis::from_u64((payload.amount * 100_000_000.0) as u64)
        .map_err(|_| FaucetError::Wallet("Invalid send amount".to_string()))?;
    if balance.orchard < amount_zatoshis {
        return Err(FaucetError::InsufficientBalance(format!(
            "Need {} ZEC in Orchard, have {} ZEC",
            payload.amount,
            balance.orchard_zec()
        )));
    }
    
    // Send the transaction (from Orchard pool)
    let txid = wallet.send_transaction(
        None,
        &payload.address,
        payload.amount,
        payload.memo.clone(),
    ).await?;
    
    let new_balance = wallet.get_balance(None).await?;
    
    Ok(Json(json!({
        "status": "sent",
        "txid": txid,
        "to_address": payload.address,
        "amount": payload.amount,
        "memo": payload.memo.unwrap_or_default(),
        "new_balance": new_balance.total_zec(),
        "orchard_balance": new_balance.orchard_zec(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "message": format!("Sent {} ZEC from Orchard pool", payload.amount)
    })))
}

/// POST /wallets - Spawns a new wallet with `wallet_id` or returns successfully if already exists
pub(crate) async fn create_wallet(
    State(state): State<AppState>,
    Json(payload): Json<CreateWalletRequest>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    let id = payload.wallet_id.trim();
    if id.is_empty() {
        return Err(FaucetError::Wallet("Wallet ID cannot be empty".to_string()));
    }
    if id == "default" {
        return Err(FaucetError::Wallet("Cannot use reserved ID 'default'".to_string()));
    }
    
    let existed = wallet.extra_wallets.contains_key(id);
    if !existed {
        wallet.spawn_wallet(id).await?;
    }
    
    Ok(Json(json!({
        "wallet_id": id,
        "status": if existed { "exists" } else { "created" }
    })))
}

/// GET /wallets - Lists all currently loaded wallets
pub(crate) async fn list_wallets(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;
    let ids = wallet.get_wallet_ids();
    Ok(Json(json!({
        "wallets": ids
    })))
}

/// GET /wallets/:id/address - Retrieves UA & transparent address for specific wallet
pub(crate) async fn get_wallet_address(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;
    let unified_address = wallet.get_unified_address(Some(&id)).await?;
    let transparent_address = wallet.get_transparent_address(Some(&id)).await?;
    
    Ok(Json(json!({
        "wallet_id": id,
        "unified_address": unified_address,
        "transparent_address": transparent_address
    })))
}

/// GET /wallets/:id/stats - Retrieves balance & stats for specific wallet
pub(crate) async fn get_wallet_stats(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;
    let balance = wallet.get_balance(Some(&id)).await?;
    let (tx_count, total_sent) = wallet.get_stats(Some(&id))?;
    
    Ok(Json(json!({
        "wallet_id": id,
        "current_balance": balance.total_zec(),
        "orchard_balance": balance.orchard_zec(),
        "transparent_balance": balance.transparent_zec(),
        "total_requests": tx_count,
        "total_sent": total_sent
    })))
}

/// POST /wallets/:id/sync - Syncs a specific wallet
pub(crate) async fn sync_wallet_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    match wallet.sync(Some(&id)).await {
        Ok(_) => {
            Ok(Json(json!({
                "wallet_id": id,
                "status": "synced",
                "message": format!("Wallet {} synced with blockchain", id)
            })))
        },
        Err(e) if e.to_string().contains("sync is already running") => {
            tracing::info!("Wallet {} sync requested but already in progress", id);
            Ok(Json(json!({
                "wallet_id": id,
                "status": "syncing",
                "message": format!("Wallet {} sync is already in progress", id)
            })))
        },
        Err(e) => Err(e),
    }
}

/// POST /wallets/:id/shield - Shields transparent funds to Orchard for a specific wallet
pub(crate) async fn shield_wallet_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    let balance = wallet.get_balance(Some(&id)).await?;
    
    if balance.transparent == Zatoshis::ZERO {
        return Ok(Json(json!({
            "wallet_id": id,
            "status": "no_funds",
            "message": "No transparent funds to shield"
        })));
    }
    
    let fee = 10_000u64; // 0.0001 ZEC
    let fee_zatoshis = Zatoshis::from_u64(fee)
        .expect("Hardcoded fee should always be a valid Zatoshis amount");
    let shield_amount = if balance.transparent > fee_zatoshis {
        (balance.transparent - fee_zatoshis)
            .expect("Checked with > fee, so subtraction cannot underflow")
    } else {
        return Err(FaucetError::Wallet(
            "Insufficient funds to cover transaction fee".to_string()
        ));
    };
    
    let txid = wallet.shield_to_orchard(Some(&id)).await?;
    
    Ok(Json(json!({
        "wallet_id": id,
        "status": "shielded",
        "transparent_amount": balance.transparent_zec(),
        "shielded_amount": shield_amount.into_u64() as f64 / 100_000_000.0,
        "fee": fee as f64 / 100_000_000.0,
        "txid": txid,
        "message": format!("Shielded {} ZEC from transparent to orchard (fee: {} ZEC)", 
                          shield_amount.into_u64() as f64 / 100_000_000.0,
                          fee as f64 / 100_000_000.0)
    })))
}

/// POST /wallets/:id/send - Performs a shielded send from a specific wallet
pub(crate) async fn send_wallet_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<SendRequest>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    let balance = wallet.get_balance(Some(&id)).await?;
    
    let amount_zatoshis = Zatoshis::from_u64((payload.amount * 100_000_000.0) as u64)
        .map_err(|_| FaucetError::Wallet("Invalid send amount".to_string()))?;
    if balance.orchard < amount_zatoshis {
        return Err(FaucetError::InsufficientBalance(format!(
            "Need {} ZEC in Orchard, have {} ZEC",
            payload.amount,
            balance.orchard_zec()
        )));
    }
    
    let txid = wallet.send_transaction(
        Some(&id),
        &payload.address,
        payload.amount,
        payload.memo.clone(),
    ).await?;
    
    let new_balance = wallet.get_balance(Some(&id)).await?;
    
    Ok(Json(json!({
        "wallet_id": id,
        "status": "sent",
        "txid": txid,
        "to_address": payload.address,
        "amount": payload.amount,
        "memo": payload.memo.unwrap_or_default(),
        "new_balance": new_balance.total_zec(),
        "orchard_balance": new_balance.orchard_zec(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "message": format!("Sent {} ZEC from Orchard pool", payload.amount)
    })))
}