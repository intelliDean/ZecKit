use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub zebra_rpc_url: String,
    pub faucet_api_url: String,
    pub backend_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            zebra_rpc_url: "http://127.0.0.1:8232".to_string(),
            faucet_api_url: "http://127.0.0.1:8080".to_string(),
            backend_url: "http://127.0.0.1:9067".to_string(),
        }
    }
}

#[allow(dead_code)]
impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
}