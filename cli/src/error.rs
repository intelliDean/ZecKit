use thiserror::Error;

pub type Result<T> = std::result::Result<T, ZecKitError>;

#[derive(Error, Debug)]
pub enum ZecKitError {
    #[error("Docker error: {0}")]
    Docker(String),
    
    #[error("Health check failed: {0}")]
    HealthCheck(String),
    
    #[error("Service not ready: {0}")]
    ServiceNotReady(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
