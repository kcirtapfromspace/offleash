# syntax=docker/dockerfile:1.4

# ============================================================================
# Stage 1: Build dependencies (cached layer)
# ============================================================================
FROM rust:1.75-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# ============================================================================
# Stage 2: Plan dependencies
# ============================================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ============================================================================
# Stage 3: Build dependencies (this layer is cached unless Cargo.toml changes)
# ============================================================================
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin api

# ============================================================================
# Stage 4: Runtime image
# ============================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/api /app/api

# Copy migrations for runtime migration
COPY --from=builder /app/migrations /app/migrations

# Create non-root user
RUN useradd -r -s /bin/false appuser && chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=info
EXPOSE 8080

CMD ["/app/api"]
