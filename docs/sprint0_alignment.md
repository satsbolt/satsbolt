# SatsBolt Sprint 0: Architecture & Alignment Notes

This document captures the core architectural alignment and technical decisions made for **SatsBolt** during Sprint 0.

---

## 1. Lightning Polar Node Environment

Instead of manually running or configuring standalone `bitcoind` and `lnd` daemons, the SatsBolt local development environment leverages **Lightning Polar**.

- **Bitcoind Connection**: LDK (`ldk-engine` crate) connects to the Regtest `bitcoind` node managed by Polar. This connection is established via the standard JSON-RPC interface and ZeroMQ (ZMQ) event notifications.
- **LND Peering**: The platform LDK node establishes peer connections to Polar's virtual LND nodes. This allows for local channel creation, funding, and testing payment routing end-to-end.
- **Port Mapping**: Since Polar runs nodes inside Docker containers, the exposed host ports (e.g., bitcoind RPC port `18443`) are configured in the platform's local `.env` file.

---

## 2. Database Schema (PostgreSQL)

To support transactional atomicity and absolute auditability, SatsBolt implements a **custodial double-entry ledger** coupled with Lightning tracking tables.

### Database Tables:

1. **`users`**: Manages account identifiers, emails, usernames, and Argon2 password hashes.
2. **`sessions`**: Secure session tokens for Actix-web authentication.
3. **`accounts`**: Ledger accounts. Tracks the account types (Asset, Liability, Revenue) and associations:
   - *User accounts* are Liability accounts (the platform owes the user money).
   - *LDK Node account* is an Asset account (funds are held externally on Lightning).
   - *Fees/Revenue account* is a Revenue account.
4. **`ledger_transactions`**: Encapsulates metadata (description, timestamp) for a double-entry transaction.
5. **`ledger_entries`**: Maps account entries to a transaction. Uses credit/debit values (signed integers) where the sum of entries for any single transaction ID **must equal 0**.
6. **`invoices`**: Logs generated BOLT11 Lightning invoices, payment hashes, and settlement statuses.
7. **`withdrawals`**: Records outbound payment details, routing fees, and state.
8. **`payout_jobs`**: An asynchronous job queue for executing local fiat off-ramp swaps.
9. **`ldk_storage`**: Key-value byte arrays (binary blobs) storing serialized LDK channel manager states and monitor updates in PostgreSQL, ensuring persistent channel state across restarts.

---

## 3. Core LDK & Ledger Workflows

### 3.1 Inbound Deposit (Lightning Receipt)
1. User requests a deposit invoice via the mobile application (POST `/api/v1/payments/invoice`).
2. The Actix-web server requests the `ldk-engine` to generate a BOLT11 invoice.
3. The invoice is stored in the PostgreSQL database with status `pending`.
4. When the payer (from Polar) pays the invoice, the LDK engine intercepts the `PaymentClaimable` event.
5. Inside a single PostgreSQL transaction, the engine:
   - Debits the Platform's LDK Node Asset account.
   - Credits the User's Liability account.
   - Updates the invoice status to `settled`.

### 3.2 Internal Ledger Settlement (Tipping)
1. User A sends a tip to User B using their username (POST `/api/v1/tips/send`).
2. The server executes a database transaction:
   - Confirms User A's Liability balance is sufficient.
   - Creates a transaction record.
   - Debits User A's Liability account (reduces balance).
   - Credits User B's Liability account (increases balance).
   - Credits Platform Fee account (if applicable).
3. This is an **off-chain, instant transaction** carrying **zero network fees**.

### 3.3 Outbound Withdrawal & Asynchronous Off-Ramp
1. User initiates a local fiat payout (POST `/api/v1/payments/withdraw`).
2. The server performs a balance check and locks the funds:
   - Debits the User's Liability account.
   - Credits a temporary Pending Payout Liability account.
   - Creates a `withdrawal` record in `pending` status.
   - Creates a `payout_job` in `queued` status.
3. The server immediately returns success to the user (processing asynchronously).
4. The background `payout-worker` polls the `payout_jobs` queue:
   - Resolves the off-ramp Quote and calls `initiate_payout` via the `SwapProvider` plugin.
   - If the swap succeeds: Debits the Pending Payout account, credits the Platform LDK Node Asset account, marks the withdrawal `succeeded`, and completes the job.
   - If the swap fails: Reverses the lock by crediting the User's Liability account and debiting the Pending Payout account, marking the withdrawal `failed`.

---

## 4. Off-Ramp SwapProvider Trait

We define a pluggable architecture for converting Sats to fiat using external swap/payment providers (like Bitnob):

```rust
pub trait SwapProvider: Send + Sync {
    fn name(&self) -> &str;
    fn get_quote(&self, amount_sats: u64, currency: &str) -> Result<Quote, Box<dyn Error>>;
    fn initiate_payout(&self, quote_id: &str, destination: PayoutDestination) -> Result<SwapResult, Box<dyn Error>>;
    fn get_status(&self, swap_id: &str) -> Result<SwapStatus, Box<dyn Error>>;
}
```

---

## 5. Deployment and Production Assumptions

- **Target Platforms**: Dockerized deployments hosted on container infrastructure (e.g. DigitalOcean, Railway, or Fly.io).
- **Environment Management**: Key secrets (JWT secrets, DB credentials, Swap API keys) are supplied via environment variables.
- **Node Persistence**: Active LDK states are continuously written to the postgres-backed `ldk_storage` table, preventing state loss during container migrations or redeployments.
