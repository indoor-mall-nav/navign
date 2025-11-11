# PostgreSQL Integration Tests

This directory contains comprehensive integration tests for the PostgreSQL database migration.

## Overview

These tests validate the PostgreSQL database schema, PostGIS spatial operations, JSONB operations, and foreign key constraints. They are ready to use once the server migration from MongoDB to PostgreSQL is complete.

## Test Coverage

### `database_test.rs`

**Database Connection Tests:**
- Database connection and pooling
- PostgreSQL extensions (uuid-ossp, PostGIS)
- Table existence verification
- Connection pool settings

**Entity CRUD Tests:**
- Insert, select, update, delete operations
- Constraint validation (CHECK constraints)
- JSONB tag operations
- JSONB query operations (`@>` operator)

**PostGIS Geometry Tests:**
- Area insertion with POLYGON geometries
- Beacon insertion with POINT geometries
- Point-in-polygon queries (`ST_Contains`)
- Distance calculations (`ST_Distance`)
- Nearest beacon queries
- Area calculations (`ST_Area`)

### `schema_test.rs`

**Merchant Tests:**
- JSONB type field operations
- Polygon vs point style handling
- Tags with JSONB containment queries
- Social media JSONB storage
- Unique constraint validation

**Connection Tests:**
- Connection type validation (gate, escalator, elevator, stairs, rail, shuttle)
- JSONB connected_areas array
- Available period JSONB arrays
- Ground point geometry
- Tags with JSONB queries

**Foreign Key Constraint Tests:**
- CASCADE delete from entities to areas to beacons
- CASCADE delete from areas to beacons/merchants
- SET NULL delete for merchant references in beacons
- Foreign key violation detection
- Beacon secrets cascade delete

**User Tests:**
- Password-based user registration
- OAuth user registration (GitHub, Google)
- Username and email unique constraints
- Public key storage (BYTEA)
- Unlock instance creation and querying

**Firmware Tests:**
- Firmware version insertion
- Device-specific firmware queries
- Unique constraint validation (version + device)
- Latest stable firmware queries

## Running Tests

### Prerequisites

1. **Install PostgreSQL 17+:**
   ```bash
   # macOS
   brew install postgresql postgis

   # Linux
   sudo apt-get install postgresql-17 postgresql-17-postgis-3
   ```

2. **Start PostgreSQL:**
   ```bash
   # macOS
   brew services start postgresql

   # Linux
   sudo systemctl start postgresql
   ```

3. **Create test database:**
   ```bash
   psql -U postgres -c "CREATE DATABASE navign_test;"
   psql -U postgres -d navign_test -c "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";"
   psql -U postgres -d navign_test -c "CREATE EXTENSION IF NOT EXISTS postgis;"
   ```

4. **Set environment variable:**
   ```bash
   export DATABASE_URL="postgres://postgres:postgres@localhost:5432/navign_test"
   ```

### Run Tests

```bash
cd server
cargo test --tests
```

### Run Specific Test Module

```bash
cargo test --test database_test
cargo test --test schema_test
```

### Run Specific Test Function

```bash
cargo test --test database_test test_postgis_spatial_query
cargo test --test schema_test test_cascade_delete_entity
```

## Current Status

⚠️ **Note:** These tests are currently part of the PostgreSQL migration effort documented in `MIGRATION_MONGODB_TO_POSTGRESQL.md`. The server binary migration is still in progress, so these tests will only compile and run once the following are completed:

1. Server schema handler implementations updated for PostgreSQL
2. Authentication and user management updated
3. Pathfinding updated to use PostGIS
4. Access control and firmware management updated

## CI Integration

The GitHub Actions CI workflow has been updated to:
1. Use PostgreSQL 17 instead of MongoDB
2. Install PostGIS and uuid-ossp extensions
3. Create a test database with proper extensions
4. Set the `DATABASE_URL` environment variable

## Test Database Cleanup

Each test module includes a `cleanup_database` helper function that truncates all tables with CASCADE to ensure test isolation:

```rust
async fn cleanup_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("TRUNCATE TABLE unlock_instances, beacon_secrets, beacons,
                 merchants, connections, areas, users, firmwares, entities CASCADE")
        .execute(pool)
        .await?;
    Ok(())
}
```

## Future Enhancements

- [ ] Add benchmark tests for spatial query performance
- [ ] Add tests for concurrent access and connection pooling
- [ ] Add tests for database migration rollback
- [ ] Add tests for complex pathfinding queries with PostGIS
- [ ] Add load testing for high-volume operations
- [ ] Add tests for database backups and recovery

## Related Documentation

- [Migration Guide](../MIGRATION_MONGODB_TO_POSTGRESQL.md)
- [Database Schema](../migrations/20250109000001_initial_schema.sql)
- [PostGIS Documentation](https://postgis.net/documentation/)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
