#!/bin/bash
set -e

echo "Starting development server..."

# Wait for postgres
until pg_isready -h postgres -p 5432 -U dogwalker; do
    echo "Waiting for postgres..."
    sleep 2
done

echo "Postgres is ready!"

# Run migrations
echo "Running migrations..."
cd /app
./target/debug/api --migrate-only || echo "Migration flag not implemented yet, skipping..."

# Start the server with cargo-watch for live reloading
echo "Starting API server with live reload..."
exec cargo watch -x 'run --bin api' -w crates
