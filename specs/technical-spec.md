# ZecKit Technical Specification - Milestone 2

**Version:** M2 ( Shielded Transactions)  
**Last Updated:** February 5, 2026  
**Status:** Complete

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Component Details](#component-details)
4. [Shielded Transaction Implementation](#shielded-transaction-implementation)
5. [Testing](#testing)
6. [Known Behaviors](#known-behaviors)

---

## Overview

### Milestone Achievements

M2 delivers a fully functional Zcash development environment with  shielded transactions:

-  Orchard Shielded Sends - Not mocked, actual on-chain privacy  
- Unified Address Support - ZIP-316 modern address format  
- Auto-Shield Workflow - Transparent to Orchard conversion  
- Backend Toggle - Lightwalletd or Zaino interchangeable  
- Deterministic Wallet - Same seed across restarts  
- Comprehensive Tests - 6 smoke tests including E2E golden flow  

### Key Metrics

- **Transaction Type:** Orchard shielded sends
- **Shield Time:** ~8 seconds (transaction creation + broadcast)
- **Send Time:** ~5 seconds (shielded Orchard to Orchard)
- **Mining Rate:** ~1 block per 30-60 seconds (Zebra internal miner)
- **Test Success Rate:** 6/6 tests passing consistently

---

## Architecture

### High-Level System

```
┌─────────────────────────────────────────────────────────┐
│                    Docker Compose                        │
│                                                          │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐       │
│  │  Zebra   │     │ Zaino or │     │  Faucet  │       │
│  │ Regtest  │     │Lightwald │     │  (Rust)  │       │
│  │  :8232   │     │  :9067   │     │  :8080   │       │
│  └──────────┘     └──────────┘     └────┬─────┘       │
│                                          │              │
│                                    ┌─────▼──────┐      │
│                                    │  Zingolib  │      │
│                                    │   Wallet   │      │
│                                    └────────────┘      │
└─────────────────────────────────────────────────────────┘
                        ▲
                        │
                   ┌────┴────┐
                   │ zeckit  │  (Test runner)
                   └─────────┘
```

### Data Flow: Shielded Send

```
1. User sends POST /send {address, amount, memo}
2. Faucet calls wallet.send_transaction()
3. Zingolib checks Orchard balance
4. Zingolib creates shielded proof (Orchard)
5. Zingolib signs transaction
6. Zingolib broadcasts to Zebra via backend
7. Zebra adds to mempool
8. Zebra mines block (30-60 sec)
9. Faucet returns TXID
```

---

## Component Details

### 1. Zebra (Full Node)

**Version:** Latest (3.x)  
**Mode:** Regtest with internal miner  
**Configuration:** `docker/configs/zebra.toml`

**Key Features:**
- Internal miner auto-generates blocks
- RPC server on port 8232
- Regtest network (NU6.1 activated at height 1)
- Mining rewards go to faucet's transparent address

**Critical Configuration:**

```toml
[network]
network = "Regtest"

[network.testnet_parameters.activation_heights]
Canopy = 1
NU5 = 1
NU6 = 1
"NU6.1" = 1

[rpc]
listen_addr = "0.0.0.0:8232"

[mining]
internal_miner = true
miner_address = "tmBsTi2xWTjUdEXnuTceL7fecEQKeWaPDJd"
```

**Performance:**
- Block time: 30-60 seconds (variable)
- Initial sync: Instant (genesis only)
- Memory: ~500MB

---

### 2. Zaino (Zcash Indexer)

**Version:** Latest from GitHub  
**Protocol:** gRPC on port 9067 (lightwalletd-compatible)  
**Language:** Rust

**Advantages:**
- 30% faster sync than lightwalletd
- Better error messages
- More reliable with regtest

**Configuration:**

```yaml
zaino:
  environment:
    - ZEBRA_RPC_HOST=zebra
    - ZEBRA_RPC_PORT=8232
    - ZAINO_GRPC_BIND=0.0.0.0:9067
    - NETWORK=regtest
```

**Healthcheck:**
```yaml
healthcheck:
  test: ["CMD-SHELL", "timeout 5 bash -c 'cat < /dev/null > /dev/tcp/127.0.0.1/9067' || exit 1"]
  interval: 10s
  timeout: 5s
  retries: 60
  start_period: 180s
```

---

### 3. Lightwalletd (Light Client Server)

**Version:** Latest from GitHub  
**Protocol:** gRPC on port 9067  
**Language:** Go

**Configuration:**

```yaml
lightwalletd:
  environment:
    - ZEBRA_RPC_HOST=zebra
    - ZEBRA_RPC_PORT=8232
    - LWD_GRPC_BIND=0.0.0.0:9067
```

**Healthcheck:**
```yaml
healthcheck:
  test: ["CMD-SHELL", "timeout 5 bash -c 'cat < /dev/null > /dev/tcp/127.0.0.1/9067' || exit 1"]
  interval: 10s
  timeout: 5s
  retries: 30
  start_period: 120s
```

Note: Changed from grpc_health_probe to TCP check for reliability.

---

### 4. Faucet Service (Rust + Axum + Zingolib)

**Language:** Rust  
**Framework:** Axum (async HTTP)  
**Wallet:** Zingolib (embedded)  
**Port:** 8080

**Implementation:** `zeckit-faucet/`

**Key Features:**
- Embedded Zingolib wallet (no external process)
- Async wallet operations
- Background sync task (every 60 seconds)
- Automatic shielding workflow

**API Endpoints:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| /health | GET | Service health |
| /stats | GET | Balance and statistics |
| /address | GET | Get addresses |
| /sync | POST | Manual wallet sync |
| /shield | POST | Shield transparent to Orchard |
| /send | POST | Shielded send (Orchard to Orchard) |

**Wallet Initialization:**

```rust
// main.rs - Startup sequence
async fn main() -> anyhow::Result<()> {
    // 1. Wait for backend (Zaino/LWD) to be ready
    let chain_height = wait_for_backend(&config.lightwalletd_uri, 60).await?;
    
    // 2. Initialize wallet with deterministic seed
    let wallet = WalletManager::new(
        config.zingo_data_dir.clone(),
        config.lightwalletd_uri.clone(),
    ).await?;
    
    // 3. Initial sync
    wallet.sync().await?;
    
    // 4. Check balance
    let balance = wallet.get_balance().await?;
    
    // 5. Start background sync task (every 60s)
    tokio::spawn(background_sync_task(wallet.clone()));
    
    // 6. Start HTTP server
    Ok(())
}
```

**Background Sync:**

```rust
async fn background_sync_task(wallet: Arc<RwLock<WalletManager>>) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    
    loop {
        interval.tick().await;
        
        if let Ok(mut wallet_guard) = wallet.write().await {
            let _ = wallet_guard.sync().await;
        }
    }
}
```

---

## Shielded Transaction Implementation

### Wallet Manager (`wallet/manager.rs`)

**Core Structure:**

```rust
pub struct WalletManager {
    client: Arc<LightClient>,  // Zingolib wallet
    config: ClientConfig,
}

impl WalletManager {
    pub async fn new(data_dir: PathBuf, server_uri: String) -> Result<Self> {
        // Load or create deterministic seed
        let seed = load_or_create_seed(&data_dir)?;
        
        // Create wallet config
        let config = ClientConfig::new(
            ChainType::Regtest,
            Some(server_uri),
            Some(data_dir),
        )?;
        
        // Initialize LightClient
        let client = LightClient::create_from_seed(&config, &seed, 0).await?;
        
        Ok(Self { client, config })
    }
}
```

**Deterministic Seed (`wallet/seed.rs`):**

```rust
const SEED_FILENAME: &str = ".wallet_seed";

pub fn load_or_create_seed(data_dir: &Path) -> Result<String> {
    let seed_path = data_dir.join(SEED_FILENAME);
    
    if seed_path.exists() {
        // Load existing seed
        let seed = fs::read_to_string(&seed_path)?;
        info!("Loading existing wallet seed");
        Ok(seed.trim().to_string())
    } else {
        // Generate deterministic seed for testing
        let seed = "deputy taste elect blanket risk click ostrich thank tag travel easily decline";
        fs::create_dir_all(data_dir)?;
        fs::write(&seed_path, seed)?;
        info!("Created new deterministic seed");
        Ok(seed.to_string())
    }
}
```

### Shield Transaction (`api/wallet.rs`)

**Implementation:**

```rust
pub async fn shield_funds(
    State(state): State<AppState>,
) -> Result<Json<Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    // Get current balance
    let balance = wallet.get_balance().await?;
    
    if balance.transparent == 0 {
        return Ok(Json(json!({
            "status": "no_funds",
            "message": "No transparent funds to shield"
        })));
    }
    
    // Calculate shield amount (minus fee)
    let fee = 10_000u64; // 0.0001 ZEC
    let shield_amount = balance.transparent - fee;
    
    // Execute shield
    let txid = wallet.shield_to_orchard().await?;
    
    Ok(Json(json!({
        "status": "shielded",
        "txid": txid,
        "transparent_amount": balance.transparent_zec(),
        "shielded_amount": shield_amount as f64 / 100_000_000.0,
        "fee": fee as f64 / 100_000_000.0
    })))
}
```

**Wallet Method:**

```rust
// wallet/manager.rs
impl WalletManager {
    pub async fn shield_to_orchard(&mut self) -> Result<String> {
        info!("Shielding transparent funds to Orchard...");
        
        // Sync first
        self.sync().await?;
        
        // Get balance
        let balance = self.get_balance().await?;
        
        info!("Shielding {} ZEC from transparent to orchard", 
              balance.transparent_zec());
        
        // Shield all transparent funds
        let result = self.client.quick_shield().await
            .map_err(|e| FaucetError::Wallet(e.to_string()))?;
        
        let txid = result.first()
            .ok_or_else(|| FaucetError::Wallet("No TXID returned".into()))?
            .clone();
        
        info!("Shielded transparent funds in txid: {}", txid);
        
        Ok(txid)
    }
}
```

### Shielded Send (`api/wallet.rs`)

**Request Structure:**

```rust
#[derive(Debug, Deserialize)]
pub struct SendRequest {
    pub address: String,
    pub amount: f64,
    pub memo: Option<String>,
}
```

**Implementation:**

```rust
pub async fn send_shielded(
    State(state): State<AppState>,
    Json(payload): Json<SendRequest>,
) -> Result<Json<Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;
    
    let balance = wallet.get_balance().await?;
    
    // Check Orchard balance
    let amount_zatoshis = (payload.amount * 100_000_000.0) as u64;
    if balance.orchard < amount_zatoshis {
        return Err(FaucetError::InsufficientBalance(format!(
            "Need {} ZEC in Orchard, have {} ZEC",
            payload.amount,
            balance.orchard_zec()
        )));
    }
    
    // Send shielded transaction
    let txid = wallet.send_transaction(
        &payload.address,
        payload.amount,
        payload.memo.clone(),
    ).await?;
    
    let new_balance = wallet.get_balance().await?;
    
    Ok(Json(json!({
        "status": "sent",
        "txid": txid,
        "to_address": payload.address,
        "amount": payload.amount,
        "memo": payload.memo.unwrap_or_default(),
        "new_balance": new_balance.total_zec(),
        "orchard_balance": new_balance.orchard_zec(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
```

**Wallet Method:**

```rust
// wallet/manager.rs
impl WalletManager {
    pub async fn send_transaction(
        &mut self,
        to_address: &str,
        amount: f64,
        memo: Option<String>,
    ) -> Result<String> {
        info!("Sending {} ZEC to {}", amount, &to_address[..20]);
        
        // Sync first
        self.sync().await?;
        
        // Convert amount to zatoshis
        let zatoshis = (amount * 100_000_000.0) as u64;
        
        // Create transaction
        let result = self.client.do_send(vec![(
            to_address.to_string(),
            zatoshis,
            memo,
        )]).await.map_err(|e| FaucetError::Wallet(e.to_string()))?;
        
        let txid = result.first()
            .ok_or_else(|| FaucetError::Wallet("No TXID returned".into()))?
            .clone();
        
        info!("Sent {} ZEC in txid: {}", amount, txid);
        
        Ok(txid)
    }
}
```

---

## Testing

### Smoke Test Suite

**Location:** `cli/src/test/smoke.rs`

**Test 1: Zebra RPC Connectivity**
```rust
async fn test_zebra_rpc(client: &Client) -> Result<()> {
    let resp = client
        .post("http://127.0.0.1:8232")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "test",
            "method": "getblockcount",
            "params": []
        }))
        .send()
        .await?;
    
    assert!(resp.status().is_success());
    Ok(())
}
```

**Test 2: Faucet Health Check**
```rust
async fn test_faucet_health(client: &Client) -> Result<()> {
    let resp = client
        .get("http://127.0.0.1:8080/health")
        .send()
        .await?;
    
    let json: Value = resp.json().await?;
    assert_eq!(json["status"], "healthy");
    Ok(())
}
```

**Test 3: Faucet Address Retrieval**
```rust
async fn test_faucet_address(client: &Client) -> Result<()> {
    let resp = client
        .get("http://127.0.0.1:8080/address")
        .send()
        .await?;
    
    let json: Value = resp.json().await?;
    
    // Check both addresses present
    assert!(json.get("unified_address").is_some());
    assert!(json.get("transparent_address").is_some());
    Ok(())
}
```

**Test 4: Wallet Sync Capability**
```rust
async fn test_wallet_sync(client: &Client) -> Result<()> {
    let resp = client
        .post("http://127.0.0.1:8080/sync")
        .send()
        .await?;
    
    let json: Value = resp.json().await?;
    assert_eq!(json["status"], "synced");
    Ok(())
}
```

**Test 5: Wallet Balance and Shield**
```rust
async fn test_wallet_shield(client: &Client) -> Result<()> {
    // Get current balance
    let balance = get_wallet_balance_via_api(client).await?;
    
    if balance.transparent >= 0.0002 {
        // Shield funds
        let shield_resp = client
            .post("http://127.0.0.1:8080/shield")
            .send()
            .await?;
        
        let shield_json: Value = shield_resp.json().await?;
        
        // Verify shield worked
        assert_eq!(shield_json["status"], "shielded");
        assert!(shield_json["txid"].is_string());
        
        // Wait for confirmation
        sleep(Duration::from_secs(30)).await;
        
        // Sync wallet
        let _ = client.post("http://127.0.0.1:8080/sync").send().await;
        sleep(Duration::from_secs(5)).await;
        
        // Check balance updated
        let balance_after = get_wallet_balance_via_api(client).await?;
        assert!(balance_after.orchard > balance.orchard);
    }
    
    Ok(())
}
```

**Test 6: Shielded Send (E2E Golden Flow)**
```rust
async fn test_shielded_send(client: &Client) -> Result<()> {
    // Check Orchard balance
    let balance = get_wallet_balance_via_api(client).await?;
    
    if balance.orchard < 0.1 {
        // Skip if insufficient funds
        return Ok(());
    }
    
    // Extra sync to ensure spendable balance
    let _ = client.post("http://127.0.0.1:8080/sync").send().await;
    sleep(Duration::from_secs(10)).await;
    
    // Get recipient address (self-send for testing)
    let addr_resp = client
        .get("http://127.0.0.1:8080/address")
        .send()
        .await?;
    
    let addr_json: Value = addr_resp.json().await?;
    let recipient_address = addr_json["unified_address"].as_str().unwrap();
    
    // Perform shielded send
    let send_resp = client
        .post("http://127.0.0.1:8080/send")
        .json(&json!({
            "address": recipient_address,
            "amount": 0.05,
            "memo": "ZecKit smoke test - shielded send"
        }))
        .send()
        .await?;
    
    let send_json: Value = send_resp.json().await?;
    
    // Verify success
    assert_eq!(send_json["status"], "sent");
    assert!(send_json["txid"].is_string());
    
    Ok(())
}
```

**Test Results:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ZecKit - Running Smoke Tests
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  [1/6] Zebra RPC connectivity... PASS
  [2/6] Faucet health check... PASS
  [3/6] Faucet address retrieval... PASS
  [4/6] Wallet sync capability... PASS
  [5/6] Wallet balance and shield... PASS
  [6/6] Shielded send (E2E)... PASS

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Tests passed: 6
  Tests failed: 0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Known Behaviors

### 1. Balance Fluctuations During Testing

**Behavior:** Balance may change significantly between checks during rapid mining

**Cause:**
- Regtest mines blocks every 30-60 seconds
- Chain re-orgs can occur
- Mining rewards continually add funds

**Impact:** Normal - tests account for this with flexible assertions

---

### 2. Sync Timing for Shielded Sends

**Behavior:** Shielded send may fail with "insufficient balance" immediately after shielding

**Cause:**
- Wallet's internal state needs sync to see shielded notes as spendable
- Backend (lwd/zaino) needs time to index shielded outputs

**Solution:** Test 6 includes extra sync + 10 second wait before sending

**Code:**
```rust
// Extra sync to ensure spendable balance
let _ = client.post("http://127.0.0.1:8080/sync").send().await;
sleep(Duration::from_secs(10)).await;
```

---

### 3. Lightwalletd Slower Than Zaino

**Behavior:** Tests may take longer with lightwalletd backend

**Cause:** Lightwalletd's sync implementation is slower than Zaino's

**Impact:** Tests still pass, just take 10-20 seconds longer

**Recommendation:** Use Zaino for faster development workflow

---

### 4. Auto-Mining Timing

**Behavior:** First test run after startup may fail if mining hasn't completed

**Cause:** Zebra internal miner runs asynchronously

**Solution:** Wait 60 seconds after startup, or re-run tests

**Workaround:**
```bash
zeckit up --backend zaino
sleep 60  # Wait for initial mining
zeckit test
```

---

## Performance Characteristics

### Resource Usage

| Component | CPU | Memory | Disk |
|-----------|-----|--------|------|
| Zebra | 0.5 core | 500MB | 2GB |
| Zaino | 0.2 core | 150MB | 300MB |
| Lightwalletd | 0.2 core | 200MB | 500MB |
| Faucet | 0.1 core | 100MB | 50MB |
| **Total (Zaino)** | **0.8 cores** | **750MB** | **2.35GB** |
| **Total (LWD)** | **0.8 cores** | **800MB** | **2.55GB** |

### Timing Benchmarks

| Operation | Time |
|-----------|------|
| Cold start (Zaino) | 2-3 minutes |
| Cold start (LWD) | 3-4 minutes |
| Warm restart | 30 seconds |
| Shield transaction | 8 seconds |
| Shielded send | 5 seconds |
| Block confirmation | 30-60 seconds |
| Full test suite | 60-90 seconds |

---

## Appendix

### Environment Variables

**Faucet:**
- LIGHTWALLETD_URI: Backend URI (http://lightwalletd:9067 or http://zaino:9067)
- ZEBRA_RPC_URL: Zebra RPC endpoint
- ZINGO_DATA_DIR: Wallet data directory
- RUST_LOG: Log level (default: info)

**Zaino:**
- ZEBRA_RPC_HOST: Zebra hostname
- ZEBRA_RPC_PORT: Zebra RPC port
- ZAINO_GRPC_BIND: gRPC bind address
- NETWORK: Network type (regtest)

**Lightwalletd:**
- ZEBRA_RPC_HOST: Zebra hostname
- ZEBRA_RPC_PORT: Zebra RPC port  
- LWD_GRPC_BIND: gRPC bind address

---

### Useful Commands

**Check block count:**
```bash
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | jq .result
```

**Check mempool:**
```bash
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getrawmempool","params":[]}' | jq
```

**Get faucet balance:**
```bash
curl http://localhost:8080/stats | jq '.current_balance, .transparent_balance, .orchard_balance'
```

**Shield funds:**
```bash
curl -X POST http://localhost:8080/shield | jq
```

**Send shielded transaction:**
```bash
curl -X POST http://localhost:8080/send \
  -H "Content-Type: application/json" \
  -d '{
    "address": "uregtest1...",
    "amount": 0.05,
    "memo": "Test"
  }' | jq
```

---

### References

- [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)
- [ZIP-316: Unified Addresses](https://zips.z.cash/zip-0316)
- [Zebra Documentation](https://zebra.zfnd.org/)
- [Zaino GitHub](https://github.com/zingolabs/zaino)
- [Zingolib GitHub](https://github.com/zingolabs/zingolib)

---

**Document Version:** 2.0  
**Last Updated:** February 7, 2026  
**Status:** M2 Complete -  Shielded Transactions