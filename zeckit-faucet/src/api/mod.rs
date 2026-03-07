pub mod health;
pub mod faucet;
pub mod stats;
pub mod wallet;

use axum::{Json, extract::State};
use serde_json::json;

use crate::AppState;

pub(crate) async fn root(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "name": "ZecKit Faucet",
        "version": "0.3.0",
        "description": "Zcash Regtest Development Faucet (Rust + ZingoLib)",
        "network": "regtest",
        "wallet_backend": "zingolib",
        "endpoints": {
            "health": "/health",
            "stats": "/stats",
            "request": "/request",
            "address": "/address",
            "sync": "/sync",
            "shield": "/shield",
            "send": "/send",
            "history": "/history"
        }
    }))
}