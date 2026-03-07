use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FaucetError {
    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for FaucetError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            FaucetError::InvalidAddress(msg) => (StatusCode::BAD_REQUEST, msg),
            FaucetError::InvalidAmount(msg) => (StatusCode::BAD_REQUEST, msg),
            FaucetError::InsufficientBalance(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            FaucetError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            FaucetError::Wallet(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            FaucetError::TransactionFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            FaucetError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}