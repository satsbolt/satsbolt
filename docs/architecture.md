# SatsBolt - System Architecture Documentation

**Product Name**: SatsBolt  
**Version**: 1.0 (MVP)  
**Date**: June 2026  
**Architect**: Senior Engineer (Rust + Bitcoin focus)  
**Authors**: Muhammad Hamaza, Usman, and SatsBolt Contributors

---

## 1. Executive Architecture Summary

SatsBolt uses a **modular, layered, custodial-first architecture** optimized for:
- Fast 5-week MVP delivery
- High security (immutable double-entry ledger)
- Future extensibility (API-first, hybrid self-custody path)
- Global South realities (low connectivity, mobile-first)

**Core Principle**: **Internal Ledger** handles most transactions (zero fees), Lightning is used only for external inflows/outflows.

---

## 2. High-Level Architecture (C4 Context + Container View)

### System Architecture Diagram

```mermaid
graph TD
    subgraph Client Application Layer
        UserApp(["Flutter Mobile App"])
    end

    subgraph Service Layer (SatsBolt Backend)
        API["api-server :8080"]
        LDK["lightning-node :8081"]
    end

    subgraph Database Layer
        DB["PostgreSQL Database"]
    end

    subgraph External Infrastructure (Polar / Regtest)
        Bitcoin["Polar bitcoind"]
        PolarNodes["Polar LND/CLN Nodes"]
    end

    subgraph External Fiat Off-Ramp
        Bitnob["Bitnob Sandbox API"]
    end

    %% Client communication
    UserApp -->|REST API Auth, Balances, Merchants| API
    
    %% Inter-service communication
    API -->|HTTP: Proxy Invoice / Pay Requests| LDK
    
    %% Database persistence
    API -->|Read/Write Ledger, Users, Invoices| DB
    LDK -->|Write LDK Node State, Write Ledger Entries on Payment| DB
    
    %% External node communication
    LDK -->|JSON-RPC: Block & TX Sync| Bitcoin
    LDK -->|P2P LN Protocol: Peer Connections & Routing| PolarNodes
    
    %% Off-ramp communication
    API -->|HTTP: Request Quote & Payout| Bitnob
```

### Context Diagram (Text)
```
Users (Mobile + USSD) 
    ↓ (HTTPS / REST + WebSocket)
SatsBolt Platform
    ├── Flutter App (Social + Business Modes)
    ├── Rust Backend (Actix-web)
    ├── Internal Ledger (Double-Entry)
    ├── Lightning Integration (LDK)
    └── Fiat Off-Ramp Providers (Plugin)
```

### Container View (Main Components)

1. **Presentation Layer**
   - Flutter Mobile App (Android/iOS)
   - USSD Interface (Africa's Talking)

2. **Application Layer**
   - Rust Backend (Actix-web server)

3. **Domain Layer**
   - User Service, Profile Service
   - Payment Service + Internal Ledger
   - Tiered Fee Engine

4. **Infrastructure Layer**
   - PostgreSQL Database
   - LDK (Lightning Dev Kit)
   - Swap Provider Plugins
   - Redis (optional for caching/rate limiting)

---

## 3. Detailed Component Design & Communication

### 3.1 Internal Ledger (Core Security Component)
- **Type**: Double-entry accounting system (every transaction has debit + credit)
- **Implementation**:
  - Immutable transaction log (`transactions` table)
  - Account balances updated atomically using database transactions
  - Event sourcing pattern for auditability
- **Security Standards** (Best Practices):
  - All writes wrapped in ACID transactions
  - Cryptographic hashing of transaction records
  - Separate hot/cold wallet strategy for Lightning liquidity
  - Regular reconciliation with Lightning node state

### 3.2 Lightning Integration
- **Library**: **LDK (Lightning Dev Kit)** — chosen for Rust-native, flexible, and suitable for custodial use cases.
- **Service Decoupling**: Runs as a separate microservice (`lightning-node` on port 8081) to isolate LDK state management and SQLite linkages from the main REST server.
- **Role**: Handles channel management, invoice generation, and payments.
- **Custodial Flow**:
  - User deposits → LDK node receives. The background worker detects the incoming payment, matches the payment hash, and triggers an authenticated HTTP webhook call to `api-server`'s `/api/v1/internal/settle-deposit` to credit the user liability ledger.
  - Internal transfers bypass Lightning.
  - Withdrawals → `api-server` requests payment via LDK node `/pay` route.

### 3.3 Tiered Fee Engine
- **Model**: Volume-based tiers (familiar fintech style)
- **Criteria**: Monthly transaction volume (sats sent/received) or total value processed.
- Dashboard shows current tier and progress toward next tier.

### 3.4 API Layer (Extensibility)
- RESTful endpoints with clear versioning (`/api/v1/`)
- Implemented and placeholder endpoints:
  - `POST /api/v1/merchant/invoice` - Proxy invoice generation to LDK node.
  - `GET /api/v1/merchant/invoice/{id}` - Retrieve invoice settlement status.
  - `POST /api/v1/ledger/withdraw/lightning` - Pay out outbound Lightning invoices.
  - `POST /api/v1/offramp/quote` - Fetch conversion quotes.
  - `POST /api/v1/ledger/withdraw/offramp` - Initiate fiat bank/mobile money swaps.
  - `/tips/send` - Send off-chain tips between users.
- Webhooks for real-time notifications

### 3.5 Data Flow Example (User Tips Merchant)
1. Sender Flutter app → POST `/tips/send`
2. Backend validates + checks tier/fees
3. Atomic update in Internal Ledger (debit sender, credit receiver)
4. Real-time WebSocket update to both users
5. Optional: Receiver triggers off-ramp to fiat

---

## 4. Technology Stack (Finalized)

| Layer              | Technology                          | Justification |
|--------------------|-------------------------------------|-------------|
| Frontend           | Flutter + GetX                      | Cross-platform, fast real-time updates |
| Backend            | Rust + Actix-web                    | Memory safety, high concurrency |
| Lightning          | LDK (Lightning Dev Kit)             | Rust-native, flexible for custodial |
| Database           | PostgreSQL                          | Strong ACID support for ledger |
| Auth               | JWT + Argon2 hashing                | Secure sessions |
| USSD               | Africa's Talking API                | Nice-to-have for MVP |
| Deployment         | Docker + CI/CD (GitHub Actions)     | From day one |
| Hosting            | DigitalOcean / Railway (start)      | Cost-effective, easy scaling |

---

## 5. Security & Compliance Considerations
- Custodial model with clear risk disclosure
- Plan for hybrid self-custody (premium feature)
- KYC/AML hooks ready for off-ramp partners
- Rate limiting, input sanitization, logging

---

## 6. 5-Week MVP Implementation Plan

**Week 1**: Setup (Rust project, DB, Auth, basic Flutter)
**Week 2**: Internal Ledger + User Profiles
**Week 3**: Social Tipping + Basic Merchant Invoices
**Week 4**: LDK Lightning Integration + Fee Engine
**Week 5**: Testing, USSD basics (if time), API placeholders, Deployment
