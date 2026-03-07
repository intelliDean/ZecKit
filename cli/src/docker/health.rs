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
            max_retries: 900,
            retry_delay: Duration::from_secs(2),
            backend_max_retries: 900,
        }
    }

    pub async fn wait_for_zebra_miner(&self, pb: &ProgressBar) -> Result<()> {
        for i in 0..self.max_retries {
            pb.tick();
            
            match self.check_zebra(8232).await {
                Ok(_) => return Ok(()),
                Err(_) if i < self.max_retries - 1 => {
                    sleep(self.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(ZecKitError::ServiceNotReady("Zebra Miner".into()))
    }

    pub async fn wait_for_zebra_sync(&self, pb: &ProgressBar) -> Result<()> {
        for i in 0..self.max_retries {
            pb.tick();
            
            match self.check_zebra(18232).await {
                Ok(_) => return Ok(()),
                Err(_) if i < self.max_retries - 1 => {
                    sleep(self.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(ZecKitError::ServiceNotReady("Zebra Sync".into()))
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
            
            match self.check_backend(backend).await {
                Ok(_) => return Ok(()),
                Err(_) if i < self.backend_max_retries - 1 => {
                    sleep(self.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(ZecKitError::ServiceNotReady(format!("{} not ready", backend)))
    }

    async fn check_zebra(&self, port: u16) -> Result<()> {
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
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ZecKitError::HealthCheck(format!("Zebra on port {} not ready", port)))
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
    
    async fn check_backend(&self, backend: &str) -> Result<()> {
        // Zaino and Lightwalletd are gRPC services on port 9067
        // They don't respond to HTTP, so we do a TCP connection check
        
        let backend_name = if backend == "lwd" { "lightwalletd" } else { "zaino" };
        
        // Try to connect to localhost:9067 with 2 second timeout
        match TcpStream::connect_timeout(
            &"127.0.0.1:9067".parse().unwrap(),
            StdDuration::from_secs(2)
        ) {
            Ok(_) => {
                // For Zaino, give it extra time after port opens to initialize
                if backend == "zaino" {
                    sleep(Duration::from_secs(10)).await;
                }
                Ok(())
            }
            Err(_) => {
                // Port not accepting connections yet
                Err(ZecKitError::HealthCheck(format!("{} not ready", backend_name)))
            }
        }
    }
}