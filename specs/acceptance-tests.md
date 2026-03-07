# ZecKit M2 Acceptance Tests

## Overview

This document defines the acceptance criteria for Milestone 2:  Shielded Transactions.

---

## Test Environment

- **Platform:** Ubuntu 22.04 LTS, WSL2, or macOS
- **Docker:** Engine 24.x + Compose v2
- **Rust:** 1.70+
- **Resources:** 2 CPU, 4GB RAM, 5GB disk

---

## M2 Acceptance Criteria

### AC1: Start with Zaino Backend

**Test:** Start devnet with Zaino backend

```bash
zeckit up --backend zaino
```

Result:
- Zebra starts in regtest mode
- Zaino connects to Zebra
- Faucet starts and initializes wallet
- All services report healthy
- Auto-mining begins (blocks increasing)
- Startup time: under 3 minutes

Verification:
```bash
docker-compose ps
# All services show "running" or "healthy"

curl http://localhost:8080/health
# {"status":"healthy"}
```

---

### AC2: Start with Lightwalletd Backend

**Test:** Start devnet with Lightwalletd backend

```bash
zeckit down
zeckit up --backend lwd
```

Result:
- Zebra starts in regtest mode
- Lightwalletd connects to Zebra
- Faucet starts and initializes wallet
- All services report healthy
- Auto-mining begins
- Startup time: under 4 minutes

Verification:
```bash
docker-compose ps
# All services show "running" or "healthy"

curl http://localhost:8080/health
# {"status":"healthy"}
```

---

### AC3: Automated Test Suite Passes

**Test:** Run comprehensive smoke tests

```bash
zeckit test
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

Result:
- All 6 tests pass
- No errors thrown
- Test execution under 2 minutes

---

### AC4:  Shielded Transactions Work

**Test:** Execute  shielded send

```bash
# 1. Check balance (should have Orchard funds)
curl http://localhost:8080/stats

# Response:
# {
#   "orchard_balance": 500+,
#   ...
# }

# 2. Send shielded transaction
curl -X POST http://localhost:8080/send \
  -H "Content-Type: application/json" \
  -d '{
    "address": "uregtest1h8fnf3vrmswwj0r6nfvq24nxzmyjzaq5jvyxyc2afjtuze8tn93zjqt87kv9wm0ew4rkprpuphf08tc7f5nnd3j3kxnngyxf0cv9k9lc",
    "amount": 0.05,
    "memo": "Test payment"
  }'
```

Response:
```json
{
  "status": "sent",
  "txid": "a8a51e4ed52562ce...",
  "to_address": "uregtest1...",
  "amount": 0.05,
  "memo": "Test payment",
  "orchard_balance": 543.74,
  "timestamp": "2026-02-05T05:41:22Z"
}
```

Result:
- Returns valid 64-char hex TXID
- Status is "sent"
- Balance decreases appropriately
- Transaction is Orchard to Orchard (shielded)

---

### AC5: Shield Workflow Works

**Test:** Shield transparent funds to Orchard

```bash
# 1. Check transparent balance
curl http://localhost:8080/stats

# Response:
# {
#   "transparent_balance": 100+,
#   "orchard_balance": X,
#   ...
# }

# 2. Shield transparent funds
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
  "message": "Shielded 156.2499 ZEC from transparent to orchard..."
}
```

Result:
- Returns valid TXID
- Transparent balance decreases
- Orchard balance increases (after confirmation)
- Fee is correctly deducted

---

### AC6: Unified Address Generation

**Test:** Faucet generates and returns Unified Address

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

Result:
- Unified address starts with "uregtest1"
- Transparent address starts with "tm"
- Same address returned consistently
- Address is deterministic (same seed = same address)

---

### AC7: Backend Toggle Works

**Test:** Switch between backends without data loss

```bash
# Start with Zaino
zeckit up --backend zaino
sleep 60

