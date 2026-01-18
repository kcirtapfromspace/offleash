# Dog Walker Booking API

A travel-aware dog walking booking application with Rust backend, PostgreSQL database, and Google Maps integration.

## Features

- **Availability Engine**: Gap-based scheduling with travel time consideration
- **Double-booking Prevention**: Transaction locking with advisory locks
- **JWT Authentication**: Secure user authentication
- **Travel Time Matrix**: Google Maps Distance Matrix integration
- **Square Payments**: Prepay model (coming soon)

## Tech Stack

- **Backend**: Rust + Axum
- **Database**: PostgreSQL + SQLx
- **Container**: Docker + Kubernetes
- **Local Dev**: Tilt, Devbox, or devenv (Nix)

---

## Quick Start

Choose your preferred development environment:

### Option 1: Devbox (Recommended for beginners)

```bash
# Install devbox
curl -fsSL https://get.jetify.com/devbox | bash

# Start the environment
devbox shell

# Start PostgreSQL
devbox services up

# Run migrations
devbox run migrate

# Start the API with hot reload
devbox run dev
```

### Option 2: devenv (Nix-based, most reproducible)

```bash
# Install devenv (requires Nix)
# See: https://devenv.sh/getting-started/

# Start the environment
devenv shell

# Start PostgreSQL in background
devenv up -d

# Run migrations
migrate

# Start the API with hot reload
dev
```

### Option 3: Tilt + k3d (Kubernetes-native)

```bash
# Install prerequisites: docker, kubectl, k3d, tilt

# Setup cluster and start
./scripts/setup-dev.sh
tilt up

# Open Tilt UI (press 's' in terminal or visit http://localhost:10350)
```

### Option 4: Docker Compose (Simple containers)

```bash
docker compose up -d
```

---

## Project Structure

```
dog_walker/
├── crates/
│   ├── api/           # HTTP API server (Axum routes, middleware)
│   ├── domain/        # Pure business logic (availability engine)
│   ├── shared/        # Shared types (IDs, Money, Time, Coordinates)
│   ├── db/            # Database models and repositories
│   └── integrations/  # Google Maps, Square
├── migrations/        # SQLx database migrations
├── k8s/              # Kubernetes manifests
│   ├── base/         # Base resources
│   └── dev/          # Development overlays
├── scripts/          # Development scripts
├── Tiltfile          # Tilt configuration
├── devbox.json       # Devbox configuration
├── devenv.nix        # devenv/Nix configuration
└── justfile          # Task runner commands
```

## Common Commands

Using `just` (install with `cargo install just`):

```bash
just dev         # Run with hot reload
just test        # Run tests
just lint        # Run clippy
just migrate     # Run database migrations
just db-reset    # Reset database
just k8s-setup   # Create k3d cluster
just tilt-up     # Start Tilt
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| POST | `/auth/register` | Register new user |
| POST | `/auth/login` | Login |
| GET | `/services` | List services |
| GET | `/availability/:walker_id` | Get available slots |
| POST | `/bookings` | Create booking |
| POST | `/bookings/:id/confirm` | Confirm booking |
| POST | `/bookings/:id/cancel` | Cancel booking |
| POST | `/locations` | Create location |
| GET | `/locations` | List user locations |
| POST | `/blocks` | Create availability block |

## Environment Variables

```bash
DATABASE_URL=postgres://user:pass@localhost:5432/dog_walker
JWT_SECRET=your_secret_here
GOOGLE_MAPS_API_KEY=optional_for_dev
RUST_LOG=debug,tower_http=debug,sqlx=warn
```

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_availability
```

## License

MIT
