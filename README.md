# SatsBolt ⚡

### Instant Bitcoin Micropayments for Creators, Communities, and Businesses

[![Backend CI](https://github.com/satsbolt-network/satsbolt/actions/workflows/backend.yml/badge.svg)](https://github.com/satsbolt-network/satsbolt/actions/workflows/backend.yml)
[![Mobile CI](https://github.com/satsbolt-network/satsbolt/actions/workflows/mobile.yml/badge.svg)](https://github.com/satsbolt-network/satsbolt/actions/workflows/mobile.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

SatsBolt is a unified, Lightning-native monorepo platform engineered to make Bitcoin (sats) payments seamless, fast, and accessible. By combining social tipping mechanics with robust merchant point-of-sale tools, SatsBolt provides public, white-label infrastructure for borderless micro-commerce.

---

## 1. Core Architecture Principle

SatsBolt splits transactions into a hybrid engine to optimize performance and reduce fees:
1. **The Internal Ledger:** A high-throughput, ACID-compliant double-entry accounting ledger written in Rust. Transactions between internal platform users settle off-chain instantly with **absolute zero fees**.
2. **The Lightning Rail:** Powered by the Rust-native **Lightning Dev Kit (LDK)**, used strictly for external network inflows (deposits) and outbound withdrawals.

For structural specs and requirements, see the documentation in `docs/`:
- 📄 [Software Requirements Specification (SRS)](docs/srs.md)
- 📄 [System Architecture Documentation](docs/architecture.md)

---

## 2. Directory Structure

```
satsbolt/
├── .github/
│   ├── ISSUE_TEMPLATE/        # Bug report and feature request templates
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── workflows/
│       ├── backend.yml        # Rust compiler and test workflow
│       └── mobile.yml         # Flutter test and analyzer workflow
├── docs/                      # Technical specifications and documentation
├── backend/                   # Rust Workspace Root
│   ├── Cargo.toml
│   ├── crates/                # Core libraries
│   │   ├── core-ledger/       # Double-entry ledger engine
│   │   ├── ldk-engine/        # Lightning Dev Kit integration
│   │   └── offramp-swap/      # SwapProvider traits and plugins
│   └── services/              # Executable applications
│       ├── api-server/        # Actix-web backend server
│       └── ussd-worker/       # Offline USSD background processor
├── frontend/
│   └── mobile/                # Flutter native mobile app
├── docker/                    # PostgreSQL and Bitcoind Regtest containers
├── scripts/                   # Utility development scripts
├── .env.example               # Environmental variable templates
├── Justfile                   # Task automation recipe scripts
├── LICENSE                    # MIT License
├── README.md
├── CONTRIBUTING.md            # Git branching and contribution guide
└── CODE_OF_CONDUCT.md
```

---

## 3. Local Development & Setup

SatsBolt uses `just` as a unified build and execution tool. Ensure you have the tool runner installed:
```bash
cargo install just
```

### 1. Copy Environment Configuration
```bash
cp .env.example .env
```

### 2. Launch Local Signet/Regtest Infrastructure
Boot up the Docker containers containing the PostgreSQL database and mock Bitcoin node:
```bash
just dev-up
```

---

## 4. Testing Suite (Crucial Verification)

Open-source contributions must pass the verification suite. SatsBolt implements unit, integration, and functional testing levels.

### Running Backend Tests (Rust)
Use the unified Justfile recipe:
```bash
just test-backend
```

Under the hood, you can run specific levels:
*   **Unit Tests:** Verify individual library functions.
    ```bash
    cargo test --workspace --lib
    ```
*   **Integration Tests:** Verify ledger database queries.
    ```bash
    cargo test --test ledger_integration
    ```
*   **Functional Tests:** Verify web API route actions.
    ```bash
    cargo test --test api_functional
    ```

### Running Frontend Tests (Flutter)
```bash
just get-frontend
just test-frontend
```

---

## 5. Security Policy

If you discover a vulnerability, please do not open a public issue. Instead, report it directly to the maintainers or follow the security coordinator instructions in [CONTRIBUTING.md](CONTRIBUTING.md).

---

## 6. Authors & Contributors

### Authors
*   **Muhammad Hamaza** (Core Maintainer)
*   **Usman** (Core Maintainer)


### Contributors
We welcome contributions from the open-source community! Check out [CONTRIBUTING.md](CONTRIBUTING.md) to get started.