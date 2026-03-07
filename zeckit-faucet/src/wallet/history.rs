use crate::error::FaucetError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub timestamp: DateTime<Utc>,
    pub to_address: String,
    pub amount: f64,
    pub txid: String,
    pub memo: String,
}

pub struct TransactionHistory {
    file_path: PathBuf,
    transactions: Vec<TransactionRecord>,
}

impl TransactionHistory {
    pub fn load(data_dir: &Path) -> Result<Self, FaucetError> {
        let file_path = data_dir.join("faucet-history.json");
        
        let transactions = if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| FaucetError::Internal(format!("Failed to read history: {}", e)))?;
            
            serde_json::from_str(&content)
                .map_err(|e| FaucetError::Internal(format!("Failed to parse history: {}", e)))?
        } else {
            Vec::new()
        };

        Ok(Self {
            file_path,
            transactions,
        })
    }

    pub fn add_transaction(&mut self, record: TransactionRecord) -> Result<(), FaucetError> {
        self.transactions.push(record);
        self.save()?;
        Ok(())
    }

    fn save(&self) -> Result<(), FaucetError> {
        let json = serde_json::to_string_pretty(&self.transactions)
            .map_err(|e| FaucetError::Internal(format!("Failed to serialize history: {}", e)))?;
        
        fs::write(&self.file_path, json)
            .map_err(|e| FaucetError::Internal(format!("Failed to write history: {}", e)))?;
        
        Ok(())
    }

    pub fn get_all(&self) -> &[TransactionRecord] {
        &self.transactions
    }

    pub fn get_recent(&self, limit: usize) -> Vec<TransactionRecord> {
        self.transactions
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}
