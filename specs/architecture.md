# ZecKit Architecture

## System Overview

ZecKit is a containerized development toolkit for Zcash that provides  shielded transactions on a local regtest network. It enables developers to test Orchard shielded sends without connecting to testnet or mainnet.

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Docker Compose Network                    │
│                      (zeckit-network)                        │
│                                                              │
│  ┌──────────────┐         ┌──────────────┐                 │
│  │    Zebra     │         │   Faucet     │                 │
│  │  (Rust)      │  RPC    │  (Rust)      │                 │
│  │  regtest     │  8232   │  Axum+       │                 │
│  │              │         │  Zingolib    │                 │
│  │ Auto-mining  │         │  :8080       │                 │
│  └──────┬───────┘         └──────┬───────┘                 │
│         │                        │                          │
│         │                        │                          │
│         ▼                        ▼                          │
│  ┌──────────────┐      ┌──────────────┐                   │
│  │ Zaino (Rust) │      │ Embedded     │                   │
│  │   gRPC :9067 │      │ Zingolib     │                   │
│  │              │      │ Wallet       │                   │
│  └──────────────┘      └──────────────┘                   │
│         OR                                                  │
│  ┌──────────────┐                                          │
│  │Lightwalletd  │                                          │
│  │   (Go)       │                                          │
│  │   gRPC :9067 │                                          │
│  └──────────────┘                                          │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
                       ┌────┴────┐
                       │ zeckit  │  (Rust CLI - test runner)
                       │  test   │
                       └─────────┘
```

---

## Component Architecture

### 1. Zebra Node

**Technology:** Rust  
**Role:** Full Zcash node with internal miner  
**Port:** 8232 (RPC)

**Responsibilities:**
- Validate and store blockchain
- Provide RPC interface
- Auto-mine blocks (regtest mode)
- Broadcast transactions

**Key Features:**
- Internal Miner: Automatically generates blocks every 30-60 seconds
- Regtest Mode: Isolated test network with NU6.1 activated
- No Checkpoint Sync: Allows genesis start for clean testing
- Coinbase Mining: Rewards go to faucet's transparent address

**Configuration Flow:**
```
zebra.toml
    ├── [network] = "Regtest"
    ├── [rpc] listen_addr = "0.0.0.0:8232"
    └── [mining]
        ├── internal_miner = true
        └── miner_address = "tmBsTi2xWTjUdEXnuTceL7fecEQKeWaPDJd"
```

---

### 2. Zaino Indexer

**Technology:** Rust  
**Role:** Light client protocol server (lightwalletd-compatible)  
**Port:** 9067 (gRPC)

**Responsibilities:**
- Index blockchain data from Zebra
- Serve compact blocks to wallet
- Provide transaction broadcast API
- Cache shielded note commitments

**Advantages:**
- 30% faster sync than lightwalletd
- Better error messages
- More reliable with rapid block generation
- Memory-safe (Rust)

**Data Flow:**
```
Zebra → Zaino → Wallet (Zingolib)
  │       │
  │       └──── Indexes: Blocks, Transactions, Notes
  └─────────── Broadcasts: New transactions
