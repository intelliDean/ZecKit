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
use zcash_protocol::consensus::BlockHeight;
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

pub struct ExtraWallet {
    pub client: LightClient,
    pub history: TransactionHistory,
}

pub struct WalletManager {
    pub default_wallet: ExtraWallet,
    pub extra_wallets: std::collections::HashMap<String, ExtraWallet>,
    pub data_dir: std::path::PathBuf,
    pub server_uri: String,
}

fn derive_seed_for_id(id: &str) -> Result<String, FaucetError> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(b"zeckit-dynamic-wallet-salt:");
    hasher.update(id.as_bytes());
    let entropy = hasher.finalize();
    // Coerce GenericArray to &[u8] via deref
    let mnemonic: bip0039::Mnemonic<bip0039::English> = bip0039::Mnemonic::from_entropy(&*entropy)
        .map_err(|e| FaucetError::Wallet(format!("Entropy derivation failed: {}", e)))?;
    Ok(mnemonic.phrase().to_string())
}

fn get_configured_activation_heights() -> ConfiguredActivationHeights {
    let mut heights = ConfiguredActivationHeights {
        before_overwinter: Some(1),
        overwinter: Some(1),
        sapling: Some(1),
        blossom: Some(1),
        heartwood: Some(1),
        canopy: Some(1),
        nu5: Some(1),
        nu6: Some(1),
        nu6_1: Some(1),
        nu7: None,
    };

    if let Ok(env_val) = std::env::var("ZECKIT_ACTIVATION_HEIGHTS") {
        for part in env_val.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() == 2 {
                let key = kv[0].trim().to_lowercase();
                if let Ok(height) = kv[1].trim().parse::<u32>() {
                    let block_height = Some(height);
                    match key.as_str() {
                        "before_overwinter" => heights.before_overwinter = block_height,
                        "overwinter" => heights.overwinter = block_height,
                        "sapling" => heights.sapling = block_height,
                        "blossom" => heights.blossom = block_height,
                        "heartwood" => heights.heartwood = block_height,
                        "canopy" => heights.canopy = block_height,
                        "nu5" => heights.nu5 = block_height,
                        "nu6" => heights.nu6 = block_height,
                        "nu6_1" | "nu6.1" => heights.nu6_1 = block_height,
                        "nu7" => heights.nu7 = block_height,
                        _ => {}
                    }
                }
            }
        }
    }

    heights
}

impl WalletManager {
    async fn load_or_create_wallet_client(
        wallet_dir: PathBuf,
        server_uri: &str,
        seed_phrase: &str,
    ) -> Result<LightClient, FaucetError> {
        let uri: Uri = server_uri.parse().map_err(|e| {
            FaucetError::Wallet(format!("Invalid server URI: {}", e))
        })?;

        std::fs::create_dir_all(&wallet_dir).map_err(|e| {
            FaucetError::Wallet(format!("Failed to create wallet directory: {}", e))
        })?;

        let activation_heights = get_configured_activation_heights();
        let chain_type = ChainType::Regtest(activation_heights);
        
        let config = ZingoConfig::build(chain_type)
            .set_lightwalletd_uri(uri)
            .set_wallet_dir(wallet_dir.clone())
            .create();

        let wallet_path = wallet_dir.join("zingo-wallet.dat");
        
        let client = if wallet_path.exists() {
            info!("Loading existing wallet from {:?}", wallet_path);
            LightClient::create_from_wallet_path(config).map_err(|e| {
                FaucetError::Wallet(format!("Failed to load wallet: {}", e))
            })?
        } else {
            info!("Creating new wallet with deterministic seed");
            
            let mnemonic: bip0039::Mnemonic<bip0039::English> = bip0039::Mnemonic::from_phrase(seed_phrase)
                .map_err(|e| FaucetError::Wallet(format!("Invalid mnemonic phrase: {}", e)))?;
            
            let wallet = LightWallet::new(
                chain_type,
                WalletBase::Mnemonic {
                    mnemonic,
                    no_of_accounts: std::num::NonZeroU32::new(1).unwrap(),
                },
                BlockHeight::from_u32(1), // Scan from regtest genesis
                config.wallet_settings.clone(),
            ).map_err(|e| {
                FaucetError::Wallet(format!("Failed to create wallet: {}", e))
            })?;
            
            LightClient::create_from_wallet(wallet, config, false).map_err(|e| {
                FaucetError::Wallet(format!("Failed to create client from wallet: {}", e))
            })?
        };

        Ok(client)
    }

