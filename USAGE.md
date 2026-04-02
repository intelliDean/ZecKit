# 🛡️ ZecKit Integration Guide
This guide provides a step-by-step process for integrating the **ZecKit** developer toolkit into your Zcash projects.

---

### Step 1: Initialize your Project
If you haven't already, create your folder and initialize Git:
```bash
# Replace 'my-zcash-project' with your project name
mkdir my-zcash-project && cd my-zcash-project
git init
echo "# My Zcash Project" > README.md
git add README.md
git commit -m "initial commit"
```

### Step 2: Set up your GitHub Remote
If you have created a repository on GitHub, add it as a remote:
```bash
# Replace the URL with your own repo
git remote add origin https://github.com/USERNAME/REPO_NAME.git
```
> [!TIP]
> If you need to authenticate with a **Personal Access Token (PAT)**, use this format:
> `git remote set-url origin https://<TOKEN>@github.com/USERNAME/REPO_NAME.git`

### Step 3: Generate the ZecKit CI Workflow
Use the ZecKit CLI to generate the standardized GitHub Action configuration. If you don't have the binary, you can run it via Cargo:
```bash
# Run the ZecKit Generator (assuming ZecKit is a sibling or installed)
zeckit init --backend zaino
```
*   **What this does**: Creates `.github/workflows/zeckit-e2e.yml`.
*   **Verify**: Open the generated file and ensure `uses: intelliDean/ZecKit@main` is correct.

### Step 4: Create a Smoke Test Script
ZecKit spins up the environment, but you need to tell it what to test. Create a simple verification script:
```bash
cat > smoke_test.sh <<EOF
#!/bin/bash
set -e
echo "🔍 Checking Devnet Health..."
# Check if the ZecKit Faucet is reachable
curl -s http://127.0.0.1:8080/stats | grep -q "current_balance"
echo "✅ ZecKit Devnet is ALIVE!"
EOF

# Make it executable
chmod +x smoke_test.sh
```

### Step 5: Trigger your First CI Run
Stage your changes and push them to GitHub. The ZecKit Action will take over from here!
```bash
git add .
git commit -m "feat: first successful ZecKit CI integration"
git push -u origin main  # Or 'master'
```

---
### 🛠️ Troubleshooting
- **Pull Access Denied**: Ensure `image_prefix: 'ghcr.io/intellidean/zeckit'` is present in your YAML `with:` block.
- **Startup Timeout**: If Zebra is taking too long to sync, increase `startup_timeout_minutes` to `20`.

By following these steps, you will have a production-ready Zcash Devnet running on every commit! 🛡️✨
