# ZecKit Milestone 5 Roadmap (v1.2.0) 👑✨🛠️

Following the structural success of the M4 architectural and stability overhaul covering the unified Rust workspaces, cross-platform Apple Silicon targets, and dynamic automated dependency pipelines; our next focal intent pivots outward!

Milestone 5 aggressively transitions back into building feature-rich APIs strictly aimed at easing developer experiences, expanding testnet boundary capabilities, and integrating broader dynamic Zcash blockchain manipulations.

## 🚀 Future Feature Set (Milestone 5)

- [ ] **Multi-Wallet Testing Arrays**: Refactor the internal `WalletManager` orchestration limits allowing engineers to instantly spawn arrays of independently addressable transparent and shielded identities enabling cross-wallet functional checks (Alice -> Bob -> Charlie).
- [ ] **Custom Chain Params Bootstrapping**: Remove hard-coded `regtest` limitations, granting CI users the explicit capability to override block intervals, consensus branch IDs, and ZIP activation heights dynamically via command line arguments (`zeckit up --custom-params /path`).
- [ ] **Data Volume Snapshot & Cloning**: Establish an integrated command array (`zeckit snapshot`) caching the state block data from the Zebra miner seamlessly pushing configurations towards standard registries, empowering teammates to clone heavily indexed blockchains instantly avoiding lengthy local Sync lags.

---
**Status**: DRAFT (Created during 1.1.0 Feature Expansion) 🚀🛡️✨
