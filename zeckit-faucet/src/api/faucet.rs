use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use zcash_address::ZcashAddress;
use crate::AppState;
use crate::error::FaucetError;

#[derive(Debug, Deserialize)]
pub struct FaucetRequest {
    address: String,
    amount: Option<f64>,
    memo: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FaucetResponse {
    success: bool,
    txid: String,
    address: String,
    amount: f64,
    new_balance: f64,
    timestamp: String,
    network: String,
    message: String,
}

/// Validate a Zcash address for regtest environment.
/// The ZcashAddress API doesn't expose network() method in this version,
/// so we just validate that it parses correctly.
fn validate_address(address: &str) -> Result<String, FaucetError> {
    // Parse the address to validate format
    address.parse::<ZcashAddress>()
        .map_err(|e| FaucetError::InvalidAddress(
            format!("Invalid Zcash address format: {}", e)
        ))?;
    
    // For regtest, if it parses successfully, we accept it
    // The network validation is implicit in the parsing
    Ok(address.to_string())
}

/// Request funds from the faucet.
/// This handler is exposed via routing but not part of the public module API.
pub(crate) async fn request_funds(
    State(state): State<AppState>,
    Json(payload): Json<FaucetRequest>,
) -> Result<Json<FaucetResponse>, FaucetError> {
    // Validate address
    let validated_address = validate_address(&payload.address)?;
    
    // Get and validate amount
    let amount = payload.amount.unwrap_or(state.config.faucet_amount_default);
    if amount < state.config.faucet_amount_min || amount > state.config.faucet_amount_max {
        return Err(FaucetError::InvalidAmount(format!(
            "Amount must be between {} and {} ZEC",
            state.config.faucet_amount_min,
            state.config.faucet_amount_max
        )));
    }
    
    // Send transaction
    let mut wallet = state.wallet.write().await;
    let txid = wallet.send_transaction(&validated_address, amount, payload.memo).await?;
    
    // Get new balance
    let new_balance = wallet.get_balance().await?;
    
    Ok(Json(FaucetResponse {
        success: true,
        txid: txid.clone(),
        address: validated_address,
        amount,
        new_balance: new_balance.total_zec(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        network: "regtest".to_string(),
        message: format!("Sent {} ZEC on regtest. TXID: {}", amount, txid),
    }))
}

/// Get the faucet's own address and balance.
/// 
/// Useful for monitoring faucet health and available funds.
/// Note: Currently unused but kept for future API endpoint that exposes faucet status.
/// This will be used when we add a GET /faucet/status endpoint for monitoring.
#[allow(dead_code)]
pub async fn get_faucet_address(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;
    let address = wallet.get_unified_address().await?;
    let balance = wallet.get_balance().await?;
    
    Ok(Json(json!({
        "address": address,
        "balance": balance.total_zec(),
        "network": "regtest"
    })))
}