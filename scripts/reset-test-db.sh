#!/bin/bash
# =====================================================
# Reset Test Database
# =====================================================
# Drops and recreates the test database, runs migrations,
# and seeds with test data.
# =====================================================

set -e

# Configuration
DB_NAME="${TEST_DB_NAME:-offleash_test}"
DB_USER="${TEST_DB_USER:-offleash}"
DB_HOST="${TEST_DB_HOST:-localhost}"
DB_PORT="${TEST_DB_PORT:-5432}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "============================================="
echo "Resetting Test Database: $DB_NAME"
echo "============================================="

# Check if database exists and drop it
echo "Dropping existing database (if exists)..."
dropdb --if-exists -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" "$DB_NAME" 2>/dev/null || true

# Create fresh database
echo "Creating fresh database..."
createdb -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" "$DB_NAME"

# Run migrations
echo "Running migrations..."
cd "$PROJECT_ROOT"
if [ -f ".env.test" ]; then
    export $(grep -v '^#' .env.test | xargs)
fi

# Use sqlx-cli if available, otherwise run raw SQL
if command -v sqlx &> /dev/null; then
    DATABASE_URL="postgres://$DB_USER@$DB_HOST:$DB_PORT/$DB_NAME" sqlx migrate run
else
    echo "Warning: sqlx-cli not found, running migrations manually..."
    for migration in "$PROJECT_ROOT/migrations"/*.sql; do
        echo "  Running: $(basename "$migration")"
        psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$migration" > /dev/null
    done
fi

# Seed test data
echo "Seeding test data..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$PROJECT_ROOT/tests/fixtures/seed.sql"

echo ""
echo "============================================="
echo "Test database reset complete!"
echo "============================================="