```

---

### 3. Lightwalletd Server

**Technology:** Go  
**Role:** Light client protocol server (original implementation)  
**Port:** 9067 (gRPC)

**Responsibilities:**
- Index blockchain data from Zebra
- Serve compact blocks to wallet
- Provide transaction broadcast API
- Cache shielded note commitments

**Configuration:**
```
Zebra RPC: zebra:8232
gRPC Bind: 0.0.0.0:9067
TLS: Disabled (dev only)
```

**Healthcheck Fix:**
- Changed from grpc_health_probe to TCP port check
- More reliable for regtest environment

---

### 4. Faucet Service

**Technology:** Rust + Axum + Zingolib  
**Role:** REST API with embedded shielded wallet  
**Port:** 8080 (HTTP)

**Architecture:**

```
┌──────────────────────────┐
│     Axum HTTP Server     │
│         :8080            │
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│   API Handlers           │
│  • /health               │
│  • /stats                │
│  • /address              │
│  • /sync                 │
│  • /shield               │
│  • /send                 │
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│   WalletManager          │
│  (Rust wrapper)          │
│  • sync()                │
│  • get_balance()         │
│  • shield_to_orchard()   │
│  • send_transaction()    │
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│   Zingolib LightClient   │
│  (Embedded library)      │
│  • Create transactions   │
│  • Sign with keys        │
│  • Broadcast via backend │
└──────────────────────────┘
```

**Key Design Decisions:**

1. Embedded Wallet: No external process, library directly linked
2. Async Everything: Tokio runtime for concurrent operations
3. Deterministic Seed: Same seed = same addresses (testing)
4. Background Sync: Auto-sync every 60 seconds

---

## Data Flow Diagrams

### Startup Sequence

```
1. zeckit up --backend zaino
   │
   ├─► Start Zebra
   │   ├── Load regtest config
   │   ├── Initialize blockchain from genesis
   │   └── Start internal miner
   │
   ├─► Start Zaino
   │   ├── Connect to Zebra RPC
   │   ├── Wait for Zebra to be ready
   │   └── Start indexing blocks
   │
   └─► Start Faucet
       ├── Wait for Zaino to be ready
       ├── Load or create deterministic seed
       ├── Initialize Zingolib wallet
       ├── Sync with blockchain
       ├── Start background sync task
       └── Start HTTP server on :8080

[2-3 minutes later]
   │
   └─► Services ready
       • Zebra mining blocks
       • Zaino indexing
       • Faucet has balance
```

### Shield Transaction Flow

```
User Request
   │
   ├─► POST /shield
   │
   ▼
Faucet API Handler
   │
   ├─► wallet.get_balance()
   │   └── Check transparent balance > 0
   │
   ├─► wallet.shield_to_orchard()
   │   │
   │   ├─► Zingolib: Select transparent UTXOs
   │   ├─► Zingolib: Create Orchard note
   │   ├─► Zingolib: Generate shielded proof
   │   ├─► Zingolib: Sign transaction
   │   └─► Zingolib: Broadcast via Zaino
   │
   ├─► Zaino: Forward to Zebra RPC
   │
   ├─► Zebra: Add to mempool
   │
   ├─► Zebra Internal Miner: Include in next block
   │   └── [30-60 seconds]
   │
   └─► Return TXID to user
```

### Shielded Send Flow

```
User Request
   │
   ├─► POST /send {address, amount, memo}
   │
   ▼
Faucet API Handler
   │
   ├─► wallet.get_balance()
   │   └── Check Orchard balance >= amount
   │
   ├─► wallet.send_transaction(address, amount, memo)
   │   │
   │   ├─► Zingolib: Select Orchard notes
   │   ├─► Zingolib: Create output note
   │   ├─► Zingolib: Encrypt memo
   │   ├─► Zingolib: Generate shielded proof
   │   ├─► Zingolib: Sign transaction
   │   └─► Zingolib: Broadcast via Zaino
   │
   ├─► Zaino: Forward to Zebra RPC
   │
   ├─► Zebra: Add to mempool
   │
   ├─► Zebra Internal Miner: Include in next block
   │   └── [30-60 seconds]
   │
   └─► Return TXID to user
```

---

## Network Configuration

### Docker Network

**Name:** zeckit-network  
**Type:** Bridge  
**Subnet:** Auto-assigned by Docker

### Port Mapping

| Service | Internal Port | Host Port | Protocol |
|---------|---------------|-----------|----------|
| Zebra RPC | 8232 | 127.0.0.1:8232 | HTTP |
| Zaino/LWD | 9067 | 127.0.0.1:9067 | gRPC |
| Faucet API | 8080 | 0.0.0.0:8080 | HTTP |

Note: Only Faucet API is exposed to LAN (0.0.0.0). Zebra and backend are localhost-only for security.

### Service Discovery

All services use Docker DNS for service discovery:

```
faucet
zaino → http://zebra:8232
lightwalletd → http://zebra:8232
```

---

## Storage Architecture

### Docker Volumes

```
zeckit_zebra-data/
├── state/
│   └── rocksdb/          # Blockchain database
│       ├── blocks/
│       ├── state/
│       └── finalized/

