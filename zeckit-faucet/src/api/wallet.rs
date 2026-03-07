use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::json;
use zcash_protocol::value::Zatoshis;
use crate::{AppState, error::FaucetError};

/// GET /address - Returns wallet addresses
pub(crate) async fn get_addresses(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;
    
    let unified_address = wallet.get_unified_address().await?;
    let transparent_address = wallet.get_transparent_address().await?;
    
    Ok(Json(json!({
        "unified_address": unified_address,
        "transparent_address": transparent_address
    })))
}

/// POST /sync - Syncs wallet with blockchain
pub(crate) async fn sync_wallet(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    wallet.sync().await?;
    
    Ok(Json(json!({
        "status": "synced",
        "message": "Wallet synced with blockchain"
    })))
}

/// POST /shield - Shields transparent funds to Orchard
pub(crate) async fn shield_funds(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    let balance = wallet.get_balance().await?;
    
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
    
    let txid = wallet.shield_to_orchard().await?;
    
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

/// POST /send - Send shielded funds to another address
/// This performs a shielded send from Orchard pool to recipient's address
pub(crate) async fn send_shielded(
    State(state): State<AppState>,
    Json(payload): Json<SendRequest>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    let balance = wallet.get_balance().await?;
    
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
        &payload.address,
        payload.amount,
        payload.memo.clone(),
    ).await?;
    
    let new_balance = wallet.get_balance().await?;
    
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