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
- 📄 [Sprint 0 Alignment Notes](docs/sprint0_alignment.md)

---

## 2. Key Features & Tech Stack

### Features
- **Zero-Fee Internal Ledger**: Instant, high-throughput double-entry accounting transactions settled off-chain with absolute zero fees.
- **Lightning Network Rail**: Inbound deposits and outbound withdrawals integrated with Rust-native LDK (Lightning Dev Kit).
- **Asynchronous Off-Ramp**: Convert sats to local fiat using a background worker queue with support for external swap providers.
- **Dual-Mode Mobile App**: Seamless switching between Creator (Tipping) and Merchant (Point of Sale) modes.
- **USSD Interface Support**: Telephony gateway integration for offline users.

### Technology Stack
- **Backend**: Rust, Actix-web, SQLx, PostgreSQL
- **Lightning Rail**: LDK (Lightning Dev Kit)
- **Frontend Client**: Flutter, GetX
- **Development Tooling**: Lightning Polar, Docker, Justfile

---

## 3. Directory Structure

```
satsbolt/
├── backend/            # Rust workspace (API Server, USSD Worker, LDK, Ledger)
├── frontend/           # Mobile client app (Flutter)
├── docker/             # Local database orchestration configurations
├── docs/               # Technical specifications & documentation
└── scripts/            # Script helpers for automation
```

---

## 3. Local Development & Setup

SatsBolt uses `just` as a unified build and execution tool. Ensure you have the tool runner installed:
```bash
cargo install just
```

### Step 1: Copy Environment Configuration
Create your local `.env` configuration file from the template:
```bash
cp .env.example .env
```

### Step 2: Database Setup
You can run PostgreSQL either natively on your system or via Docker Compose.

#### Option A: Native PostgreSQL Setup (Recommended for Linux/macOS)
1. **Install PostgreSQL**:
   - **Ubuntu/Debian**: `sudo apt install -y postgresql postgresql-contrib`
   - **macOS**: `brew install postgresql@15`
2. **Start and enable the service**:
   - `sudo systemctl start postgresql`
   - `sudo systemctl enable postgresql`
3. **Configure Database & User Credentials**:
   Access the Postgres console:
   ```bash
   sudo -u postgres psql
   ```
   Run the SQL commands to create the database, user, and grant privileges:
   ```sql
   CREATE DATABASE satsbolt_ledger;
   CREATE USER satsbolt WITH PASSWORD 'secretpassword';
   GRANT ALL PRIVILEGES ON DATABASE satsbolt_ledger TO satsbolt;
   \q
   ```
4. **Apply SQL Schema Migration**:
   ```bash
   sudo -u postgres psql -d satsbolt_ledger -f backend/migrations/0001_init_schema.sql
   ```

#### Option B: Docker Compose Setup
1. **Spin up the database container**:
   ```bash
   just dev-up
   ```
2. **Apply SQL Schema Migration**:
   ```bash
   docker exec -i satsbolt-db psql -U satsbolt -d satsbolt_ledger < backend/migrations/0001_init_schema.sql
   ```

### Step 3: Bitcoin & Lightning Node Setup (Lightning Polar)
SatsBolt uses **Lightning Polar** to manage local Regtest bitcoin nodes and LND/CLN instances.
1. Download and run [Lightning Polar](https://lightningpolar.com/).
2. Create a new network with at least 1 `bitcoind` node and 2 `LND` (or CLN) nodes.
3. Start the network in Polar.
4. Update your local `.env` configuration with the Polar nodes' details:
   - `BITCOIN_RPC_URL`: Set to the RPC endpoint of the `bitcoind` node.
   - `BITCOIN_RPC_USER` & `BITCOIN_RPC_PASS`: Set to the RPC credentials shown in Polar.
   - For LDK engine peer connections, use the peer addresses and ports exposed by your Polar nodes.

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

## 5. Security

If you discover a security vulnerability, please do not open a public issue. Follow the security instructions in [CONTRIBUTING.md](CONTRIBUTING.md) or contact security coordinators directly.

---

## 6. License

SatsBolt is released under the [MIT License](LICENSE).