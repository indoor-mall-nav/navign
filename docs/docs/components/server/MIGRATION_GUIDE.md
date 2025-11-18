# MongoDB to PostgreSQL Migration Guide

This guide explains how to migrate your Navign data from MongoDB to PostgreSQL.

## Overview

The migration process involves:

1. **Schema Migration**: Create PostgreSQL tables using SQLx migrations
2. **Data Migration**: Transfer all data from MongoDB to PostgreSQL
3. **Dual-Database Mode**: Run server with both databases simultaneously
4. **Switch to PostgreSQL**: Disable MongoDB and use PostgreSQL exclusively

## Prerequisites

Before starting the migration, ensure you have:

- ✅ MongoDB instance running with your data
- ✅ PostgreSQL 12+ installed and running
- ✅ PostGIS extension available
- ✅ Rust toolchain installed
- ✅ Database backup (recommended)

## Phase 1: Schema Migration

First, create the PostgreSQL schema:

```bash
# Set PostgreSQL connection URL
export POSTGRES_URL="postgresql://user:password@localhost:5432/navign"

# Run server with migrations enabled (will apply schema and exit)
POSTGRES_RUN_MIGRATIONS=true cargo run --bin navign-server
```

This will:
- Create all tables (entities, users, areas, beacons, merchants, connections, etc.)
- Set up indexes and constraints
- Install PostGIS extension
- Create triggers for automatic timestamp updates

**Verify schema creation:**

```bash
psql $POSTGRES_URL -c "\dt"
```

You should see tables: entities, users, areas, beacons, merchants, connections, beacon_secrets, user_public_keys, firmwares.

## Phase 2: Data Migration

### Option A: Using the migration script (Recommended)

```bash
# Navigate to server directory
cd server

# Set environment variables
export MONGODB_HOST="localhost:27017"
export MONGODB_DB_NAME="navign"
export POSTGRES_URL="postgresql://user:password@localhost:5432/navign"

# Dry run to see what will be migrated
./scripts/migrate.sh --dry-run

# Run actual migration
./scripts/migrate.sh

# Run migration with skip-existing flag (useful for incremental updates)
./scripts/migrate.sh --skip-existing

# Run with custom batch size
./scripts/migrate.sh --batch-size 50
```

### Option B: Using cargo directly

```bash
# Set environment variables
export MONGODB_HOST="localhost:27017"
export MONGODB_DB_NAME="navign"
export POSTGRES_URL="postgresql://user:password@localhost:5432/navign"

# Run migration binary
cargo run --bin migrate

# With options
cargo run --bin migrate -- --dry-run --skip-existing --batch-size 100
```

### Migration Process

The migration tool performs the following steps in order:

1. **Phase 1: Top-level entities** (no foreign keys)
   - Entities (MongoDB ObjectId → PostgreSQL UUID)
   - Users (MongoDB ObjectId → PostgreSQL UUID)

2. **Phase 2: Dependent entities**
   - Areas (MongoDB ObjectId → PostgreSQL Integer, references entities)
   - Merchants (MongoDB ObjectId → PostgreSQL Integer, references entities + areas)
   - Connections (MongoDB ObjectId → PostgreSQL Integer, references entities)

3. **Phase 3: Beacons**
   - Beacons (MongoDB ObjectId → PostgreSQL Integer, references entities, areas, merchants, connections)

4. **Phase 4: Related tables**
   - Beacon secrets
   - User public keys
   - Firmwares

### Migration Statistics

After migration completes, you'll see a summary:

```
=== Migration Summary ===
Entities:         42
Users:            150
Areas:            320
Beacons:          89
Merchants:        156
Connections:      28
Beacon Secrets:   89
User Public Keys: 245
Firmwares:        5
========================
Total migrated:   1124
Errors:           0
```

### Verify Migration

