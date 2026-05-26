# ZecKit

> A toolkit for Zcash Regtest development

[![E2E Tests](https://github.com/Zecdev/ZecKit/actions/workflows/e2e-test.yml/badge.svg)](https://github.com/Zecdev/ZecKit/actions/workflows/e2e-test.yml)
[![Smoke Test](https://github.com/Zecdev/ZecKit/actions/workflows/smoke-test.yml/badge.svg)](https://github.com/Zecdev/ZecKit/actions/workflows/smoke-test.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

---

## Project Status

**Current Milestone:** M5 Complete — Multi-Wallet Testing Arrays ✅ | M6 In Progress — 90-Day Maintenance ⏳

- **Network Support:** Full compatibility with **NU6** and **NU6.1** Zcash network upgrades.
- **Wallet Engine:** Upgraded to **Zingolib v3.0.0**.
- **Devnet Features:** 2-node Zebra cluster via GitHub Action or local CLI.

**M1 - Foundation**

- Zebra regtest node in Docker
- Health check automation
- Basic smoke tests
- Project structure and documentation

**M2 - Shielded Transactions**

- zeckit CLI tool with automated setup
- On-chain shielded transactions via ZingoLib
- Faucet API with actual blockchain broadcasting
- Backend toggle (lightwalletd or Zaino)
- Automated mining with coinbase maturity
- Unified Address (ZIP-316) support
- Shield transparent funds to Orchard
- Shielded send (Orchard to Orchard)

**M3 - GitHub Action** ✅

- Reusable GitHub Action for CI (E2E Tests + Smoke Tests)
- Two-node Zebra Regtest cluster (miner + sync)
- Full E2E golden flow: fund → shield → shielded send verified on-chain
- 8-test smoke suite passing in CI
- Artifact upload on failure for easy triage
- Continuous block mining (1 block / 15s) during tests

**M4 - Docs & Quickstarts**

- "2-minute local start" guide
- "5-line CI setup" snippet for other repos
- Compatibility matrix (Zebra / Zaino versions)
- Demo video

**M5 - Multi-Wallet Testing Arrays** ✅

- Refactored Faucet WalletManager to support spawning and managing multiple dynamic wallets
- Deterministic seed derivation from wallet ID string using SHA256 and custom salt
- Dynamic wallet endpoints under `/wallets` (create, list, address, stats, sync, shield, send)
- Multi-wallet E2E testing array (Test 7: alice -> bob shielded send flow)
- Background sync loop to iterate all dynamic wallets

**M6 - 90-Day Maintenance** ⏳

**User Stories:**
- As an adopter, I want timely version bumps so my CI stays green as Zebra/backends update.
- As a contributor, I want responsive triage so issues get unblocked.

**Deliverables:**
- 90-day maintenance: version pin updates (Zebra/backends), small fixes, monthly status notes.

**Acceptance Criteria:**
- Matrix CI remains green on current Zebra/backends; at least two monthly status notes posted; in-scope bugs are triaged with fixes or documented workarounds.
---

## Quick Start

### Option A: Rapid CI Integration (Zero Install)
The fastest way to use ZecKit if you just want to verify your own application's Zcash privacy logic in GitHub Actions.

1.  **Initialize**: Run the following in your CLI (no install needed if you have Rust):
    ```bash
    cargo run --package zeckit -- init --backend zaino
    ```
2.  **Commit**: Push the generated `.github/workflows/zeckit-e2e.yml` to your repo.
3.  **Done**: GitHub will now spin up a full Zcash devnet on every PR and verify your logic.

For a detailed step-by-step tutorial, see the **[Setup Guide](USAGE.md)**.

---

### Option B: Local Standalone Development
Use this if you want to develop and debug your application manually on your laptop.

### Prerequisites

- **OS:** Linux (Ubuntu 22.04+), WSL2, or macOS with Docker Desktop 4.34+
- **Docker:** Engine 24.x + Compose v2
- **Rust:** 1.70+ (for building CLI)
- **Resources:** 2 CPU cores, 4GB RAM, 5GB disk
- **GitHub Actions Runner:** A `self-hosted` runner is required for executing the ZecKit `smoke-test` CI pipeline (more details below).

## Architecture: How ZecKit Works
Many developers assume ZecKit is strictly a GitHub Action. **It is not.**
ZecKit is deeply composed of three layers:
1. **The Regtest Cluster:** A completely containerized Docker Compose environment running an isolated Zcash blockchain (Zebra), an indexing backend (Zaino or lightwalletd), and a custom Faucet for funding.
2. **The Rust CLI:** The `zeckit up` and `zeckit test` commands orchestrate the heavy lifting: pinging health checks, dynamically driving the background miner, extracting state, and executing golden-flow tests.
3. **The GitHub Action:** A thin wrapper (`action.yml`) that simply downloads the CLI and runs it inside your CI pipeline to seamlessly verify your own downstream applications against a disposable Regtest node.

**You can run ZecKit identically on your local laptop as it runs in the cloud.** Check out the [integrated application](https://github.com/intelliDean/zeckit-sample-test/tree/main/example-app) in the sample repository for a tutorial on how a standard Node.js Web3 application interacts with the local Regtest devnet.

### Action Runner Setup

For the repository's native CI workflows (like the Zebra Smoke Test) to execute successfully without timing out, a [self-hosted GitHub Action Runner](https://docs.github.com/en/actions/hosting-your-own-runners/managing-self-hosted-runners/adding-self-hosted-runners) MUST be configured and actively running on a machine that meets the prerequisites above.

1. Navigate to your repository settings on GitHub (`Settings > Actions > Runners`).
2. Click **New self-hosted runner**.
3. Follow the provided instructions to download, configure, and execute the `./run.sh` daemon on your local workstation or VPS.
4. Ensure the runner is tagged as `self-hosted`.

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

### How to Start Local Devnet (Quick Reference)

For detailed instructions and service health checks, see the [Startup Guide](startup_guide.md).

1.  **Build the CLI**: `cd cli && cargo build --release && cd ..`
2.  **Launch the Network**: `./cli/target/release/zeckit up --backend zaino`
3.  **Check Health**: `curl http://localhost:8080/stats`
4.  **Stop**: `./cli/target/release/zeckit down`

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

### Auto-Initialize CI Workflow
Generate a professional GitHub Actions E2E suite for your own repository in one command.

This command will automatically detect your project structure and drop a complete `.github/workflows/zeckit-e2e.yml` file into your repository. This file is pre-configured to spin up a Zeckit Regtest node and run your project's tests against it!

```bash
# Default (Zaino backend)
./cli/target/release/zeckit init

# Custom backend and output path
./cli/target/release/zeckit init --backend lwd --output .github/workflows/custom-test.yml
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

  [0/8] Cluster synchronization... WARN (non-fatal) Sync node lagging: Miner=217 Sync=0
  [1/8] Zebra RPC connectivity (Miner)... PASS
  [2/8] Faucet health check... PASS
  [3/8] Faucet address retrieval... PASS
  [4/8] Wallet sync capability... PASS
  [5/8] Wallet balance and shield... PASS
  [6/8] Shielded send (E2E)... PASS
  [7/8] Multi-wallet array (alice -> bob)... PASS

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Tests passed: 8
  Tests failed: 0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ All smoke tests PASSED!
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

The `zeckit test` command runs 8 tests:

| Test | What It Validates |
| ---- | ----------------- |
| 0. Cluster Sync | Sync node height vs miner (warn-only) |
| 1. Zebra RPC | Miner node RPC is live |
| 2. Faucet Health | Faucet service is healthy |
| 3. Address Retrieval | Can get unified + transparent addresses |
| 4. Wallet Sync | Wallet can sync with blockchain |
| 5. Shield Funds | Transparent → Orchard shielding works |
| 6. Shielded Send | E2E golden flow: Orchard → Orchard |
| 7. Multi-wallet Array | E2E flow between two dynamic wallets (alice → bob) |

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

#### POST /wallets

Spawns a new dynamic wallet with `wallet_id`

```bash
curl -X POST http://localhost:8080/wallets \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_id": "alice"
  }'
```

Response:

```json
{
  "wallet_id": "alice",
  "status": "created"
}
```

#### GET /wallets

Lists all currently loaded wallets

```bash
curl http://localhost:8080/wallets
```

Response:

```json
{
  "wallets": [
    "default",
    "alice",
    "bob"
  ]
}
```

#### GET /wallets/:id/address

Get addresses for a specific dynamic wallet

```bash
curl http://localhost:8080/wallets/alice/address
```

Response:

```json
{
  "wallet_id": "alice",
  "unified_address": "uregtest1...",
  "transparent_address": "tmBs..."
}
```

#### GET /wallets/:id/stats

Get balance and stats for a specific dynamic wallet

```bash
curl http://localhost:8080/wallets/alice/stats
```

Response:

```json
{
  "wallet_id": "alice",
  "current_balance": 0.1,
  "orchard_balance": 0.0,
  "transparent_balance": 0.1,
  "total_requests": 0,
  "total_sent": 0.0
}
```

#### POST /wallets/:id/sync

Sync a specific dynamic wallet with blockchain

```bash
curl -X POST http://localhost:8080/wallets/alice/sync
```

Response:

```json
{
  "wallet_id": "alice",
  "status": "synced",
  "message": "Wallet alice synced with blockchain"
}
```

#### POST /wallets/:id/shield

Shield transparent funds to Orchard pool for a specific dynamic wallet

```bash
curl -X POST http://localhost:8080/wallets/alice/shield
```

Response:

```json
{
  "wallet_id": "alice",
  "status": "shielded",
  "txid": "...",
  "transparent_amount": 0.1,
  "shielded_amount": 0.0999,
  "fee": 0.0001,
  "message": "Shielded 0.0999 ZEC from transparent to orchard (fee: 0.0001 ZEC)"
}
```

#### POST /wallets/:id/send

Send shielded transaction from a specific dynamic wallet

```bash
curl -X POST http://localhost:8080/wallets/alice/send \
  -H "Content-Type: application/json" \
  -d '{
    "address": "uregtest1...",
    "amount": 0.04,
    "memo": "from alice"
  }'
```

Response:

```json
{
  "wallet_id": "alice",
  "status": "sent",
  "txid": "...",
  "to_address": "uregtest1...",
  "amount": 0.04,
  "memo": "from alice",
  "new_balance": 0.0598,
  "orchard_balance": 0.0598,
  "timestamp": "2026-05-26T13:48:00Z",
  "message": "Sent 0.04 ZEC from Orchard pool"
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

**M3 - GitHub Action** ✅ (Complete)

- Reusable CI action running on every push
- E2E golden flow verified in CI
- Full 7-test smoke suite

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
4. Ensure all 7 tests pass (test 0 is warn-only)
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

**Last Updated:** May 26, 2026
**Status:** **M5 Multi-Wallet Testing Arrays Complete** — CI passing (8/8 tests) ✅
- **Protocol:** Upgraded to Zingolib v3.0.0 for official NU6 support.
- **Performance:** Optimized image pulling (2-4 min CI setup).
