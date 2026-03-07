use axum::{Json, extract::State};
use serde_json::json;

use crate::AppState;
use crate::error::FaucetError;

pub(crate) async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;
    let balance = wallet.get_balance().await?;

    Ok(Json(json!({
        "status": "healthy",
        "wallet_backend": "zingolib",
        "network": "regtest",
        "balance": balance.total_zec(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.3.0"
    })))
}