use crate::error::{Result, ZecKitError};
use reqwest::Client;
use indicatif::ProgressBar;
use tokio::time::{sleep, Duration};
use serde_json::Value;
use std::net::TcpStream;
use std::time::Duration as StdDuration;

pub struct HealthChecker {
    client: Client,
    max_retries: u32,
    retry_delay: Duration,
    backend_max_retries: u32,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            max_retries: 1800, // 1 hour (1800 * 2s)
            retry_delay: Duration::from_secs(2),
            backend_max_retries: 1800, 
        }
    }

    pub async fn check_zebra_miner_ready(&self) -> Result<()> {
        self.check_zebra(8232).await
    }

    pub async fn check_zebra_sync_ready(&self) -> Result<()> {
        self.check_zebra(18232).await
    }

    pub async fn wait_for_faucet(&self, pb: &ProgressBar) -> Result<()> {
        for i in 0..self.max_retries {
            pb.tick();
            
            match self.check_faucet().await {
                Ok(_) => return Ok(()),
                Err(_) if i < self.max_retries - 1 => {
                    sleep(self.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(ZecKitError::ServiceNotReady("Faucet".into()))
    }

    pub async fn wait_for_backend(&self, backend: &str, pb: &ProgressBar) -> Result<()> {
        for i in 0..self.backend_max_retries {
            pb.tick();
            
            // First check TCP connectivity
            if self.check_backend_port(9067).is_ok() {
                // Then check sync parity via faucet (if lwd)
                if backend == "lwd" {
                    match self.check_backend_sync_parity().await {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            pb.set_message(format!("Waiting for {} sync: {}", backend, e));
                        }
                    }
                } else {
                    return Ok(());
                }
            }

            if i < self.backend_max_retries - 1 {
                sleep(self.retry_delay).await;
            }
        }

        Err(ZecKitError::ServiceNotReady(format!("{} not ready or synchronized", backend)))
    }

    async fn check_backend_sync_parity(&self) -> Result<()> {
        // 1. Get Zebra Miner height
        let zebra_height = self.get_zebra_height(8232).await?;
        
        // 2. Get Faucet/LWD synced height
        let faucet_height = self.get_faucet_height().await?;
        
        if faucet_height < zebra_height.saturating_sub(1) {
            return Err(ZecKitError::HealthCheck(format!(
                "Backend lagging: Miner={} LWD={}", 
                zebra_height, faucet_height
            )));
        }
        
        Ok(())
    }

    async fn get_zebra_height(&self, port: u16) -> Result<u64> {
        let url = format!("http://127.0.0.1:{}", port);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": "health",
                "method": "getblockcount",
                "params": []
            }))
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .map_err(|e| ZecKitError::HealthCheck(format!("RPC call to {} failed: {}", url, e)))?;

        let json: Value = resp.json().await
            .map_err(|e| ZecKitError::HealthCheck(format!("Failed to parse Zebra response: {}", e)))?;
        
        json["result"].as_u64()
            .ok_or_else(|| ZecKitError::HealthCheck("Invalid height in Zebra response".into()))
    }

    async fn get_faucet_height(&self) -> Result<u64> {
        let resp = self
            .client
            .get("http://127.0.0.1:8080/health")
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .map_err(|_| ZecKitError::HealthCheck("Faucet not responding".into()))?;

        let json: Value = resp.json().await
            .map_err(|_| ZecKitError::HealthCheck("Failed to parse Faucet health JSON".into()))?;
        
        json["synced_height"].as_u64()
            .ok_or_else(|| ZecKitError::HealthCheck("synced_height missing in Faucet response".into()))
    }

    fn check_backend_port(&self, port: u16) -> Result<()> {
        match TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", port).parse().unwrap(),
            StdDuration::from_secs(1)
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err(ZecKitError::HealthCheck(format!("Port {} not open", port)))
        }
    }

    async fn check_zebra(&self, port: u16) -> Result<()> {
        match self.get_zebra_height(port).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }

    async fn check_faucet(&self) -> Result<()> {
        let resp = self
            .client
            .get("http://127.0.0.1:8080/health")
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ZecKitError::HealthCheck("Faucet not ready".into()));
        }

        let json: Value = resp.json().await?;
        
        if json.get("status").and_then(|s| s.as_str()) == Some("unhealthy") {
            return Err(ZecKitError::HealthCheck("Faucet unhealthy".into()));
        }

        Ok(())
    }
}