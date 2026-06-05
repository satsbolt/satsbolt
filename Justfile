# SatsBolt Build & Automation Runner
# Use 'just' to run any recipe. Example: 'just dev-up'

# List all available recipes
default:
    @just --list

# --- Backend Recipes (Rust) ---

# Run cargo check on backend
check-backend:
    cargo check --manifest-path backend/Cargo.toml

# Check backend code formatting (nightly formatted)
fmt-backend:
    cargo fmt --manifest-path backend/Cargo.toml --all -- --check

# Run Clippy linter on backend
lint-backend:
    cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings

# Execute backend tests
test-backend:
    cargo test --manifest-path backend/Cargo.toml --all

# --- Frontend Recipes (Flutter) ---

# Fetch Flutter dependencies
get-frontend:
    cd frontend/mobile && flutter pub get

# Check frontend code formatting
fmt-frontend:
    cd frontend/mobile && dart format --set-exit-if-changed lib test

# Analyze Dart code for errors
lint-frontend:
    cd frontend/mobile && flutter analyze

# Run Flutter unit and widget tests
test-frontend:
    cd frontend/mobile && flutter test

# --- Docker & Infrastructure Recipes ---

# Spin up local PostgreSQL and Mock Bitcoin Node containers
dev-up:
    docker compose -f docker/docker-compose.yml up -d

# Stop and remove local containers and networks
dev-down:
    docker compose -f docker/docker-compose.yml down

# View logs from running containers
dev-logs:
    docker compose -f docker/docker-compose.yml logs -f
