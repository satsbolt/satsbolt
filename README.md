# SatsBolt ⚡

### Instant Bitcoin Micropayments for Creators, Communities, and Businesses
[![Rust CI](https://github.com/satbolt-network/satsbolt/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/satbolt-network/satsbolt/actions/workflows/rust-ci.yml)
[![Flutter CI](https://github.com/satbolt-network/satsbolt/actions/workflows/flutter-ci.yml/badge.svg)](https://github.com/satbolt-network/satsbolt/actions/workflows/flutter-ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

SatsBolt is a unified, Lightning-native monorepo platform engineered to make Bitcoin (sats) payments seamless, fast, and accessible for emerging markets across the Global South and beyond. By combining social tipping mechanics with robust merchant point-of-sale tools, SatsBolt provides public, white-label infrastructure for borderless micro-commerce.

---

## 1. The Core Architecture Principle

SatsBolt splits transactions into a hybrid engine to overcome network latency and channel routing fees:
1. **The Internal Ledger:** A high-throughput, immutable double-entry database ledger written in Rust. Transactions between internal platform users settle off-chain instantly with **absolute zero fees**.
2. **The Lightning Rail:** Powered by the Rust-native **Lightning Dev Kit (LDK)**, used strictly for external network inflows (deposits) and outbound withdrawals.

---

## 2. Platform Capabilities

*   **Social Mode:** Custom user profiles (usernames, bios), instant username-based tipping, live global leaderboards, and interactive public tip feeds.
*   **Business Mode:** Variable itemized invoice generation, dynamic point-of-sale (PoS) QR codes, transaction metrics, and analytical tracking interfaces.
*   **Unified Accessibility:** A high-performance Flutter mobile application for data-connected environments alongside an integrated telephony USSD fallback shortcode engine for offline data collection.
*   **Plugin Off-Ramps:** A modular backend trait system (`SwapProvider`) designed to seamlessly integrate local fiat aggregators for instant automatic bank settlement.

---

## 3. Core Tech Stack

| Layer | Component | Purpose |
| :--- | :--- | :--- |
| **Frontend** | Flutter + GetX | Native cross-platform UI with real-time reactive state updates. |
| **Backend Boundary** | Rust + Actix-web | Memory-safe, thread-safe asynchronous API services. |
| **L2 Protocol** | Lightning Dev Kit (LDK) | Comprehensive library-driven channel management and invoice routing. |
| **Persistence** | PostgreSQL | Strict ACID compliance for the internal bookkeeping ledger tables. |
| **Telephony Gateway** | Africa's Talking API | Translates offline USSD cell inputs into standard HTTP API payloads. |

---

## 4. Local Development Quickstart

SatsBolt uses `just` as its unified command runner. Ensure you have the dependencies installed (`cargo install just`).

### 1. Launch Local Infrastructure
Boot up local bitcoind signet nodes, the persistence engines, and auxiliary network mock tools using containerized Docker environments:
```bash
just dev-up

satsbolt/
├── .github/workflows/     # Automated multi-environment GitHub Actions CI
├── backend/               # Rust Workspaces (Crates & Core API Services)
│   ├── crates/            # Decoupled core libraries (ledger, ldk-engine, swap)
│   └── services/          # Public HTTP boundaries and USSD background workers
├── frontend/              # Presentation Layers
│   └── mobile/            # Cross-platform Flutter Mobile Application
├── docker/                # Local Signet node orchestration files
└── justfile               # System automation build recipes

This project is licensed under the MIT License - see the LICENSE file for details.