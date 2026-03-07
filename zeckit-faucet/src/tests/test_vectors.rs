
#[cfg(test)]
mod address_validation_tests {
    use zcash_address::{ZcashAddress, Network};

    #[test]
    fn test_regtest_unified_address() {
        // Valid regtest unified address
        let address = "uregtest1m0xkgl4q8z6p8pzxdfp4hvvqyfn7nqngz6sjskxsq0q6e0yaq4w7dp9hzq2jdmx8xqtlkvx3mevha2pxpxr0k5sfm29rwc9fj82xa95xtcu0pqpy39crt8g0h9mzqr9r".to_string();
        
        // This should parse successfully
        let parsed = ZcashAddress::try_from_encoded(&address);
        assert!(parsed.is_ok());
        
        if let Ok(addr) = parsed {
            assert_eq!(addr.network(), Network::Regtest);
        }
    }

    #[test]
    fn test_regtest_transparent_address() {
        // Valid regtest transparent address
        let address = "tmGWyihj4Q64yHJutdHKC5FEg2CjzSf2CJ4".to_string();
        
        let parsed = ZcashAddress::try_from_encoded(&address);
        assert!(parsed.is_ok());
        
        if let Ok(addr) = parsed {
            assert_eq!(addr.network(), Network::Regtest);
        }
    }

    #[test]
    fn test_invalid_address() {
        let invalid_addresses = vec![
            "not_an_address",
            "zs1",  // incomplete
            "t1",   // mainnet on regtest
        ];

        for addr in invalid_addresses {
            let parsed = ZcashAddress::try_from_encoded(addr);
            assert!(parsed.is_err(), "Should reject invalid address: {}", addr);
        }
    }

    #[test]
    fn test_mainnet_address_on_regtest() {
        // This is a valid mainnet address but should be rejected on regtest
        let mainnet_addr = "t1Hsc1LR8yKnbbe3twRp88p6vFfC5t7DLbs";
        
        let parsed = ZcashAddress::try_from_encoded(mainnet_addr);
        
        if let Ok(addr) = parsed {
            // Address is valid, but network should be mainnet
            assert_eq!(addr.network(), Network::Main);
            // Our validation should reject this on regtest
        }
    }
}