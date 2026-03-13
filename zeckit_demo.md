# ZecKit Local Development & Verification Demo

This guide walks you through testing the **ZecKit** toolkit locally.

## Prerequisites

Ensure you have the ZecKit CLI built:

```bash
cd cli
cargo build --release
```

---

## Method 1: Local Application Development (Integrated)

The repository includes an `example-app/` directory. You can test your local `ZecKit` binary by running this app against it.

1.  **Navigate to the example app**:
    ```bash
    cd ../zeckit-sample-test/example-app
    ```

2.  **Run the application**:
    ```bash
    npm install
    npm start
    ```
    *This script connects to a running ZecKit devnet. Ensure you have run `zeckit up` in the background first.*

---

## Method 2: Seamless Dual-Linkage (For 'act' or Local Workflows)

This allows you to test the actual GitHub Actions YAML using your local code.

1.  **Activate Local Linkage**:
    ```bash
    ./link-local.sh
    ```
    *This creates a symlink to your local ZecKit project. The workflows are configured to detect and prioritize this link.*

2.  **Run with `act`**:
    ```bash
    act -W .github/workflows/ci.yml
    ```

3.  **Deactivate (Optional)**:
    If you want to revert to testing the remote repository version:
    ```bash
    rm .zeckit-action
    ```

---

## Method 3: Running the Example App Manually

If you want to iterate on the application code itself while the devnet is running:

1.  **Start the devnet** (in one terminal):
    ```bash
    ./test-local.sh zaino
    ```
    *Wait until you see "Starting E2E tests..."*

2.  **Run the app** (in a second terminal):
    ```bash
    cd example-app
    npm install   # Only needed once
    npm start
    ```

---

---

## Milestone 2 Verification: Shielded Transactions

Milestone 2 introduces the actual Zcash privacy engine. Verification requires using the CLI to drive the "Golden Flow" (Fund → Shield → Send).

### 1. The E2E "Golden Flow"
Prove that private Orchard transactions are functional on your local machine.

1.  **Ensure Devnet is running**:
    ```bash
    ./cli/target/release/zeckit up --backend zaino
    ```

2.  **Run the E2E Test Suite**:
    ```bash
    ./cli/target/release/zeckit test
    ```

3.  **Verify Success**:
    - You should see **`[5/7] Wallet balance and shield... PASS`**
    - You should see **`[6/7] Shielded send (E2E)... PASS`**
    - This confirms that ZecKit successfully mined coinbase rewards, auto-shielded them to the Orchard pool, and performed a private transaction.

### 2. Backend Interoperability
Verify that ZecKit works seamlessly with different privacy indexers.

1.  **Switch to Lightwalletd**:
    ```bash
    ./cli/target/release/zeckit down
    ./cli/target/release/zeckit up --backend lwd
    ```
2.  **Repeat the test**:
    ```bash
    ./cli/target/release/zeckit test
    ```
    - Both backends (Zaino and LWD) should pass the same E2E suite.

---

## Milestone 1 Verification: The Foundation

Milestone 1 focuses on the orchestration engine, health checks, and repository standards. Follow these steps to verify that the core ZecKit foundations are solid.

### 1. Local Orchestration & Health Checks
Prove that the CLI can spin up a healthy Zebra regtest cluster with one command.

1.  **Navigate to the CLI folder**:
    ```bash
    cd cli
    ```

2.  **Start the devnet**:
    ```bash
    cargo run -- up --backend zaino
    ```

3.  **Verify Success**:
    - The terminal should show readiness signals: `✓ Zebra Miner ready`, `✓ Zebra Sync node ready`, etc.
    - The command should finish with: `━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ZecKit Devnet ready  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━`

### 2. CI Smoke Test Validation
Verify that the repository includes a "fail-fast" smoke test to detect unhealthy clusters in CI.

1.  **Check GitHub Actions**: Look for the **Smoke Test** workflow in the ZecKit repository.
2.  **Logic**: This job verifies that all 3 nodes (Zebra, Faucet, Indexer) are reachable and report basic metadata in < 5 minutes.

### 3. Repository Standards Check
Ensure the repository meets the official Zcash community bootstrapping requirements.

- **Legal**: Check for `LICENSE-MIT` and `LICENSE-APACHE`.
- **Onboarding**: Verify `CONTRIBUTING.md` exists.
- **Support**: Check `.github/ISSUE_TEMPLATE/bug_report.md`.
- **Technical**: Review `specs/technical-spec.md` and `specs/acceptance-tests.md`.

---
- **Docker Errors**: Check that `docker compose` is installed and running (`docker compose version`).
