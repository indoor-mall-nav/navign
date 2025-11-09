# MongoDB to PostgreSQL Migration Guide

## Overview

This document describes the migration from MongoDB to PostgreSQL for the Navign server. The migration involves schema changes, database connection updates, and query refactoring.

## Key Changes

### 1. ID Types

**MongoDB** → **PostgreSQL**

- **Entities**: `ObjectId` → `UUID` (PostgreSQL UUID type)
- **Merchants**: `ObjectId` → `UUID` (PostgreSQL UUID type)
- **Connections**: `ObjectId` → `UUID` (PostgreSQL UUID type)
- **Users**: `ObjectId` → `UUID` (PostgreSQL UUID type)
- **Firmwares**: `ObjectId` → `UUID` (PostgreSQL UUID type)
- **Beacons**: `ObjectId` → `BIGSERIAL` (auto-incrementing integer)
- **Areas**: `ObjectId` → `BIGSERIAL` (auto-incrementing integer)

### 2. Geometry/Geography Data

**MongoDB (WKT strings)** → **PostgreSQL PostGIS**

- **Area polygons**: Text → `GEOMETRY(POLYGON, 4326)`
- **Beacon locations**: Text → `GEOMETRY(POINT, 4326)`
- **Merchant locations**: Text → `GEOMETRY(POINT, 4326)`
- **Connection ground points**: Text → `GEOMETRY(POINT, 4326)`

### 3. JSON/Array Fields

**MongoDB (native BSON)** → **PostgreSQL JSONB**

- **Tags**: `Vec<String>` → `JSONB` array
- **Merchant type**: Complex enum → `JSONB` object
- **Connected areas**: Custom type → `JSONB` array
- **Available periods**: `Vec<(i32, i32)>` → `JSONB` array
- **Social media**: Custom struct → `JSONB` array

### 4. Database Connection

**Before (MongoDB)**:
```rust
use mongodb::Database;

let db = mongodb::Client::with_options(options)?
    .database("navign");
```

**After (PostgreSQL)**:
```rust
use sqlx::PgPool;

let pool = PgPoolOptions::new()
    .max_connections(8)
    .connect(&database_url)
    .await?;
```

### 5. Shared Library Features

**Before**:
```toml
navign-shared = { features = ["std", "serde", "mongodb"] }
```

**After**:
```toml
navign-shared = { features = ["std", "serde", "sql"] }
```

## Environment Variables

### MongoDB (Old)
```bash
MONGODB_HOST=localhost:27017
MONGODB_DB_NAME=indoor-mall-nav
```

### PostgreSQL (New)
```bash
DATABASE_URL=postgres://username:password@localhost:5432/navign
```

## Migration Steps

### 1. Install PostgreSQL

```bash
# macOS
brew install postgresql postgis

# Linux
sudo apt-get install postgresql postgresql-contrib postgis

# Start PostgreSQL
brew services start postgresql  # macOS
sudo systemctl start postgresql # Linux
```

### 2. Create Database

```bash
psql postgres
CREATE DATABASE navign;
\c navign
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS postgis;
\q
```

### 3. Set Environment Variable

```bash
export DATABASE_URL="postgres://localhost:5432/navign"
```

Or create a `.env` file in the `server/` directory:
```
DATABASE_URL=postgres://localhost:5432/navign
```

### 4. Run Migrations

The migrations will run automatically when starting the server:

```bash
cd server
cargo run
```

Or manually:
```bash
cd server
sqlx migrate run --database-url postgres://localhost:5432/navign
```

### 5. Data Migration (if needed)

If you have existing MongoDB data, you'll need to migrate it manually. Here's a general approach:

1. Export data from MongoDB:
```bash
mongoexport --db=indoor-mall-nav --collection=entities --out=entities.json
```

2. Transform and import to PostgreSQL using a custom script (to be created).

## Database Schema

### Entities Table
```sql
CREATE TABLE entities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    longitude_min DOUBLE PRECISION NOT NULL,
    longitude_max DOUBLE PRECISION NOT NULL,
    latitude_min DOUBLE PRECISION NOT NULL,
    latitude_max DOUBLE PRECISION NOT NULL,
    altitude_min DOUBLE PRECISION,
    altitude_max DOUBLE PRECISION,
    nation VARCHAR(100),
    region VARCHAR(100),
    city VARCHAR(100),
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
```

### Areas Table (Incremental ID)
```sql
CREATE TABLE areas (
    id BIGSERIAL PRIMARY KEY,
    entity UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    beacon_code VARCHAR(50) NOT NULL,
    floor_type VARCHAR(20),
    floor_name INTEGER,
    polygon GEOMETRY(POLYGON, 4326) NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
```

