use axum::{Json, extract::{State, Query}};
use serde::Deserialize;
use serde_json::json;

use crate::AppState;
use crate::error::FaucetError;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    limit: Option<usize>,
}

pub(crate) async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;
    
    let address = wallet.get_unified_address().await?;
    let balance = wallet.get_balance().await?;
    let (tx_count, total_sent) = wallet.get_stats();
    
    let uptime = chrono::Utc::now() - state.start_time;
    let uptime_seconds = uptime.num_seconds();

    let recent_txs = wallet.get_transaction_history(5);
    let last_request = recent_txs.first().map(|tx| tx.timestamp.to_rfc3339());

    Ok(Json(json!({
        "faucet_address": address,
        "current_balance": balance.total_zec(),
        "orchard_balance": balance.orchard_zec(),
        "transparent_balance": balance.transparent_zec(),
        "total_requests": tx_count,
        "total_sent": total_sent,
        "last_request": last_request,
        "uptime_seconds": uptime_seconds,
        "network": "regtest",
        "wallet_backend": "zingolib",
        "version": "0.3.0"
    })))
}

pub(crate) async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;
    
    let limit = params.limit.unwrap_or(100).min(1000).max(1);
    let history = wallet.get_transaction_history(limit);

    Ok(Json(json!({
        "count": history.len(),
        "limit": limit,
        "transactions": history
    })))
}