# justfile - Task runner for the Dog Walker API
# Install: cargo install just (or use devenv/devbox which includes it)
# Usage: just <recipe>

# Default recipe - show help
default:
    @just --list

# ============================================================================
# Development
# ============================================================================

# Run the API server with hot reload
dev:
    cargo watch -x 'run --bin api'

# Run the API server (no hot reload)
run:
    cargo run --bin api

# Run all tests
test:
    cargo test

# Run tests with coverage
test-coverage:
    cargo llvm-cov

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting and lints
check: fmt lint
    cargo check

# ============================================================================
# Database
# ============================================================================

# Run database migrations
migrate:
    sqlx migrate run --source migrations

# Reset database (drop, create, migrate)
db-reset:
    sqlx database drop -y || true
    sqlx database create
    sqlx migrate run --source migrations

# Open psql shell
db-shell:
    psql $DATABASE_URL

# ============================================================================
# Kubernetes / Tilt
# ============================================================================

# Create k3d cluster for development
k8s-setup:
    ./scripts/setup-dev.sh

# Start Tilt development environment
tilt-up:
    tilt up

# Stop Tilt and cleanup
tilt-down:
    tilt down

# Delete k3d cluster
k8s-teardown:
    k3d cluster delete dog-walker

# ============================================================================
# Docker
# ============================================================================

# Build production Docker image
docker-build:
    docker build -t dog-walker-api:latest .

# Build development Docker image
docker-build-dev:
    docker build -t dog-walker-api:dev -f Dockerfile.dev .

# ============================================================================
# CI/CD
# ============================================================================

# Run full CI checks locally
ci: check test
    @echo "CI checks passed!"

# Build release binary
release:
    cargo build --release
