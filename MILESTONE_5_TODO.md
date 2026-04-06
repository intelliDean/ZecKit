# ZecKit Milestone 5 Roadmap (v1.2.0) 👑✨🛠️

Following the structural success of the M4 architectural and stability overhaul covering the unified Rust workspaces, cross-platform Apple Silicon targets, and dynamic automated dependency pipelines; our next focal intent pivots outward!

Milestone 5 aggressively transitions back into building feature-rich APIs strictly aimed at easing developer experiences, expanding testnet boundary capabilities, and integrating broader dynamic Zcash blockchain manipulations.

## 🛡️ Stabilization & Infrastructure (Active)

- [/] **CI Performance (Pull-Based Images)**: Transition `zaino` and `zeckit-faucet` E2E tests to pull pre-built images from GHCR instead of building from source, cutting CI time from 30m to <5m.
- [/] **Multi-Arch Docker Images (ARM64)**: Building native `linux/arm64` images to eliminate Rosetta dependency for Apple Silicon developers.

## 🚀 Future Feature Set (Milestone 5)

- [ ] **Multi-Wallet Testing Arrays**: Refactor the internal `WalletManager` orchestration limits allowing engineers to instantly spawn arrays of independently addressable transparent and shielded identities enabling cross-wallet functional checks.
- [ ] **Custom Chain Params Bootstrapping**: Capable of overriding block intervals and consensus branch IDs dynamically via command line arguments.
- [ ] **Data Volume Snapshot & Cloning**: Command array (`zeckit snapshot`) for instant blockchain state restoration avoiding lengthy local Sync lags.

---
**Status**: 🚀 v1.0.4 Released | CI Optimization In Progress ✨🕵️

