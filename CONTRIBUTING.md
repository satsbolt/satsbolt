# Contributing to SatsBolt ⚡

First off, thank you for considering contributing to SatsBolt! 

SatsBolt is built to be the default public infrastructure for borderless, instant Bitcoin value transfer in the Global South. By contributing, you are helping build a highly scalable, Lightning-native digital economy that empowers creators and local businesses.

We value structural codebase integrity, comprehensive communication, and public learning. Please review these guidelines to ensure a smooth contribution process.

---

## 1. The Technology Stack
Our repository is structured as a Monorepo. You will be working across two distinct environments:
*   **Backend (Rust):** An Actix-web workspace managing our immutable double-entry ledger, PostgreSQL database, and Lightning Dev Kit (LDK) node infrastructure.
*   **Frontend (Flutter):** A cross-platform mobile client utilizing GetX for reactive state management and instantaneous UI updates.

## 2. Local Environment Prerequisites
To run the full SatsBolt stack locally, ensure your machine has the following installed:
1.  **Rust Toolchain:** Latest stable channel + the `nightly` component (required for backend code formatting).
2.  **Flutter SDK:** Latest stable release channel.
3.  **Just Command Runner:** Install via `cargo install just`. We use `just` to automate all build and test pipelines.
4. **Docker & Docker Compose:** Required to orchestrate local PostgreSQL databases and Signet Bitcoin nodes.

---

## 3. The Development Workflow

### Step 1: Branching
We enforce strict branch protection on `master`. Never commit directly to the master branch. 
Create a descriptive branch for your work:
*   Features: `feat/add-lnurl-pay`
*   Bug Fixes: `fix/ledger-race-condition`
*   Chores/Docs: `chore/update-readme`

### Step 2: Boot Local Infrastructure
Before testing your code, spin up the local database and mock nodes:
```bash
just dev-up