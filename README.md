# ZecKit

> A toolkit for Zcash Regtest development

---

## Project Status

**Current Milestone:** M2 Complete - Shielded Transactions

### What Works Now

**M1 - Foundation**

- Zebra regtest node in Docker
- Health check automation
- Basic smoke tests
- Project structure and documentation

**M2 - Shielded Transactions**

- zeckit CLI tool with automated setup
- on-chain shielded transactions via ZingoLib
- Faucet API with actual blockchain broadcasting
- Backend toggle (lightwalletd or Zaino)
- Automated mining with coinbase maturity
- Unified Address (ZIP-316) support
- Shield transparent funds to Orchard
- Shielded send (Orchard to Orchard)
- Comprehensive test suite (6 tests)

**M3 - GitHub Action (Next)**

- Reusable GitHub Action for CI
- Pre-mined blockchain snapshots
- Advanced shielded workflows

---

## Quick Start

### Prerequisites

- **OS:** Linux (Ubuntu 22.04+), WSL2, or macOS with Docker Desktop 4.34+
- **Docker:** Engine 24.x + Compose v2
- **Rust:** 1.70+ (for building CLI)
- **Resources:** 2 CPU cores, 4GB RAM, 5GB disk

### Installation

```bash
# Clone repository
git clone https://github.com/Zecdev/ZecKit.git
cd ZecKit

# Build CLI (one time)
cd cli
cargo build --release
cd ..

# Start devnet with Zaino (recommended - faster)
./cli/target/release/zeckit up --backend zaino

# OR start with lightwalletd
./cli/target/release/zeckit up --backend lwd

# Wait for services to be ready (2-3 minutes)
# Zebra will auto-mine blocks in the background

# Run test suite
./cli/target/release/zeckit test
```

### Verify It's Working

```bash
# Check faucet has funds
curl http://localhost:8080/stats

# Response:
# {
#   "current_balance": 600+,
#   "transparent_balance": 100+,
#   "orchard_balance": 500+,
#   "faucet_address": "uregtest1...",
#   ...
# }

# Test shielded send
curl -X POST http://localhost:8080/send \
  -H "Content-Type: application/json" \
  -d '{
    "address": "uregtest1h8fnf3vrmswwj0r6nfvq24nxzmyjzaq5jvyxyc2afjtuze8tn93zjqt87kv9wm0ew4rkprpuphf08tc7f5nnd3j3kxnngyxf0cv9k9lc",
    "amount": 0.05,
    "memo": "Test transaction"
  }'

# Returns TXID from blockchain
```

---

## CLI Usage

### Start Services

```bash
# With Zaino (recommended - faster sync)
./cli/target/release/zeckit up --backend zaino

# With Lightwalletd
./cli/target/release/zeckit up --backend lwd
```

What happens:

1. Zebra starts in regtest mode with auto-mining
2. Backend (Zaino or Lightwalletd) connects to Zebra
3. Faucet wallet initializes with deterministic seed
4. Blocks are mined automatically, faucet receives coinbase rewards
5. Faucet auto-shields transparent funds to Orchard pool
6. Ready for shielded transactions

First startup: Takes 2-3 minutes for initial sync  
Subsequent startups: About 30 seconds (uses existing data)

### Stop Services

```bash
./cli/target/release/zeckit down
```

### Run Test Suite

```bash
./cli/target/release/zeckit test
```

Output:

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

### Switch Backends

```bash
# Stop current backend
./cli/target/release/zeckit down

# Start different backend
./cli/target/release/zeckit up --backend lwd

# Or back to Zaino
./cli/target/release/zeckit up --backend zaino
```

### Fresh Start

```bash
# Stop services
./cli/target/release/zeckit down

# Remove all data
docker volume rm zeckit_zebra-data zeckit_zaino-data zeckit_faucet-data

# Start fresh
./cli/target/release/zeckit up --backend zaino
```

---

## Test Suite

### Automated Tests

The `zeckit test` command runs 6 comprehensive tests:

| Test                 | What It Validates                         |
| -------------------- | ----------------------------------------- |
| 1. Zebra RPC         | Zebra node is running and RPC responds    |
| 2. Faucet Health     | Faucet service is healthy                 |
| 3. Address Retrieval | Can get unified and transparent addresses |
| 4. Wallet Sync       | Wallet can sync with blockchain           |
| 5. Shield Funds      | Can shield transparent to Orchard         |
| 6. Shielded Send     | E2E golden flow: Orchard to Orchard       |

Tests 5 and 6 prove shielded transactions work.

### Manual Testing

```bash
# Check service health
curl http://localhost:8080/health

# Get wallet addresses
curl http://localhost:8080/address

# Check balance
curl http://localhost:8080/stats

# Sync wallet
curl -X POST http://localhost:8080/sync

# Shield transparent funds to Orchard
curl -X POST http://localhost:8080/shield

# Send shielded transaction
curl -X POST http://localhost:8080/send \
  -H "Content-Type: application/json" \
  -d '{
    "address": "uregtest1...",
    "amount": 0.05,
    "memo": "Test payment"
  }'
```

---

## Faucet API

### Base URL

```
http://localhost:8080
```

### Endpoints

#### GET /health

Check service health

```bash
curl http://localhost:8080/health
```

Response:

```json
{
  "status": "healthy"
}
```

#### GET /stats

Get wallet statistics

```bash
curl http://localhost:8080/stats
```

Response:

```json
{
  "current_balance": 681.24,
  "transparent_balance": 125.0,
  "orchard_balance": 556.24,
  "faucet_address": "uregtest1h8fnf3vrmsw...",
  "network": "regtest",
  "wallet_backend": "zingolib",
  "version": "0.3.0",
  "total_requests": 5,
  "total_sent": 0.25,
  "uptime_seconds": 1234
}
```

