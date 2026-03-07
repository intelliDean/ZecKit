use crate::error::FaucetError;
use crate::wallet::history::{TransactionHistory, TransactionRecord};
use std::path::PathBuf;
use tracing::info;
use zingolib::{
    lightclient::LightClient,
    config::{ZingoConfig, ChainType},
    wallet::{LightWallet, WalletBase},
};
use axum::http::Uri;
use zcash_primitives::consensus::BlockHeight;
use zebra_chain::parameters::testnet::ConfiguredActivationHeights;
use zcash_primitives::memo::MemoBytes;
use zcash_client_backend::zip321::{TransactionRequest, Payment};
use crate::wallet::seed::SeedManager;
use zcash_protocol::value::Zatoshis;

#[derive(Debug, Clone)]
pub struct Balance {
    pub transparent: Zatoshis,
    pub sapling: Zatoshis,
    pub orchard: Zatoshis,
}

impl Balance {
    pub fn total_zatoshis(&self) -> Zatoshis {
        (self.transparent + self.sapling + self.orchard)
            .expect("Balance overflow - this should never happen")
    }

    pub fn total_zec(&self) -> f64 {
        self.total_zatoshis().into_u64() as f64 / 100_000_000.0
    }

    pub fn orchard_zec(&self) -> f64 {
        self.orchard.into_u64() as f64 / 100_000_000.0
    }

    pub fn transparent_zec(&self) -> f64 {
        self.transparent.into_u64() as f64 / 100_000_000.0
    }

    pub fn sapling_zec(&self) -> f64 {
        self.sapling.into_u64() as f64 / 100_000_000.0
    }
}

pub struct WalletManager {
    client: LightClient,
    history: TransactionHistory,
}

impl WalletManager {
    pub async fn new(
        data_dir: PathBuf,
        server_uri: String,
    ) -> Result<Self, FaucetError> {
        info!("Initializing ZingoLib LightClient");
        
        let uri: Uri = server_uri.parse().map_err(|e| {
            FaucetError::Wallet(format!("Invalid server URI: {}", e))
        })?;

        std::fs::create_dir_all(&data_dir).map_err(|e| {
            FaucetError::Wallet(format!("Failed to create wallet directory: {}", e))
        })?;

        // ============================================================
        // NEW: Get or create deterministic seed
        // ============================================================
        let seed_manager = SeedManager::new(&data_dir);
        let seed_phrase = seed_manager.get_or_create_seed()?;
        
        let activation_heights = ConfiguredActivationHeights {
            before_overwinter: Some(1),
            overwinter: Some(1),
            sapling: Some(1),
            blossom: Some(1),
            heartwood: Some(1),
            canopy: Some(1),
            nu5: Some(1),
            nu6: None,      // ← Changed to None
            nu6_1: None,    // ← Changed to None
            nu7: None,      // ← Changed to None
        };
        let chain_type = ChainType::Regtest(activation_heights);
        
        let config = ZingoConfig::build(chain_type)
            .set_lightwalletd_uri(uri)
            .set_wallet_dir(data_dir.clone())
            .create();

        let wallet_path = data_dir.join("zingo-wallet.dat");
        
        // ============================================================
        // Load existing wallet or create new one with deterministic seed
        // ============================================================
        let client = if wallet_path.exists() {
            info!("Loading existing wallet from {:?}", wallet_path);
            LightClient::create_from_wallet_path(config).map_err(|e| {
                FaucetError::Wallet(format!("Failed to load wallet: {}", e))
            })?
        } else {
            info!("Creating new wallet with deterministic seed");
            
            // Convert seed phrase string to Mnemonic
            let mnemonic = bip0039::Mnemonic::from_phrase(seed_phrase)
                .map_err(|e| FaucetError::Wallet(format!("Invalid mnemonic phrase: {}", e)))?;
            
            // Create wallet from mnemonic
            let wallet = LightWallet::new(
                chain_type,
                WalletBase::Mnemonic {
                    mnemonic,
                    no_of_accounts: std::num::NonZeroU32::new(1).unwrap(),
                },
                BlockHeight::from_u32(0),
                config.wallet_settings.clone(),
            ).map_err(|e| {
                FaucetError::Wallet(format!("Failed to create wallet: {}", e))
            })?;
            
            // Create LightClient from the wallet
            LightClient::create_from_wallet(wallet, config, false).map_err(|e| {
                FaucetError::Wallet(format!("Failed to create client from wallet: {}", e))
            })?
        };

        let history = TransactionHistory::load(&data_dir)?;

        info!("Wallet initialized successfully (sync not started)");

        Ok(Self { client, history })
    }

    pub async fn get_unified_address(&self) -> Result<String, FaucetError> {
        let addresses_json = self.client.unified_addresses_json().await;
        
        let first_address = addresses_json[0]["encoded_address"]
            .as_str()
            .ok_or_else(|| FaucetError::Wallet("No unified address found".to_string()))?;
        
        Ok(first_address.to_string())
    }

    pub async fn get_transparent_address(&self) -> Result<String, FaucetError> {
        let addresses_json = self.client.transparent_addresses_json().await;
        
        let first_address = addresses_json[0]["encoded_address"]
            .as_str()
            .ok_or_else(|| FaucetError::Wallet("No transparent address found".to_string()))?;
        
        Ok(first_address.to_string())
    }

