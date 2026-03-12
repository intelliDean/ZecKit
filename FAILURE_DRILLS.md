# ZecKit Failure Drills Guide

Failure Drills are designed to prove that your downstream CI handles edge cases (like out-of-funds or timeouts) gracefully. Instead of a standard "Happy Path" test, Failure Drills intentionally break the Devnet to verify that diagnostic artifacts are collected and the pipeline behaves predictably.

## Available Configuration Parameters

When using the `intelliDean/ZecKit` Action to configure a Failure Drill, you can override several parameters to trigger specific failure conditions.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `backend` | `string` | `"zaino"` | The indexing backend to use (`"zaino"`, `"lightwalletd"`, or `"none"`). |
| `startup_timeout_minutes` | `string` | `"10"` | How long to wait for the devnet to report healthy status. Set to `"1"` to trigger a timeout drill. |
| `send_amount` | `string` | `"0.5"` | The amount of ZEC to send in the E2E Golden Flow test. Set to `"999.0"` to trigger an insufficient funds overflow drill. |
| `block_wait_seconds` | `string` | `"75"` | Time to wait for blockchain propagation and syncing after mining starts. Lowering it can trigger sync timeouts. |
| `upload_artifacts` | `string` | `"on-failure"` | To ensure logs are always captured during drills, set this to `"always"`. |

## How to Add a New Failure Drill

You can add Failure Drills inside your own repository's `.github/workflows/failure-drill.yml` file.

Below is a complete template showcasing two common failure drills: "Startup Timeout" and "Send Amount Overflow".

### Example Failure Drill Workflow Template

```yaml
name: Failure Drill Verification

on: [workflow_dispatch, push]

jobs:
  # Example Drill 1: Purposefully Time Out Devnet Startup
  drill-timeout:
    runs-on: ubuntu-latest
    steps:
      - name: ZecKit Action - Force Timeout
        id: zeckit
        uses: intelliDean/ZecKit@main
        with:
          backend: zaino
          startup_timeout_minutes: '1' # Extremely short timeout
          upload_artifacts: always
        # The drill WILL fail, so we allow it to continue to assert the failure.
        continue-on-error: true

      - name: Assert Failure correctly captured
        run: |
          if [[ "${{ steps.zeckit.outputs.test_result }}" == "pass" ]]; then
            echo "::error::Drill failed: Expected a timeout failure, but got a pass!"
            exit 1
          fi
          echo "Drill successfully produced an expected timeout error."

  # Example Drill 2: Overflow Send Amount
  drill-insufficient-funds:
    runs-on: ubuntu-latest
    steps:
      - name: ZecKit Action - Force Overflow
        id: zeckit
        uses: intelliDean/ZecKit@main
        with:
          backend: lightwalletd
          send_amount: '9999.0' # Amount larger than the faucet holds
          upload_artifacts: always
        continue-on-error: true

      - name: Assert Failure correctly captured
        run: |
          if [[ "${{ steps.zeckit.outputs.test_result }}" == "pass" ]]; then
            echo "::error::Drill failed: Expected an insufficient funds failure, but got a pass!"
            exit 1
          fi
          echo "Drill successfully caught the overflow exception."
```

## Validating Output Artifacts

Because the Action was provided `upload_artifacts: always`, it will upload a ZIP folder containing `.log` files (e.g., `zebra.log`, `lightwalletd.log`, `containers.log`) for every drill. You can download and parse these logs automatically via the GitHub CLI (`gh run download`) as a final verification step in your CI!