# Check balance
BALANCE_ZAINO=$(curl -s http://localhost:8080/stats | jq .current_balance)

# Stop Zaino
zeckit down

# Start Lightwalletd
zeckit up --backend lwd
sleep 90

# Check balance again (should be same - data persisted)
BALANCE_LWD=$(curl -s http://localhost:8080/stats | jq .current_balance)

# Compare
echo "Zaino balance: $BALANCE_ZAINO"
echo "LWD balance: $BALANCE_LWD"
```

Result:
- Both backends start successfully
- Wallet data persists across backend switch
- Balance is approximately the same (plus or minus mining rewards)
- Tests pass with both backends

---

### AC8: Fresh Start Works

**Test:** Complete reset and restart

```bash
# Stop services
zeckit down

# Remove all data
docker volume rm zeckit_zebra-data zeckit_zaino-data zeckit_faucet-data

# Start fresh
zeckit up --backend zaino

# Wait for initialization
sleep 120

# Run tests
zeckit test
```

Result:
- Fresh blockchain mined from genesis
- New wallet created with deterministic seed
- Same addresses generated as before
- All services healthy
- All tests pass

---

### AC9: Service Health Checks

**Test:** All services have working health endpoints

```bash
# Zebra RPC
curl -X POST http://localhost:8232 \
  -d '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}'

# Faucet health
curl http://localhost:8080/health

# Faucet stats
curl http://localhost:8080/stats
```

Result:
- Zebra: Returns block height greater than 100
- Faucet health: {"status":"healthy"}
- Faucet stats: Shows positive balance
- All endpoints respond within 5 seconds
- No 500 errors

---

### AC10: Deterministic Behavior

**Test:** Same seed produces same addresses

```bash
# First run
zeckit up --backend zaino
sleep 60
ADDR_1=$(curl -s http://localhost:8080/address | jq -r .unified_address)

# Reset
zeckit down
docker volume rm zeckit_zebra-data zeckit_zaino-data zeckit_faucet-data

# Second run
zeckit up --backend zaino
sleep 60
ADDR_2=$(curl -s http://localhost:8080/address | jq -r .unified_address)

# Compare
echo "Address 1: $ADDR_1"
echo "Address 2: $ADDR_2"
```

Result:
- Both addresses are identical
- Transparent address also matches
- Seed file exists at /var/zingo/.wallet_seed

---

## Performance Benchmarks

| Metric | Target | Actual |
|--------|--------|--------|
| Startup time (Zaino) | under 3 min | 2-3 min |
| Startup time (LWD) | under 4 min | 3-4 min |
| Test execution | under 2 min | 60-90 sec |
| Shield transaction | under 15 sec | 8 sec |
| Shielded send | under 10 sec | 5 sec |
| Memory usage | under 4GB | 750-800MB |
| Disk usage | under 5GB | 2.35-2.55GB |

---

## Test Matrix

### Backend Compatibility

| Test | Zaino | Lightwalletd |
|------|-------|--------------|
| Start services | PASS | PASS |
| Health checks | PASS | PASS |
| Address generation | PASS | PASS |
| Wallet sync | PASS | PASS |
| Shield funds | PASS | PASS |
| Shielded send | PASS | PASS |
| All 6 smoke tests | PASS | PASS |

Both backends must pass all tests.

---

## Known Acceptable Behaviors

### 1. Timing Variations

**Behavior:** Test timing may vary by 10-20 seconds

**Cause:** Block mining is probabilistic

**Acceptable:** Yes - tests have flexible timeouts

---

### 2. Balance Fluctuations

**Behavior:** Balance may change between API calls during testing

**Cause:** Continuous mining + chain re-orgs in regtest

**Acceptable:** Yes - tests check for positive balance, not exact amounts

---

### 3. First Test Run After Startup

**Behavior:** Tests may fail if run immediately after zeckit up

**Cause:** Mining hasn't completed yet

**Acceptable:** Yes - wait 60 seconds then re-run

**Solution:**
```bash
zeckit up --backend zaino
sleep 60  # Wait for initial mining
zeckit test
```

---

### 4. Lightwalletd Slower Sync

**Behavior:** Lightwalletd tests take 10-20 seconds longer

**Cause:** Different indexing implementation

**Acceptable:** Yes - both backends work, just different speeds

---

## Regression Tests

### Test Case: Shield Then Send

```bash
# Start fresh
zeckit down
docker volume rm zeckit_zebra-data zeckit_zaino-data zeckit_faucet-data
zeckit up --backend zaino
sleep 90

# 1. Check transparent balance
curl http://localhost:8080/stats | jq .transparent_balance
# Should be greater than 0

# 2. Shield funds
curl -X POST http://localhost:8080/shield
# Should return success

# 3. Wait for confirmation
sleep 30

# 4. Sync wallet
curl -X POST http://localhost:8080/sync

# 5. Check Orchard balance
curl http://localhost:8080/stats | jq .orchard_balance
# Should be greater than 0

# 6. Send shielded
curl -X POST http://localhost:8080/send \
  -H "Content-Type: application/json" \
  -d '{
    "address": "uregtest1h8fnf3vrmswwj0r6nfvq24nxzmyjzaq5jvyxyc2afjtuze8tn93zjqt87kv9wm0ew4rkprpuphf08tc7f5nnd3j3kxnngyxf0cv9k9lc",
    "amount": 0.05,
    "memo": "Test"
  }'
# Should return valid TXID
```

Result: All steps succeed with valid responses

---

## Sign-Off Criteria

**Milestone 2 is considered complete when:**

1. Both backends (Zaino + Lightwalletd) start successfully
2. All 6 smoke tests pass with both backends
3.  shielded transactions work (Orchard to Orchard)
4. Shield workflow works (Transparent to Orchard)
5. Unified addresses generated correctly
6. Deterministic wallet behavior confirmed
7. Fresh start works without errors
8. Performance benchmarks met
9. Documentation complete
10. All test matrix cells pass

---

## M3 Future Tests

Coming in Milestone 3:

- GitHub Action integration
- Pre-mined blockchain snapshots
- Multi-recipient shielded sends
- Memo field edge cases
- Long-running stability tests
- Cross-platform testing (Linux, macOS, Windows WSL2)

---

**Status:** M2 Complete  
**Date:** February 7, 2026  
**All Acceptance Criteria:** PASSED