zeckit_zaino-data/
└── db/                   # Indexed compact blocks
    ├── blocks.db
    └── notes.db

zeckit_lightwalletd-data/
└── cache/                # Indexed data
    └── compact-blocks/

zeckit_faucet-data/
└── wallets/              # Wallet database
    ├── .wallet_seed      # Deterministic seed (24 words)
    └── zingo-wallet.dat  # Encrypted wallet state
```

### Volume Lifecycle

**Default Behavior:**
- Volumes persist across zeckit down
- Allows fast restarts (no re-sync needed)
- Wallet retains same addresses

**Fresh Start:**
```bash
zeckit down
docker volume rm zeckit_zebra-data zeckit_zaino-data zeckit_faucet-data
zeckit up --backend zaino
```

---

## Security Model

### Development Only Warning

ZecKit is NOT production-ready

**Security Limitations:**
- No authentication on any API
- No TLS/HTTPS encryption
- No rate limiting
- No secret management
- Regtest network only (not  ZEC)

### Isolation Boundaries

```
Internet
   ↕
Host Network
   ↕
┌─────────────────────────┐
│   Docker Bridge         │
│                         │
│  Zebra ←→ Zaino ←→ Faucet
│                         │
└─────────────────────────┘
```

**Exposed Ports:**
- Faucet API: 0.0.0.0:8080 (LAN accessible)
- Zebra RPC: 127.0.0.1:8232 (localhost only)
- Zaino/LWD: 127.0.0.1:9067 (localhost only)

---

## Concurrency Model

### Faucet Service

```
┌──────────────────────────────┐
│   Tokio Async Runtime        │
│                              │
│  ┌────────────────────────┐ │
│  │  HTTP Server           │ │
│  │  (Axum)                │ │
│  │  • Concurrent requests │ │
│  │  • Non-blocking I/O    │ │
│  └────────────────────────┘ │
│                              │
│  ┌────────────────────────┐ │
│  │  Background Tasks      │ │
│  │  • Wallet sync (60s)   │ │
│  │  • Health monitoring   │ │
│  └────────────────────────┘ │
│                              │
│  ┌────────────────────────┐ │
│  │  Shared State          │ │
│  │  Arc<RwLock<Wallet>>   │ │
│  │  • Read: Many threads  │ │
│  │  • Write: Exclusive    │ │
│  └────────────────────────┘ │
└──────────────────────────────┘
```

**Locking Strategy:**
- Reads (balance, address): Shared lock (multiple concurrent)
- Writes (send, shield, sync): Exclusive lock (one at a time)
- Background sync: Skips if lock unavailable (no blocking)

---

## Performance Characteristics

### Resource Usage

| Component | CPU (avg) | Memory | Disk I/O |
|-----------|-----------|--------|----------|
| Zebra | 20-50% | 500MB | Low |
| Zaino | 5-10% | 150MB | Medium |
| Lightwalletd | 5-10% | 200MB | Medium |
| Faucet | 2-5% | 100MB | Low |

**Total System:**
- CPU: 0.8-1.0 cores
- Memory: 750-850MB
- Disk: 2.5GB

### Timing Benchmarks

| Operation | Zaino | Lightwalletd |
|-----------|-------|--------------|
| Cold start | 2-3 min | 3-4 min |
| Warm restart | 30 sec | 30 sec |
| Shield tx | 8 sec | 8 sec |
| Shielded send | 5 sec | 6 sec |
| Block confirmation | 30-60 sec | 30-60 sec |
| Wallet sync | 3-5 sec | 5-8 sec |

---

## Design Decisions

### Why Embedded Wallet?

**Pros:**
- Simpler architecture (no external process)
- Better performance (no IPC overhead)
- Direct API access (no CLI parsing)
- Easier error handling

**Cons:**
- Library dependency (must update zingolib)
- Less flexibility (can't swap wallet implementations)

**Decision:** Pros outweigh cons for development use case

---

### Why Deterministic Seed?

**Pros:**
- Predictable addresses for testing
- Easy to reproduce issues
- Simpler documentation (hardcode example addresses)

**Cons:**
- Not suitable for production
- Users can't generate custom wallets

**Decision:** Perfect for development, clearly documented as dev-only

---

### Why Both Backends?

**Pros:**
- Tests compatibility with both implementations
- Developers can choose preferred backend
- Catches backend-specific bugs

**Cons:**
- More complex docker-compose
- Double the testing matrix

**Decision:** Worth it for ecosystem compatibility

---

## Failure Modes

### Network Partitions

**Scenario:** Zaino/LWD can't reach Zebra

**Symptoms:**
- Sync fails
- Balance shows 0
- Transactions fail

**Recovery:**
- Services auto-retry connection
- Manual: docker-compose restart faucet

---

### Wallet Desync

**Scenario:** Wallet thinks it's ahead of chain

**Symptoms:**
- "wallet height > chain height" error
- Balance incorrect

**Recovery:**
```bash
zeckit down
docker volume rm zeckit_faucet-data
zeckit up --backend zaino
```

---

### Mining Stalls

**Scenario:** Zebra stops mining blocks

**Symptoms:**
- Block count not increasing
- Transactions stuck in mempool

**Recovery:**
```bash
docker-compose restart zebra
```

---

## Future Architecture (M3+)

### Planned Enhancements

1. Pre-mined Snapshots:
   - Start with 1000+ pre-mined blocks
   - Faster startup (under 30 seconds)

2. GitHub Action Integration:
   - Reusable workflow
   - Automated testing in CI

3. Multi-Wallet Support:
   - Test wallet-to-wallet transfers
   - Simulate multi-user scenarios

4. Monitoring:
   - Prometheus metrics
   - Grafana dashboards
   - Alert on failures

---

## Appendix

### Container Dependencies

```
zebra
  ↓