    pub async fn new(
        data_dir: PathBuf,
        server_uri: String,
    ) -> Result<Self, FaucetError> {
        info!("Initializing ZecKit Faucet WalletManager");
        
        std::fs::create_dir_all(&data_dir).map_err(|e| {
            FaucetError::Wallet(format!("Failed to create data directory: {}", e))
        })?;

        // Initialize default wallet
        let seed_manager = SeedManager::new(&data_dir);
        let seed_phrase = seed_manager.get_or_create_seed()?;
        
        let default_client = Self::load_or_create_wallet_client(
            data_dir.clone(),
            &server_uri,
            &seed_phrase,
        ).await?;
        let default_history = TransactionHistory::load(&data_dir)?;
        
        let default_wallet = ExtraWallet {
            client: default_client,
            history: default_history,
        };

        // Scan and load existing dynamic wallets
        let mut extra_wallets = std::collections::HashMap::new();
        let wallets_dir = data_dir.join("wallets");
        if wallets_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&wallets_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        if let Some(id_str) = entry.file_name().to_str() {
                            let id = id_str.to_string();
                            info!("Found existing dynamic wallet: {}", id);
                            let wallet_dir = entry.path();
                            
                            // Derive deterministic seed phrase for this id
                            let wallet_seed = derive_seed_for_id(&id)?;
                            match Self::load_or_create_wallet_client(
                                wallet_dir.clone(),
                                &server_uri,
                                &wallet_seed,
                            ).await {
                                Ok(client) => {
                                    if let Ok(history) = TransactionHistory::load(&wallet_dir) {
                                        extra_wallets.insert(id, ExtraWallet { client, history });
                                    } else {
                                        tracing::warn!("Failed to load transaction history for extra wallet {}", id);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to load extra wallet {}: {}", id, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            default_wallet,
            extra_wallets,
            data_dir,
            server_uri,
        })
    }

    pub fn get_wallet(&self, id: Option<&str>) -> Result<&ExtraWallet, FaucetError> {
        match id {
            None | Some("") | Some("default") => Ok(&self.default_wallet),
            Some(name) => self.extra_wallets.get(name)
                .ok_or_else(|| FaucetError::Wallet(format!("Wallet not found: {}", name))),
        }
    }

    pub fn get_wallet_mut(&mut self, id: Option<&str>) -> Result<&mut ExtraWallet, FaucetError> {
        match id {
            None | Some("") | Some("default") => Ok(&mut self.default_wallet),
            Some(name) => self.extra_wallets.get_mut(name)
                .ok_or_else(|| FaucetError::Wallet(format!("Wallet not found: {}", name))),
        }
    }

    pub fn get_wallet_ids(&self) -> Vec<String> {
        let mut ids = vec!["default".to_string()];
        let mut extra_ids: Vec<String> = self.extra_wallets.keys().cloned().collect();
        extra_ids.sort();
        ids.extend(extra_ids);
        ids
    }

    pub async fn spawn_wallet(&mut self, id: &str) -> Result<(), FaucetError> {
        if id.is_empty() {
            return Err(FaucetError::Wallet("Wallet ID cannot be empty".to_string()));
        }
        if id == "default" {
            return Err(FaucetError::Wallet("Cannot spawn wallet with reserved name 'default'".to_string()));
        }
        if self.extra_wallets.contains_key(id) {
            return Ok(()); // Already exists
        }

        info!("Spawning dynamic wallet: {}", id);
        let wallet_dir = self.data_dir.join("wallets").join(id);
        let seed_phrase = derive_seed_for_id(id)?;

        let client = Self::load_or_create_wallet_client(
            wallet_dir.clone(),
            &self.server_uri,
            &seed_phrase,
        ).await?;

        let history = TransactionHistory::load(&wallet_dir)?;

        self.extra_wallets.insert(id.to_string(), ExtraWallet { client, history });
        Ok(())
    }

    pub async fn get_unified_address(&self, id: Option<&str>) -> Result<String, FaucetError> {
        let w = self.get_wallet(id)?;
        let addresses_json = w.client.unified_addresses_json().await;
        
        let first_address = addresses_json[0]["encoded_address"]
            .as_str()
            .ok_or_else(|| FaucetError::Wallet("No unified address found".to_string()))?;
        
        Ok(first_address.to_string())
    }

    pub async fn get_transparent_address(&self, id: Option<&str>) -> Result<String, FaucetError> {
        let w = self.get_wallet(id)?;
        let addresses_json = w.client.transparent_addresses_json().await;
        
        let first_address = addresses_json[0]["encoded_address"]
            .as_str()
            .ok_or_else(|| FaucetError::Wallet("No transparent address found".to_string()))?;
        
        Ok(first_address.to_string())
    }

    pub async fn get_balance(&self, id: Option<&str>) -> Result<Balance, FaucetError> {
        let w = self.get_wallet(id)?;
        let account_balance = w.client
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

    pub async fn get_height(&self, id: Option<&str>) -> Result<u32, FaucetError> {
        let w = self.get_wallet(id)?;
        let info_str = w.client.do_info().await;
        let info_json: serde_json::Value = serde_json::from_str(&info_str)
            .map_err(|e| FaucetError::Wallet(format!("Failed to parse info JSON: {}", e)))?;
        
        let height = info_json["latest_block_height"]
            .as_u64()
            .ok_or_else(|| FaucetError::Wallet("latest_block_height missing from info".to_string()))?;
            
        Ok(height as u32)
    }

    pub async fn shield_to_orchard(&mut self, id: Option<&str>) -> Result<String, FaucetError> {
        info!("Shielding transparent funds to Orchard for wallet {:?}...", id);
        
        let balance = self.get_balance(id).await?;
        
        if balance.transparent == Zatoshis::ZERO {
            return Err(FaucetError::Wallet("No transparent funds to shield".to_string()));
        }
        
        info!("Shielding {} ZEC from transparent to orchard", balance.transparent_zec());
        
        // Step 1: Propose the shield transaction
        let proposal_result = {
            let w = self.get_wallet_mut(id)?;
            w.client.propose_shield(zip32::AccountId::ZERO).await
        };

        let _proposal = match proposal_result {
            Ok(p) => p,
            Err(e) if e.to_string().contains("additional change output") => {
                 return self.perform_fallback_shield_transfer(id, balance.transparent).await;
            },
            Err(e) => return Err(FaucetError::Wallet(format!("Shield proposal failed: {}", e)))
        };

        // Step 2: Send the stored proposal
        let send_result = {
            let w = self.get_wallet_mut(id)?;
            w.client.send_stored_proposal(true).await
        };

        match send_result {
            Ok(txids) => {
                let txid = txids.first().to_string();
                info!(" ✓ Shield transaction broadcast successfully: {}", txid);
                Ok(txid)
            },
            Err(e) if e.to_string().contains("additional change output") => {
                 info!(" ⚠ Shield proposal failed with 'additional change output' - likely too many UTXOs or fee issues. Falling back to simple transfer...");
                 self.perform_fallback_shield_transfer(id, balance.transparent).await
            },
            Err(e) => {
                tracing::error!(" ❌ Shielding failed during send: {}", e);
                Err(FaucetError::Wallet(format!("Shield send failed: {}", e)))
            }
        }
    }

    async fn perform_fallback_shield_transfer(&mut self, id: Option<&str>, utxo_total: Zatoshis) -> Result<String, FaucetError> {
        info!("Fallback: Shielding failed (change output error). Attempting manual transfer...");
        
        // Fee for large consolidation (0.01 ZEC is safe for ~500 inputs)
        let fee = Zatoshis::from_u64(1_000_000).unwrap(); 
        
        if utxo_total <= fee {
            return Err(FaucetError::Wallet("Insufficient funds for fallback shielding".to_string()));
        }
        
        let mut amount_to_shield_zat = (utxo_total - fee).unwrap();
        
        let max_batch_zat = Zatoshis::from_u64(100_000_000 * 100).unwrap();
        if amount_to_shield_zat > max_batch_zat {
            info!("  ⚠ Large balance detected ({} ZEC). Shielding in batch of 100.0 ZEC...", utxo_total.into_u64() as f64 / 100_000_000.0);
            amount_to_shield_zat = max_batch_zat;
        }

        let recipient = self.get_unified_address(id).await?;
        let amount_zec = amount_to_shield_zat.into_u64() as f64 / 100_000_000.0;
        
        self.send_from_transparent(id, &recipient, amount_zec, Some("ZecKit Batch Shield".to_string())).await
    }

    pub async fn send_from_transparent(
        &mut self,
        id: Option<&str>,
        to_address: &str,
        amount_zec: f64,
        memo: Option<String>,
    ) -> Result<String, FaucetError> {
        info!("Sending {} ZEC (from transparent) to {}", amount_zec, &to_address[..to_address.len().min(16)]);

        let amount_zatoshis = (amount_zec * 100_000_000.0) as u64;
        let recipient_address = to_address.parse()
            .map_err(|e| FaucetError::Wallet(format!("Invalid address: {}", e)))?;
        let amount = zcash_protocol::value::Zatoshis::from_u64(amount_zatoshis)
            .map_err(|_| FaucetError::Wallet("Invalid amount".to_string()))?;

        let memo_bytes = if to_address.starts_with('t') {
            None
        } else if let Some(memo_text) = &memo {
            let bytes = memo_text.as_bytes();
            let mut padded = [0u8; 512];
            padded[..bytes.len().min(512)].copy_from_slice(&bytes[..bytes.len().min(512)]);
            Some(MemoBytes::from_bytes(&padded).unwrap())
        } else {
            None
        };

        let payment = Payment::new(recipient_address, amount, memo_bytes, None, None, vec![])
            .ok_or_else(|| FaucetError::Wallet("Failed to create payment".to_string()))?;

        let request = TransactionRequest::new(vec![payment])
            .map_err(|e| FaucetError::Wallet(format!("Failed to create request: {}", e)))?;

        let w = self.get_wallet_mut(id)?;
        let txids = w.client
            .quick_send(request, zip32::AccountId::ZERO, false)
            .await
            .map_err(|e| FaucetError::TransactionFailed(format!("Fallback send failed: {}", e)))?;

        Ok(txids.first().to_string())
    }

    pub async fn send_transaction(
        &mut self,
        id: Option<&str>,
        to_address: &str,
        amount_zec: f64,
        memo: Option<String>,
    ) -> Result<String, FaucetError> {
        info!("Sending {} ZEC to {}", amount_zec, &to_address[..to_address.len().min(16)]);

        let amount_zatoshis = (amount_zec * 100_000_000.0) as u64;

        let balance = self.get_balance(id).await?;
        if balance.orchard < Zatoshis::from_u64(amount_zatoshis).unwrap() {
            return Err(FaucetError::InsufficientBalance(format!(
                "Need {} ZEC, have {} ZEC in Orchard pool",
                amount_zec,
                balance.orchard_zec()
            )));
        }

        let recipient_address = to_address.parse()
            .map_err(|e| FaucetError::Wallet(format!("Invalid address: {}", e)))?;

        let amount = zcash_protocol::value::Zatoshis::from_u64(amount_zatoshis)
            .map_err(|_| FaucetError::Wallet("Invalid amount".to_string()))?;

        let memo_bytes = if to_address.starts_with('t') {
            None
        } else if let Some(memo_text) = &memo {
            let bytes = memo_text.as_bytes();
            if bytes.len() > 512 {
                return Err(FaucetError::Wallet("Memo too long (max 512 bytes)".to_string()));
            }
            
            let mut padded = [0u8; 512];
            padded[..bytes.len()].copy_from_slice(bytes);
            
            Some(MemoBytes::from_bytes(&padded)
                .map_err(|e| FaucetError::Wallet(format!("Invalid memo: {}", e)))?)
        } else {
            None
        };

        let payment = Payment::new(
            recipient_address,
            amount,
            memo_bytes,
            None,
            None,
            vec![],
        ).ok_or_else(|| FaucetError::Wallet("Failed to create payment".to_string()))?;

        let request = TransactionRequest::new(vec![payment])
            .map_err(|e| FaucetError::Wallet(format!("Failed to create request: {}", e)))?;

        let w = self.get_wallet_mut(id)?;
        let txids = w.client
            .quick_send(request, zip32::AccountId::ZERO, false)
            .await
            .map_err(|e| {
                FaucetError::TransactionFailed(format!("Failed to send transaction: {}", e))
            })?;

        let txid = txids.first().to_string();

        w.history.add_transaction(TransactionRecord {
            txid: txid.clone(),
            to_address: to_address.to_string(),
            amount: amount_zec,
            timestamp: chrono::Utc::now(),
            memo: memo.unwrap_or_default(),
        })?;

        Ok(txid)
    }

    pub async fn sync(&mut self, id: Option<&str>) -> Result<(), FaucetError> {
        let w = self.get_wallet_mut(id)?;
        w.client.sync_and_await().await.map_err(|e| {
            FaucetError::Wallet(format!("Sync failed: {}", e))
        })?;
        Ok(())
    }

    pub fn get_transaction_history(&self, id: Option<&str>, limit: usize) -> Result<Vec<TransactionRecord>, FaucetError> {
        let w = self.get_wallet(id)?;
        Ok(w.history.get_recent(limit))
    }

    pub fn get_stats(&self, id: Option<&str>) -> Result<(usize, f64), FaucetError> {
        let w = self.get_wallet(id)?;
        let txs = w.history.get_all();
        let count = txs.len();
        let total_sent: f64 = txs.iter().map(|tx| tx.amount).sum();
        Ok((count, total_sent))
    }
}
