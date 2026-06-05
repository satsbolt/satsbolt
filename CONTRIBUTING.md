# Contributing to SatsBolt ⚡

Thank you for contributing to SatsBolt! 

SatsBolt is built to be the default public value-transfer infrastructure in emerging markets. By contributing, you are helping build a highly scalable, Lightning-native digital economy that empowers creators and local businesses.

We value structural codebase integrity, comprehensive communication, and clean code standards. Please review these guidelines to ensure a smooth contribution process.

---

## 1. Local Environment Prerequisites

To run the SatsBolt monorepo workspace locally, ensure you have the following installed:
1. **Rust Toolchain:** Latest stable channel + `nightly` component (required for backend rustfmt styling checks).
2. **Flutter SDK:** Latest stable release channel.
3. **Just Command Runner:** Install via `cargo install just`. We use `just` to run build, test, and format pipelines.
4. **Docker & Docker Compose:** Required to orchestrate PostgreSQL and Bitcoin signet/regtest containers.

---

## 2. Setting Up Your Development Environment

1. Fork the repository and clone your fork.
2. Initialize your local configuration file:
   ```bash
   cp .env.example .env
   ```
3. Spin up the backend support containers (database and bitcoin nodes):
   ```bash
   just dev-up
   ```

---

## 3. The Development & QA Workflow

### Branching Strategy
We enforce strict branch protection on `main`. Create descriptive branches for your work:
- Features: `feat/add-lnurl-pay`
- Bug Fixes: `fix/ledger-race-condition`
- Documentation & Chores: `chore/update-readme`

### Backend Checks & Tests (Rust)
Before pushing any backend changes, verify formatting, linting, and unit tests:
```bash
# Formats backend code using nightly conventions
just fmt-backend

# Lints backend using cargo clippy
just lint-backend

# Executes all unit and integration tests
just test-backend
```

### Frontend Checks & Tests (Flutter)
Before pushing any frontend changes, verify formatting, analysis, and tests:
```bash
# Fetches latest packages
just get-frontend

# Formats Dart code
just fmt-frontend

# Runs flutter analyze linter
just lint-frontend

# Runs widget and unit tests
just test-frontend
```

---

## 4. Commit Guidelines

We recommend using **Conventional Commits** for clear release tracking:
- `feat: add support for dynamic lightning invoices`
- `fix: prevent database connection pool exhaustion`
- `docs: update API specifications for offramp endpoints`
- `style: format imports and fix formatting warnings`
- `refactor: extract ledger transaction signing logic`

---

## 5. Submitting a Pull Request

1. Push your branch to your remote fork.
2. Open a Pull Request against our `main` branch.
3. Complete the **Pull Request Template** provided in `.github/PULL_REQUEST_TEMPLATE.md`, detail the changes, testing process, and attach any relevant logs or UI screenshots.
4. Ensure the GitHub Actions CI pipelines pass successfully. A maintainer will review your code shortly!