zaino/lightwalletd (condition: service_healthy)
  ↓
faucet (condition: service_started)
```

Note: Faucet uses service_started not service_healthy to avoid blocking on slow sync

---

### Health Check Details

**Zebra:**
```yaml
test: ["CMD-SHELL", "timeout 5 bash -c 'cat < /dev/null > /dev/tcp/127.0.0.1/8232' || exit 1"]
interval: 30s
retries: 10
start_period: 120s
```

**Zaino:**
```yaml
test: ["CMD-SHELL", "timeout 5 bash -c 'cat < /dev/null > /dev/tcp/127.0.0.1/9067' || exit 1"]
interval: 10s
retries: 60
start_period: 180s
```

**Lightwalletd:**
```yaml
test: ["CMD-SHELL", "timeout 5 bash -c 'cat < /dev/null > /dev/tcp/127.0.0.1/9067' || exit 1"]
interval: 10s
retries: 30
start_period: 120s
```

---

### References

- [Zebra Architecture](https://zebra.zfnd.org/dev.html)
- [Zaino Documentation](https://github.com/zingolabs/zaino)
- [Zingolib Documentation](https://github.com/zingolabs/zingolib)
- [Zcash Protocol](https://zips.z.cash/protocol/protocol.pdf)
- [ZIP-316: Unified Addresses](https://zips.z.cash/zip-0316)

---

**Last Updated:** February 7, 2026  
**Version:** M2 ( Shielded Transactions)  
**Status:** Complete