### Beacons Table (Incremental ID)
```sql
CREATE TABLE beacons (
    id BIGSERIAL PRIMARY KEY,
    entity UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    area BIGINT NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    merchant UUID REFERENCES merchants(id) ON DELETE SET NULL,
    connection UUID REFERENCES connections(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    type VARCHAR(50) NOT NULL,
    location GEOMETRY(POINT, 4326) NOT NULL,
    device VARCHAR(20) NOT NULL,
    mac VARCHAR(17) NOT NULL UNIQUE,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
```

## Remaining Work

### High Priority

1. **Server Schema Implementations** (`server/src/schema/`)
   - Update `Service` trait to use sqlx queries instead of MongoDB
   - Update all handler functions to work with PostgreSQL
   - Update entity search to use SQL LIKE queries
   - Status: ⚠️ **Not Started**

2. **Authentication & User Management** (`server/src/kernel/auth/`)
   - Update user registration to use sqlx
   - Update user login queries
   - Update OAuth handlers
   - Status: ⚠️ **Not Started**

3. **Pathfinding** (`server/src/kernel/route/`)
   - Update area/connection queries to use PostGIS
   - Update graph building to use SQL joins
   - Optimize with spatial indexes
   - Status: ⚠️ **Not Started**

4. **Access Control** (`server/src/kernel/unlocker.rs`)
   - Update unlock instance creation
   - Update beacon secrets management
   - Status: ⚠️ **Not Started**

5. **Firmware Management** (`server/src/schema/firmware.rs`)
   - Update firmware CRUD operations
   - Status: ⚠️ **Not Started**

### Medium Priority

6. **Update CLAUDE.md Documentation**
   - Update database section
   - Update schema examples
   - Update environment variables

7. **Create SQL Query Helpers**
   - Geometry conversion utilities
   - JSONB serialization helpers
   - UUID parsing utilities

8. **Add Integration Tests**
   - Test database migrations
   - Test CRUD operations
   - Test spatial queries

### Low Priority

9. **Performance Optimization**
   - Add database indexes (already in migration)
   - Optimize complex queries
   - Add connection pooling tuning

10. **Monitoring & Logging**
    - Add PostgreSQL-specific metrics
    - Log slow queries
    - Monitor connection pool

## Benefits of PostgreSQL

1. **Strong ACID Guarantees**: Better data consistency
2. **PostGIS**: Native geospatial support with indexes
3. **JSONB**: Efficient JSON storage and querying
4. **Foreign Keys**: Enforced referential integrity
5. **UUID**: Native UUID type support
6. **Complex Queries**: Better JOIN and subquery performance
7. **Mature Ecosystem**: Better tooling and monitoring

## Breaking Changes

### API Responses

- Entity IDs now return UUIDs instead of 24-character hex strings
- Beacon/Area IDs now return integers instead of hex strings
- Date format remains unchanged (Unix timestamps in milliseconds)

### Mobile App

The mobile app uses SQLite locally and is unaffected by this server migration. The shared schema library supports both through feature flags:
- Server: `features = ["sql"]` (PostgreSQL)
- Mobile: Uses SQLite through the same `sql` feature

## Testing

### Unit Tests
```bash
cd server
cargo test
```

### Integration Tests (Coming Soon)
```bash
cd server
cargo test --test integration
```

### Manual Testing
```bash
# Start server
cd server
cargo run

# Test health endpoint
curl http://localhost:3000/health

# Test entity creation
curl -X POST http://localhost:3000/api/entities \
  -H "Content-Type: application/json" \
  -d '{
    "type": "mall",
    "name": "Test Mall",
    "longitude_range": [-122.5, -122.4],
    "latitude_range": [37.7, 37.8],
    "created_at": 1704672000000,
    "updated_at": 1704672000000
  }'
```

## Rollback Plan

If critical issues arise:

1. Revert to MongoDB by checking out previous commit
2. Restore MongoDB backup if needed
3. Update environment variables back to MongoDB

## Support

For questions or issues:
- Check PostgreSQL logs: `tail -f /usr/local/var/log/postgres.log`
- Check server logs for detailed errors
- Verify DATABASE_URL is correctly set
- Ensure PostgreSQL service is running

## Status

- ✅ Dependencies updated
- ✅ Database connection migrated
- ✅ Migration files created
- ✅ Shared schemas updated
- ✅ AppState updated
- ⚠️ Server handlers need updating
- ⚠️ Authentication needs updating
- ⚠️ Pathfinding needs updating
- ⚠️ Integration tests needed
