# ZecKit Milestone 4 Roadmap (v1.1.0) 🏗️🚀🛡️

This roadmap tracks the technical debt, optimizations, and improvements identified during the stabilization of the 1.0.0-alpha.4 launch. These items will be the focus of the next major iteration.

## 🛠️ Infrastructure Improvements (Milestone 4)
- [ ] **Root Rust Workspace**: Refactor the separate `/cli` and `/zeckit-faucet` directories into a single root-level workspace to unify dependency management.
- [ ] **Strict Dependency Locking**: Re-enable the `--locked` flag in CI once the workspace architecture is unified to prevent dependency drift.
- [ ] **ARM64 (Apple Silicon) Binaries**: Add `aarch64-apple-darwin` to the release matrix in `release.yml` to provide high-speed CI support for modern Mac developers.
- [ ] **Automated Version Bumping**: Implement a tool or workflow to periodically check and update the pinned versions of `zingolib` and Zebra to keep ZecKit current with the Zcash protocol.

## 🧪 Testing & Validation
- [x] **Granular Health Checks**: Add a `--check` flag to the ZecKit CLI to validate the environment, including Docker health and network connectivity, before running tests.
- [x] **Unit Testing Suite**: Increase test coverage for the internal logic of the CLI (Rust) beyond the existing E2E smoke tests.
- [x] **Platform Parity Tests**: Expand CI to run E2E tests on multiple architectures and OS versions to ensure binary compatibility.

## 📝 Documentation & UX
- [ ] **Troubleshooting Guide**: Add a section to `USAGE.md` specifically for ARM64/Mac users explaining the source-build fallback.
- [ ] **Milestone 4 Feature Set**: Define the next set of user-facing features for ZecKit (e.g., multi-wallet testing, custom chain-params support).

---
**Status**: DRAFT (Created during 1.0.0-alpha.4 Stabilization) 🚀🛡️✨
