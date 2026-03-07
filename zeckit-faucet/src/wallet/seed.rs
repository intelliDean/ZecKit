use crate::error::FaucetError;
use std::fs;
use std::path::Path;
use tracing::info;

pub struct SeedManager {
    seed_file: std::path::PathBuf,
}

impl SeedManager {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            seed_file: data_dir.join(".wallet_seed"),
        }
    }
    
    /// Get or create deterministic seed for this ZecKit installation
    pub fn get_or_create_seed(&self) -> Result<String, FaucetError> {
        // If seed file exists, use it
        if self.seed_file.exists() {
            info!("Loading existing wallet seed from {:?}", self.seed_file);
            let seed = fs::read_to_string(&self.seed_file)
                .map_err(|e| FaucetError::Wallet(format!("Failed to read seed file: {}", e)))?;
            return Ok(seed.trim().to_string());
        }
        
        // Generate new seed for this installation
        info!("Generating new deterministic seed for this ZecKit installation");
        
        // Use a default regtest seed (same for all installations)
        // This ensures everyone gets the same wallet addresses for testing
        let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        
        info!("⚠️  Using default regtest seed - same wallet for all ZecKit installations");
        info!("   This is intentional for regtest development environments");
        
        // Save seed for future runs
        fs::write(&self.seed_file, seed_phrase)
            .map_err(|e| FaucetError::Wallet(format!("Failed to write seed file: {}", e)))?;
        
        info!("Seed saved to {:?}", self.seed_file);
        
        Ok(seed_phrase.to_string())
    }
}