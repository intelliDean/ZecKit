    # Contributing to ZecKit

    Thank you for your interest in contributing to ZecKit! This document provides guidelines and instructions for contributing.

    ## Table of Contents

    - [Code of Conduct](#code-of-conduct)
    - [Getting Started](#getting-started)
    - [Development Workflow](#development-workflow)
    - [Coding Standards](#coding-standards)
    - [Testing](#testing)
    - [Submitting Changes](#submitting-changes)
    - [Milestone Roadmap](#milestone-roadmap)

    ---

    ## Code of Conduct

    This project follows the [Zcash Community Code of Conduct](https://zcashcommunitygrants.org/code-of-conduct/).

    Be respectful, collaborative, and constructive. We're building tools to help the Zcash ecosystem, and we welcome contributors of all skill levels.

    ---

    ## Getting Started

    ### Prerequisites

    - Linux (Ubuntu 22.04+), WSL, or macOS
    - Docker Engine ≥ 24.x + Compose v2
    - 2 CPU cores, 4GB RAM, 5GB disk
    - Git

    ### Setup Development Environment

    ```bash
    # Fork and clone
    git clone https://github.com/Zecdev/ZecKit.git
    cd ZecKit

    # Run setup
    ./scripts/setup-dev.sh

    # Start devnet
    docker compose up -d

    # Verify
    ./docker/healthchecks/check-zebra.sh
    ./tests/smoke/basic-health.sh
    ```

    ---

    ## Development Workflow

    ### 1. Create a Branch

    ```bash
    git checkout -b feature/my-feature
    # or
    git checkout -b fix/issue-123
    ```

    Branch naming conventions:
    - `feature/` - New features
    - `fix/` - Bug fixes
    - `docs/` - Documentation only
    - `refactor/` - Code refactoring
    - `test/` - Test improvements

    ### 2. Make Changes

    - Write clear, self-documenting code
    - Add comments for complex logic
    - Update documentation as needed

    ### 3. Test Locally

    ```bash
    # Run smoke tests
    ./tests/smoke/basic-health.sh

    # Check logs
    docker compose logs

    # Clean restart
    docker compose down -v && docker compose up -d
    ```

    ### 4. Commit

    ```bash
    git add .
    git commit -m "feat: add health check retry logic"
    ```

    Commit message format:
    ```
    <type>: <description>

    [optional body]

    [optional footer]
    ```

    Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

    ### 5. Push and Create PR

    ```bash
    git push origin feature/my-feature
    ```

    Then open a Pull Request on GitHub.

    ---

    ## Coding Standards

    ### Shell Scripts

    - Use `#!/bin/bash` shebang
    - Enable strict mode: `set -e`
    - Add comments for non-obvious logic
    - Use meaningful variable names
    - Make scripts executable: `chmod +x`

    ### Docker / Compose

    - Pin image versions (no `latest` tags)
    - Bind to localhost (`127.0.0.1`) for exposed ports
    - Use health checks for all services
    - Document any non-standard configurations

    ### Documentation

    - Use Markdown for all docs
    - Keep README.md updated
    - Add inline comments in configs
    - Document security considerations

    ---

    ## Testing

    ### Required Tests

    All PRs must pass:

    1. **Smoke tests:** `./tests/smoke/basic-health.sh`
    2. **Health checks:** `./docker/healthchecks/check-zebra.sh`
    3. **CI pipeline:** GitHub Actions workflow must pass

    ### Adding New Tests

    When adding features, include tests:

    ```bash
    # Add test to tests/smoke/
    nano tests/smoke/my-new-test.sh

    # Make executable
    chmod +x tests/smoke/my-new-test.sh

    # Verify it works
    ./tests/smoke/my-new-test.sh
    ```

    ---

    ## Submitting Changes

    ### Pull Request Checklist

    - [ ] Branch is up to date with `main`
    - [ ] Smoke tests pass locally
    - [ ] Documentation updated (if applicable)
    - [ ] Commit messages are clear
    - [ ] PR description explains changes
    - [ ] No breaking changes (or clearly documented)

    ### PR Review Process

    1. Automated tests run via CI
    2. Code review by maintainers
    3. Address feedback if needed
    4. Approved PRs are merged

    ---

    ## Milestone Roadmap

    ### Current: M1 - Foundation
    - Repository structure
    - Zebra regtest devnet
    - Health checks & smoke tests
    - CI pipeline

    ### Next: M2 - CLI Tool
    Contributions welcome:
    - Python Flask faucet implementation
    - `zeckit` CLI tool (Rust or Bash)
    - Pre-mined fund automation

    ### Future: M3-M5
    - GitHub Action packaging
    - E2E shielded flows
    - Comprehensive documentation
    - Maintenance window

    ---

    ## Getting Help

    - **Questions:** Open a [GitHub Discussion](https://github.com/Zecdev/ZecKit/discussions)
    - **Bugs:** Open an [Issue](https://github.com/Zecdev/ZecKit/issues)
    - **Community:** [Zcash Forum](https://forum.zcashcommunity.com/)

    ---

    Thank you for contributing to ZecKit!
