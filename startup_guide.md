# ZecKit Devnet Startup Guide

This guide describes how to manage your local ZecKit Devnet.

## Quick Start
To start the devnet with the Zaino backend (recommended):
```bash
./cli/target/release/zeckit up --backend zaino
```

## Service Status
Verify the health of the devnet using the following endpoints:

- **Zebra Miner RPC**: `http://localhost:8232`
- **Faucet API**: `http://localhost:8080`
- **Zaino Indexer**: `http://localhost:9067`

### Checking Faucet Balance
```bash
curl http://localhost:8080/stats
```

### Checking Block Height
```bash
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | jq .result
```

## Stopping Devnet
To stop the devnet and all associated containers:
```bash
./cli/target/release/zeckit down
```

## Running Tests
To run the automated smoke test suite:
```bash
./cli/target/release/zeckit test
```