    pub async fn get_balance(&self) -> Result<Balance, FaucetError> {
        let account_balance = self.client
            .account_balance(zip32::AccountId::ZERO)
            .await
            .map_err(|e| FaucetError::Wallet(format!("Failed to get balance: {}", e)))?;
        
        Ok(Balance {
            transparent: account_balance.confirmed_transparent_balance
                .unwrap_or(Zatoshis::ZERO),
            sapling: account_balance.confirmed_sapling_balance
                .unwrap_or(Zatoshis::ZERO),
            orchard: account_balance.confirmed_orchard_balance
                .unwrap_or(Zatoshis::ZERO),
        })
    }

    pub async fn shield_to_orchard(&mut self) -> Result<String, FaucetError> {
        info!("Shielding transparent funds to Orchard...");
        
        let balance = self.get_balance().await?;
        
        if balance.transparent == Zatoshis::ZERO {
            return Err(FaucetError::Wallet("No transparent funds to shield".to_string()));
        }
        
        info!("Shielding {} ZEC from transparent to orchard", balance.transparent_zec());
        
        // Step 1: Propose the shield transaction
        let _proposal = self.client
            .propose_shield(zip32::AccountId::ZERO)
            .await
            .map_err(|e| FaucetError::Wallet(format!("Shield proposal failed: {}", e)))?;
        
        // Step 2: Send the stored proposal
        let txids = self.client
            .send_stored_proposal(true)
            .await
            .map_err(|e| FaucetError::Wallet(format!("Shield send failed: {}", e)))?;
        
        let txid = txids.first().to_string();
        
        info!("Shielded transparent funds in txid: {}", txid);
        Ok(txid)
    }

    pub async fn send_transaction(
        &mut self,
        to_address: &str,
        amount_zec: f64,
        memo: Option<String>,
    ) -> Result<String, FaucetError> {
        info!("Sending {} ZEC to {}", amount_zec, &to_address[..to_address.len().min(16)]);

        let amount_zatoshis = (amount_zec * 100_000_000.0) as u64;

        let balance = self.get_balance().await?;
        if balance.orchard < Zatoshis::from_u64(amount_zatoshis).unwrap() {
            return Err(FaucetError::InsufficientBalance(format!(
                "Need {} ZEC, have {} ZEC in Orchard pool",
                amount_zec,
                balance.orchard_zec()
            )));
        }

        // Parse recipient address
        let recipient_address = to_address.parse()
            .map_err(|e| FaucetError::Wallet(format!("Invalid address: {}", e)))?;

        // Create amount
        let amount = zcash_protocol::value::Zatoshis::from_u64(amount_zatoshis)
            .map_err(|_| FaucetError::Wallet("Invalid amount".to_string()))?;

        // Create memo bytes if provided
        let memo_bytes = if let Some(memo_text) = &memo {
            // Convert string to bytes (max 512 bytes for Zcash memo)
            let bytes = memo_text.as_bytes();
            if bytes.len() > 512 {
                return Err(FaucetError::Wallet("Memo too long (max 512 bytes)".to_string()));
            }
            
            // Pad to 512 bytes
            let mut padded = [0u8; 512];
            padded[..bytes.len()].copy_from_slice(bytes);
            
            Some(MemoBytes::from_bytes(&padded)
                .map_err(|e| FaucetError::Wallet(format!("Invalid memo: {}", e)))?)
        } else {
            None
        };

        // Create Payment with all 6 required arguments
        let payment = Payment::new(
            recipient_address,
            amount,
            memo_bytes,
            None,  // label
            None,  // message
            vec![], // other_params
        ).ok_or_else(|| FaucetError::Wallet("Failed to create payment".to_string()))?;

        // Create TransactionRequest
        let request = TransactionRequest::new(vec![payment])
            .map_err(|e| FaucetError::Wallet(format!("Failed to create request: {}", e)))?;

        // Send using quick_send
        let txids = self.client
            .quick_send(request, zip32::AccountId::ZERO, false)
            .await
            .map_err(|e| {
                FaucetError::TransactionFailed(format!("Failed to send transaction: {}", e))
            })?;

        let txid = txids.first().to_string();

        // Record in history
        self.history.add_transaction(TransactionRecord {
            txid: txid.clone(),
            to_address: to_address.to_string(),
            amount: amount_zec,
            timestamp: chrono::Utc::now(),
            memo: memo.unwrap_or_default(),
        })?;

        Ok(txid)
    }

    pub async fn sync(&mut self) -> Result<(), FaucetError> {
        self.client.sync_and_await().await.map_err(|e| {
            FaucetError::Wallet(format!("Sync failed: {}", e))
        })?;
        Ok(())
    }

    pub fn get_transaction_history(&self, limit: usize) -> Vec<TransactionRecord> {
        self.history.get_recent(limit)
    }

    pub fn get_stats(&self) -> (usize, f64) {
        let txs = self.history.get_all();
        let count = txs.len();
        let total_sent: f64 = txs.iter().map(|tx| tx.amount).sum();
        (count, total_sent)
    }
}