```bash
# Count records in PostgreSQL
psql $POSTGRES_URL -c "SELECT
    (SELECT COUNT(*) FROM entities) as entities,
    (SELECT COUNT(*) FROM users) as users,
    (SELECT COUNT(*) FROM areas) as areas,
    (SELECT COUNT(*) FROM beacons) as beacons,
    (SELECT COUNT(*) FROM merchants) as merchants,
    (SELECT COUNT(*) FROM connections) as connections"

# Compare with MongoDB counts
mongosh mongodb://localhost:27017/navign --eval "
    db.entities.countDocuments();
    db.users.countDocuments();
    db.areas.countDocuments();
    db.beacons.countDocuments();
    db.merchants.countDocuments();
    db.connections.countDocuments();
"
```

## Phase 3: Dual-Database Mode

Run the server with both databases to test PostgreSQL:

```bash
# Set both database URLs
export MONGODB_HOST="localhost:27017"
export DATABASE_NAME="navign"
export POSTGRES_URL="postgresql://user:password@localhost:5432/navign"

# Start server
cargo run --bin navign-server
```

The server will:
- Use PostgreSQL for reads/writes when available
- Fall back to MongoDB if PostgreSQL is not configured
- Log which database is being used for each operation

**Test API endpoints:**

```bash
# Get entities (will use PostgreSQL)
curl http://localhost:3000/api/entities

# Get specific entity
curl http://localhost:3000/api/entities/{uuid}

# Create entity (will use PostgreSQL)
curl -X POST http://localhost:3000/api/entities \
  -H "Content-Type: application/json" \
  -d '{
    "type": "Mall",
    "name": "Test Mall",
    "nation": "USA",
    "city": "Seattle",
    "longitude_range": [-122.4, -122.3],
    "latitude_range": [47.6, 47.7],
    "floors": []
  }'
```

## Phase 4: PostgreSQL-Only Mode

Once you're confident PostgreSQL is working correctly:

1. **Stop the server**

2. **Update environment variables** (remove MongoDB):

```bash
# Remove MongoDB variables
unset MONGODB_HOST
unset DATABASE_NAME

# Keep only PostgreSQL
export POSTGRES_URL="postgresql://user:password@localhost:5432/navign"
```

3. **Start server**:

```bash
cargo run --bin navign-server
```

4. **Optional: Archive MongoDB data**

```bash
# Create MongoDB backup
mongodump --host localhost:27017 --db navign --out mongodb_backup_$(date +%Y%m%d)

# Verify backup
ls -lh mongodb_backup_*/navign/
```

## Using Dual-Database API Handlers

The new dual-database handlers in `server/src/pg/handlers.rs` provide seamless fallback:

```rust
use crate::pg::handlers;

// In your routes:
.route("/api/entities", get(handlers::get_entities))
.route("/api/entities/:id", get(handlers::get_entity_by_id))
.route("/api/entities", post(handlers::create_entity))
.route("/api/entities", put(handlers::update_entity))
.route("/api/entities/:id", delete(handlers::delete_entity))
.route("/api/entities/:entity_id/areas", get(handlers::get_areas_by_entity))
.route("/api/entities/:entity_id/areas/:area_id", get(handlers::get_area_by_id))
.route("/api/entities/:entity_id/beacons", get(handlers::get_beacons_by_entity))
```

These handlers automatically:
- Check if PostgreSQL pool exists
- Use PostgreSQL if available
- Fall back to MongoDB otherwise
- Log which database is being used

## Troubleshooting

### Migration fails with "Entity not found in map"

This means a foreign key relationship is broken. Possible causes:
- Data corruption in MongoDB
- Orphaned records (area references non-existent entity)

**Solution**: Use `--skip-existing` flag and check MongoDB data consistency.

### PostgreSQL schema not initialized

```
Error: PostgreSQL schema not initialized. Please run migrations first.
```

**Solution**: Run migrations:

```bash
POSTGRES_RUN_MIGRATIONS=true cargo run --bin navign-server
```

