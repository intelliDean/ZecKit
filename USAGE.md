# 🛡️ ZecKit Usage Guide

ZecKit is a developer toolkit designed to provide a standardized, high-performance Zcash development environment (Zebra-based) for both local development and CI/CD.

---

## 🚀 Local Development CLI

Before integrating with CI, you can use the **ZecKit CLI** to manage your local devnet.

### 1. Installation
To use the `zeckit` command globally, install it from the repository root:
```bash
cd ZecKit/cli
cargo install --path .
```

### 2. Core Commands

#### Start the Devnet (`up`)
Launch a 2-node Zebra cluster with an embedded shielded faucet:
```bash
zeckit up --backend zaino
```
*   **Options**:
    *   `-b, --backend <zaino|lwd>`: Choose your light-client backend.
    *   `-f, --fresh`: Wipes previous blockchain data for a clean start.
    *   `--fund-address <ADDR>`: Automatically sends ZEC to an address once live.

#### Check Status (`status`)
Verify if the nodes, backend, and faucet are healthy and synced:
```bash
zeckit status
```

#### Run Smoke Tests (`test`)
Execute a standard end-to-end shielded transaction test to verify the network:
```bash
zeckit test --amount 0.05
```

#### Stop the Devnet (`down`)
Safely shut down all containers:
```bash
zeckit down
```
*   **Clean Slate**: Use `zeckit down --purge` to delete all Docker volumes.

---

## 🛡️ CI/CD Integration Guide

Follow these steps to integrate ZecKit into your Zcash project's GitHub Actions.

### Step 1: Initialize your Project
If you haven't already, ensure your project is a Git repository:
```bash
mkdir my-zcash-project && cd my-zcash-project
git init
echo "# My Zcash Project" > README.md
git add README.md
git commit -m "initial commit"
```

### Step 2: Set up your GitHub Remote
```bash
git remote add origin https://github.com/USERNAME/REPO_NAME.git
```
> [!TIP]
> To authenticate with a **Personal Access Token (PAT)**:
> `git remote set-url origin https://<TOKEN>@github.com/USERNAME/REPO_NAME.git`

### Step 3: Generate the CI Workflow
Run the `init` command from your project directory to create the GitHub Actions configuration:
```bash
zeckit init --backend zaino
```
*   **What this does**: Creates `.github/workflows/zeckit-e2e.yml`.
*   **Verify**: Ensure the generated file points to `uses: intelliDean/ZecKit@main`.

### Step 4: Create a Smoke Test Script
ZecKit spins up the environment in CI, but you need to tell it what to test. Create `smoke_test.sh`:
```bash
cat > smoke_test.sh <<EOF
#!/bin/bash
set -e
echo "🔍 Checking Devnet Health..."
curl -s http://127.0.0.1:8080/stats | grep -q "current_balance"
echo "✅ ZecKit Devnet is ALIVE!"
EOF

chmod +x smoke_test.sh
```

### Step 5: Trigger your First CI Run
```bash
git add .
git commit -m "feat: first successful ZecKit CI integration"
git push -u origin main
```

---

### 🛠️ Troubleshooting
- **Pull Access Denied**: Ensure `image_prefix: 'ghcr.io/intellidean/zeckit'` is present in your YAML configuration.
- **Startup Timeout**: If Zebra takes too long to sync on CI workers, increase `startup_timeout_minutes` to `20`.

By following these steps, you will have a production-ready Zcash Devnet running on every commit! 🛡️✨
