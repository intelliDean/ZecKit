#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_wallet_initialization() {
        let temp_dir = tempdir().unwrap();
        let server_uri = "http://localhost:9067".to_string();

        // This would need a running regtest network
        // For now, we test the config loading
        let config = Config {
            zingo_data_dir: temp_dir.path().to_path_buf(),
            lightwalletd_uri: server_uri,
            zebra_rpc_url: "http://localhost:8232".to_string(),
            faucet_amount_min: 0.01,
            faucet_amount_max: 100.0,
            faucet_amount_default: 10.0,
        };

        assert_eq!(config.faucet_amount_min, 0.01);
        assert_eq!(config.faucet_amount_max, 100.0);
    }

    #[test]
    fn test_balance_calculations() {
        let balance = Balance {
            transparent: 100_000_000, // 1 ZEC
            sapling: 200_000_000,     // 2 ZEC
            orchard: 300_000_000,     // 3 ZEC
        };

        assert_eq!(balance.total_zatoshis(), 600_000_000);
        assert_eq!(balance.total_zec(), 6.0);
        assert_eq!(balance.orchard_zec(), 3.0);
        assert_eq!(balance.transparent_zec(), 1.0);
    }

    #[test]
    fn test_transaction_history() {
        let temp_dir = tempdir().unwrap();
        let mut history = TransactionHistory::load(temp_dir.path()).unwrap();

        let record = TransactionRecord {
            timestamp: chrono::Utc::now(),
            to_address: "uregtest1test123".to_string(),
            amount: 10.0,
            txid: "abc123".to_string(),
            memo: "test".to_string(),
        };

        history.add_transaction(record.clone()).unwrap();

        let recent = history.get_recent(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].amount, 10.0);
    }
}