### Connection refused errors

**MongoDB:**
```bash
# Check if MongoDB is running
systemctl status mongod
# Or
brew services list | grep mongodb

# Start MongoDB
systemctl start mongod
# Or
brew services start mongodb-community
```

**PostgreSQL:**
```bash
# Check if PostgreSQL is running
systemctl status postgresql
# Or
brew services list | grep postgresql

# Start PostgreSQL
systemctl start postgresql
# Or
brew services start postgresql
```

### PostGIS extension not available

```
ERROR: type "geometry" does not exist
```

**Solution**: Install PostGIS:

```bash
# Ubuntu/Debian
sudo apt-get install postgresql-14-postgis-3

# macOS
brew install postgis

# Then connect to your database and enable:
psql $POSTGRES_URL -c "CREATE EXTENSION IF NOT EXISTS postgis;"
```

### Duplicate key errors during migration

If migration fails with unique constraint violations:

**Option 1**: Use `--skip-existing` flag:
```bash
./scripts/migrate.sh --skip-existing
```

**Option 2**: Clear PostgreSQL and re-run:
```bash
# WARNING: This deletes all PostgreSQL data
psql $POSTGRES_URL -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
POSTGRES_RUN_MIGRATIONS=true cargo run --bin navign-server
./scripts/migrate.sh
```

## Performance Considerations

### Large Datasets

For databases with millions of records:

1. **Use batch migration**:
```bash
./scripts/migrate.sh --batch-size 1000
```

2. **Disable indexes temporarily** (PostgreSQL):
```sql
-- Before migration
ALTER TABLE beacons DISABLE TRIGGER ALL;
DROP INDEX idx_beacons_entity;
-- ... drop other indexes

-- After migration
CREATE INDEX idx_beacons_entity ON beacons(entity_id);
-- ... recreate other indexes
ALTER TABLE beacons ENABLE TRIGGER ALL;
```

3. **Monitor progress**:
```bash
# Watch PostgreSQL table sizes
watch -n 5 "psql $POSTGRES_URL -c \"
    SELECT schemaname, tablename,
           pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
    FROM pg_tables
    WHERE schemaname = 'public'
    ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC\""
```

### Network Latency

If MongoDB and PostgreSQL are on different servers:
- Run migration from a machine close to both databases
- Use `--batch-size` to tune network vs. transaction overhead
- Consider using `pg_dump` / `mongodump` for large datasets

## Rollback Plan

If you need to rollback to MongoDB:

1. **Stop the server**

2. **Update environment variables**:
```bash
export MONGODB_HOST="localhost:27017"
export DATABASE_NAME="navign"
unset POSTGRES_URL
```

3. **Start server** - it will use MongoDB only

4. **Restore MongoDB backup** (if needed):
```bash
mongorestore --host localhost:27017 --db navign mongodb_backup_*/navign/
```

## Next Steps

After successful migration:

1. **Monitor performance**: Compare query times between MongoDB and PostgreSQL
2. **Update documentation**: Update deployment docs to reflect PostgreSQL usage
3. **Archive MongoDB**: Keep MongoDB backup but decommission the instance
4. **Optimize PostgreSQL**: Run VACUUM, ANALYZE, and review query plans
5. **Update CI/CD**: Update deployment scripts to use PostgreSQL

## Additional Resources

- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [PostGIS Manual](https://postgis.net/docs/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [MongoDB to PostgreSQL Migration Best Practices](https://www.postgresql.org/docs/current/tutorial.html)

## Support

If you encounter issues:

1. Check logs: `RUST_LOG=debug cargo run --bin migrate`
2. Review error messages carefully
3. Consult `server/src/pg/repository.rs` for repository implementations
4. Check `server/migrations/001_initial_schema.sql` for schema definition

For questions or issues, please file a GitHub issue with:
- Migration command used
- Error messages
- Database versions (MongoDB, PostgreSQL)
- Approximate dataset size
