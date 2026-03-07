use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub zingo_data_dir: PathBuf,
    pub lightwalletd_uri: String,
    pub zebra_rpc_url: String,
    pub faucet_amount_min: f64,
    pub faucet_amount_max: f64,
    pub faucet_amount_default: f64,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self {
            zingo_data_dir: std::env::var("ZINGO_DATA_DIR")
                .unwrap_or_else(|_| "/var/zingo".to_string())
                .into(),
            lightwalletd_uri: std::env::var("LIGHTWALLETD_URI")
                .unwrap_or_else(|_| "http://zaino:9067".to_string()),
            zebra_rpc_url: std::env::var("ZEBRA_RPC_URL")
                .unwrap_or_else(|_| "http://zebra:8232".to_string()),
            faucet_amount_min: std::env::var("FAUCET_AMOUNT_MIN")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.01),
            faucet_amount_max: std::env::var("FAUCET_AMOUNT_MAX")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100.0),
            faucet_amount_default: std::env::var("FAUCET_AMOUNT_DEFAULT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10.0),
        })
    }
}