#### GET /address

Get faucet addresses

```bash
curl http://localhost:8080/address
```

Response:

```json
{
  "unified_address": "uregtest1h8fnf3vrmswwj0r6nfvq24nxzmyjzaq5jvyxyc2afjtuze8tn93zjqt87kv9wm0ew4rkprpuphf08tc7f5nnd3j3kxnngyxf0cv9k9lc",
  "transparent_address": "tmBsTi2xWTjUdEXnuTceL7fecEQKeWaPDJd"
}
```

#### POST /sync

Sync wallet with blockchain

```bash
curl -X POST http://localhost:8080/sync
```

Response:

```json
{
  "status": "synced",
  "message": "Wallet synced with blockchain"
}
```

#### POST /shield

Shield transparent funds to Orchard pool

```bash
curl -X POST http://localhost:8080/shield
```

Response:

```json
{
  "status": "shielded",
  "txid": "86217a05f36ee5a7...",
  "transparent_amount": 156.25,
  "shielded_amount": 156.2499,
  "fee": 0.0001,
  "message": "Shielded 156.2499 ZEC from transparent to orchard (fee: 0.0001 ZEC)"
}
```

#### POST /send

Send shielded transaction (Orchard to Orchard)

```bash
curl -X POST http://localhost:8080/send \
  -H "Content-Type: application/json" \
  -d '{
    "address": "uregtest1...",
    "amount": 0.05,
    "memo": "Payment for services"
  }'
```

Response:

```json
{
  "status": "sent",
  "txid": "a8a51e4ed52562ce...",
  "to_address": "uregtest1...",
  "amount": 0.05,
  "memo": "Payment for services",
  "new_balance": 543.74,
  "orchard_balance": 543.74,
  "timestamp": "2026-02-05T05:41:22Z",
  "message": "Sent 0.05 ZEC from Orchard pool"
}
```

---

## Architecture

See [specs/architecture.md](specs/architecture.md) for detailed system architecture, component interactions, and data flows.

---

## What Makes This Different

### Shielded Transactions

Unlike other Zcash dev tools that only do transparent transactions, ZecKit supports:

- Unified Addresses (ZIP-316) - Modern address format
- Orchard Pool - Latest shielded pool (NU5+)
- Auto-shielding - Transparent to Orchard conversion
- Shielded sends - True private transactions
- Memo support - Encrypted messages

### Backend Flexibility

Toggle between two light client backends:

- **Zaino** (Rust) - Faster, better error messages
- **Lightwalletd** (Go) - Traditional, widely used

Both work with the same wallet and faucet.

### Deterministic Wallet

- Same seed across restarts
- Predictable addresses for testing
- No manual configuration needed

---

## Troubleshooting

### Common Issues

**Tests failing after restart**

```bash
# Wait for auto-mining to complete
sleep 60

# Run tests again
./cli/target/release/zeckit test
```

**Insufficient balance errors**

```bash
# Check if mining is happening
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | jq .result

# Should be increasing every 30-60 seconds
```

**Need fresh start**

```bash
./cli/target/release/zeckit down
docker volume rm zeckit_zebra-data zeckit_zaino-data zeckit_faucet-data
./cli/target/release/zeckit up --backend zaino
```

### Verify Mining

```bash
# Check block count (should increase)
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | jq .result

# Check mempool
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getrawmempool","params":[]}' | jq
```

---

## Project Goals

### Why ZecKit?

Zcash ecosystem needs a standard way to:

1. Test shielded transactions locally - Most tools only support transparent
2. Support modern addresses (UAs) - ZIP-316 unified addresses
3. Toggle backends easily - Compare lightwalletd vs Zaino
4. Catch breakage early - Automated E2E tests in CI

### Roadmap

**M1 - Foundation** (Complete)

- Zebra regtest setup
- Basic health checks
- Docker orchestration

**M2 - Transactions** (Complete)

- Shielded transaction support
- Unified addresses
- Auto-shielding workflow
- Backend toggle
- Comprehensive tests

**M3 - GitHub Action** (Next)

- Reusable CI action
- Pre-mined snapshots
- Advanced workflows

---

## Technical Details

### Wallet Implementation

- **Library:** Zingolib (Rust)
- **Address Type:** Unified (Orchard + Transparent)
- **Seed:** Deterministic (same across restarts)
- **Storage:** /var/zingo (persisted in Docker volume)

### Mining

- **Miner:** Zebra internal miner
- **Block Time:** 30-60 seconds
- **Coinbase:** Goes to faucet's transparent address
- **Auto-shield:** Faucet automatically shields to Orchard

### Network

- **Mode:** Regtest (isolated test network)
- **Ports:**
  - 8232: Zebra RPC
  - 8080: Faucet API
  - 9067: Backend (Zaino/LWD)

---

## Contributing

Contributions welcome. Please:

1. Fork and create feature branch
2. Test locally with both backends
3. Run: `./cli/target/release/zeckit test`
4. Ensure all 6 tests pass
5. Open PR with clear description

---

## Support

- **Issues:** [GitHub Issues](https://github.com/Zecdev/ZecKit/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Zecdev/ZecKit/discussions)
- **Community:** [Zcash Forum](https://forum.zcashcommunity.com/)

---

## License

Dual-licensed under MIT OR Apache-2.0

---

## Acknowledgments

**Built by:** Dapps over Apps team

**Thanks to:**

- Zcash Foundation (Zebra)
- Electric Coin Company (Lightwalletd)
- Zingo Labs (Zingolib and Zaino)
- Zcash community

---

**Last Updated:** February 5, 2026  
**Status:** M2 Complete - Shielded Transactions
