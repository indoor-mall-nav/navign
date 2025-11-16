#!/usr/bin/env bash
# MongoDB to PostgreSQL Migration Script
#
# This script helps you migrate data from MongoDB to PostgreSQL.
#
# Prerequisites:
# 1. MongoDB running and accessible
# 2. PostgreSQL running and accessible
# 3. PostgreSQL schema already migrated (run server with POSTGRES_RUN_MIGRATIONS=true first)
#
# Usage:
#   ./scripts/migrate.sh [OPTIONS]
#
# Options:
#   --dry-run         Print what would be migrated without executing
#   --skip-existing   Skip records that already exist in PostgreSQL
#   --batch-size N    Number of records to migrate at once (default: 100)
#   --help            Show this help message

set -e

# Default values
MONGODB_HOST="${MONGODB_HOST:-localhost:27017}"
MONGODB_DB_NAME="${MONGODB_DB_NAME:-navign}"
POSTGRES_URL="${POSTGRES_URL:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print colored message
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Show help
show_help() {
    grep '^#' "$0" | sed 's/^# \?//'
    exit 0
}

# Parse arguments
EXTRA_ARGS=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_help
            ;;
        --dry-run)
            EXTRA_ARGS="$EXTRA_ARGS --dry-run"
            shift
            ;;
        --skip-existing)
            EXTRA_ARGS="$EXTRA_ARGS --skip-existing"
            shift
            ;;
        --batch-size)
            EXTRA_ARGS="$EXTRA_ARGS --batch-size $2"
            shift 2
            ;;
        *)
            error "Unknown option: $1"
            show_help
            ;;
    esac
done

# Check prerequisites
info "Checking prerequisites..."

# Check if PostgreSQL URL is set
if [ -z "$POSTGRES_URL" ]; then
    error "POSTGRES_URL environment variable is not set"
    echo ""
    echo "Example:"
    echo "  export POSTGRES_URL=postgresql://user:password@localhost:5432/navign"
    exit 1
fi

# Check if MongoDB is accessible
info "Testing MongoDB connection at $MONGODB_HOST..."
if ! timeout 5 mongosh "mongodb://$MONGODB_HOST/$MONGODB_DB_NAME" --quiet --eval "db.runCommand({ ping: 1 })" > /dev/null 2>&1; then
    if ! timeout 5 mongo "mongodb://$MONGODB_HOST/$MONGODB_DB_NAME" --quiet --eval "db.runCommand({ ping: 1 })" > /dev/null 2>&1; then
        warn "Could not connect to MongoDB (this might be OK if mongosh/mongo is not in PATH)"
        warn "Migration will fail if MongoDB is not accessible"
    fi
else
    info "MongoDB connection successful"
fi

# Check if PostgreSQL is accessible
info "Testing PostgreSQL connection..."
if ! command -v psql &> /dev/null; then
    warn "psql not found in PATH, skipping PostgreSQL connection test"
else
    if ! psql "$POSTGRES_URL" -c "SELECT 1" > /dev/null 2>&1; then
        error "Could not connect to PostgreSQL"
        exit 1
    fi
    info "PostgreSQL connection successful"
fi

# Check if schema is initialized
info "Checking if PostgreSQL schema is initialized..."
SCHEMA_EXISTS=$(psql "$POSTGRES_URL" -t -c "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'entities')" 2>/dev/null | xargs || echo "")

if [ "$SCHEMA_EXISTS" != "t" ]; then
    error "PostgreSQL schema not initialized"
    echo ""
    echo "Please run migrations first:"
    echo "  POSTGRES_URL=$POSTGRES_URL POSTGRES_RUN_MIGRATIONS=true cargo run --bin navign-server"
    exit 1
fi

info "PostgreSQL schema is initialized"

# Count MongoDB records
info "Counting MongoDB records..."
ENTITY_COUNT=$(mongosh "mongodb://$MONGODB_HOST/$MONGODB_DB_NAME" --quiet --eval "db.entities.countDocuments()" 2>/dev/null || echo "?")
USER_COUNT=$(mongosh "mongodb://$MONGODB_HOST/$MONGODB_DB_NAME" --quiet --eval "db.users.countDocuments()" 2>/dev/null || echo "?")
AREA_COUNT=$(mongosh "mongodb://$MONGODB_HOST/$MONGODB_DB_NAME" --quiet --eval "db.areas.countDocuments()" 2>/dev/null || echo "?")
BEACON_COUNT=$(mongosh "mongodb://$MONGODB_HOST/$MONGODB_DB_NAME" --quiet --eval "db.beacons.countDocuments()" 2>/dev/null || echo "?")

info "MongoDB records:"
info "  Entities: $ENTITY_COUNT"
info "  Users: $USER_COUNT"
info "  Areas: $AREA_COUNT"
info "  Beacons: $BEACON_COUNT"

# Confirm before proceeding
if [[ ! "$EXTRA_ARGS" =~ "--dry-run" ]]; then
    echo ""
    warn "This will migrate data from MongoDB to PostgreSQL"
    read -p "Do you want to continue? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Migration cancelled"
        exit 0
    fi
fi

# Run migration
info "Starting migration..."
echo ""

export MONGODB_HOST
export MONGODB_DB_NAME
export POSTGRES_URL

if cargo run --bin migrate -- $EXTRA_ARGS; then
    echo ""
    info "Migration completed successfully! 🎉"

    # Show PostgreSQL counts
    if command -v psql &> /dev/null && [[ ! "$EXTRA_ARGS" =~ "--dry-run" ]]; then
        info "PostgreSQL records:"
        PG_ENTITY_COUNT=$(psql "$POSTGRES_URL" -t -c "SELECT COUNT(*) FROM entities" | xargs)
        PG_USER_COUNT=$(psql "$POSTGRES_URL" -t -c "SELECT COUNT(*) FROM users" | xargs)
        PG_AREA_COUNT=$(psql "$POSTGRES_URL" -t -c "SELECT COUNT(*) FROM areas" | xargs)
        PG_BEACON_COUNT=$(psql "$POSTGRES_URL" -t -c "SELECT COUNT(*) FROM beacons" | xargs)

        info "  Entities: $PG_ENTITY_COUNT"
        info "  Users: $PG_USER_COUNT"
        info "  Areas: $PG_AREA_COUNT"
        info "  Beacons: $PG_BEACON_COUNT"
    fi
else
    echo ""
    error "Migration failed"
    exit 1
fi
