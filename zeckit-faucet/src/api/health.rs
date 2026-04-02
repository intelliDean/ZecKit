use axum::{Json, extract::State};
use serde_json::json;
use chrono;

use crate::AppState;
use crate::error::FaucetError;

pub(crate) async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet_guard = state.wallet.read().await;
    let synced_height = wallet_guard.get_height().await?;
    let balance = wallet_guard.get_balance().await?;
    let balance_zec = balance.total_zec();

    Ok(Json(json!({
        "status": "healthy",
        "wallet_backend": "zingolib",
        "network": "regtest",
        "balance": balance_zec,
        "synced_height": synced_height,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.3.0"
    })))
}