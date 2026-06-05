# SatsBolt - System Requirements Specification (SRS)

**Product Name**: SatsBolt  
**Version**: 1.0  
**Date**: June 2026  
**Authors**: Muhammad Hamaza and  Usman 

## 1. Introduction

### 1.1 Purpose
This document describes the functional and non-functional requirements for **SatsBolt** — a unified Lightning-native platform for social tipping and merchant payments.

### 1.2 Scope
**In Scope (MVP - 5 Weeks)**:
- User authentication and profiles
- Social tipping (username, QR, internal ledger)
- Basic merchant tools (invoices, dynamic QR codes)
- Lightning Network integration
- Internal zero-fee ledger
- Basic fiat off-ramp plugin
- Foundation for future API integrations (Shopify-ready)

**Out of Scope (MVP)**:
- Advanced e-commerce plugins
- Full non-custodial wallets
- Complex analytics and reporting
- Multi-language support beyond English

**Future Vision**: Become a scalable payment infrastructure with strong API/plugin system for enterprises.

### 1.3 Definitions
- **Sats**: Satoshis (smallest Bitcoin unit)
- **Internal Ledger**: Off-chain balance system for zero-fee transfers between SatsBolt users
- **Global South**: Emerging markets with high mobile usage and crypto adoption

## 2. Overall Description

### 2.1 Product Perspective
SatsBolt bridges the gap between complex Lightning Network and everyday users by providing a simple, social-first interface while maintaining strong backend infrastructure for businesses.

### 2.2 User Classes
1. **Individual / Creator Users** — Use Social Mode for tipping
2. **Merchant / Business Users** — Use Business Mode for accepting payments
3. **Offline Users** — Use USSD interface
4. **Future Enterprise Integrators** — Use APIs

### 2.3 Operating Environment
- Mobile: Android & iOS (Flutter)
- Backend: Cloud-hosted (Railway / Fly.io / DigitalOcean)
- Target Users: Global South (poor connectivity, feature phones)

## 3. Functional Requirements

### 3.1 User Authentication
- Email / Phone + Password / Social login
- JWT-based authentication
- 2FA support (future)

### 3.2 User Profile & Social Features
- Create/Edit profile (username, avatar, bio)
- Social Mode: Send tips using username or QR code
- Leaderboards and public tip feed (optional)
- Balance viewing in real-time

### 3.3 Merchant / Business Features
- Generate invoices and payment requests
- Dynamic QR codes for in-store payments
- Transaction history and basic analytics
- Switch between Social and Business modes

### 3.4 Payment Engine
- **Internal Ledger**: Zero-fee transfers between platform users
- **Lightning Integration**: Inbound (receive) and outbound (withdraw) via LND/LDK/LNBits
- Auto-conversion to stablecoins or local fiat on request

### 3.5 Fiat Off-Ramp
- Plugin architecture (`SwapProvider` trait in Rust)
- Integration with local providers (Bitnob, etc.)
- Direct bank deposit support

### 3.6 USSD Support
- Basic balance check and tipping via Africa's Talking API or similar

### 3.7 API Layer (for future extensibility)
- RESTful APIs for external platforms
- Webhook support for payment confirmations
- Designed to support Shopify/WooCommerce plugins later

## 4. Non-Functional Requirements

### 4.1 Performance
- Internal transfers: < 2 seconds
- Lightning payments: < 10 seconds
- Support 10,000+ concurrent users initially

### 4.2 Scalability
- Backend built with Rust concurrency (Actix-web)
- Horizontal scaling ready
- Internal ledger optimized for high throughput

### 4.3 Security
- Memory-safe Rust backend
- JWT + secure session management
- Encryption for sensitive data
- Rate limiting and input validation

### 4.4 Reliability & Availability
- 99% uptime target
- Transaction atomicity (especially ledger updates)

### 4.5 Usability
- Simple, intuitive UI (Instagram-like)
- Works on low-end Android devices
- USSD fallback for feature phones

### 4.6 Maintainability
- Clean architecture, modular code
- Open-source friendly components

## 5. System Architecture (High-Level)

**Layers**:
1. **Presentation Layer** — Flutter Mobile App + USSD
2. **Application Layer** — Rust Backend (Actix-web)
3. **Domain Layer** — Internal Ledger, Payment Logic
4. **Infrastructure Layer** — Lightning (LDK/LND), Database, Swap Providers

**Key Technologies**:
- **Frontend**: Flutter + GetX (state management)
- **Backend**: Rust + Actix-web
- **Bitcoin/Lightning**: LDK (preferred for flexibility) + LNBits (for quick MVP)
- **Database**: PostgreSQL
- **Authentication**: JWT
- **USSD**: Africa's Talking API
- **FFI**: uniffi-rs (if needed for on-device logic)
- **Deployment**: Docker + Cloud provider

**Communication**:
- Mobile ↔ Backend: REST API + WebSockets (for real-time balance)
- Backend ↔ Lightning: Direct integration via LDK/LND libraries
- Backend ↔ Database: SQLx (Rust)

## 6. Assumptions & Dependencies
- Access to Lightning liquidity via LSPs
- Availability of local fiat off-ramp partners
- Regulatory compliance will be handled per country
- Contributors: Core maintainers and open-source community