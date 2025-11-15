# PostgreSQL Migration Layer

This document describes the PostgreSQL migration layer added to the Navign server. This is an **intermediate layer** that does not modify any existing MongoDB logic.

## Overview

The PostgreSQL migration layer provides:

1. **UUID-based IDs** for `entities` and `users` tables
2. **Integer-based IDs** (SERIAL) for all other tables (`areas`, `beacons`, `merchants`, `connections`, etc.)
3. **Clean repository pattern** for database operations
4. **Backward compatibility** - MongoDB code remains untouched

## Architecture

```
server/
├── migrations/                    # SQL migration files
│   └── 001_initial_schema.sql    # Initial schema with all tables
├── src/
│   ├── pg/                        # PostgreSQL layer (NEW)
│   │   ├── mod.rs                 # Module exports
│   │   ├── pool.rs                # Connection pool wrapper
│   │   ├── models.rs              # PostgreSQL models
│   │   └── repository.rs          # Repository implementations
│   ├── database.rs                # MongoDB connection (unchanged)
│   └── main.rs                    # Server entry point (updated)
```

## Schema Design

### UUID Tables

- **entities**: Main building entities (malls, hospitals, etc.)
- **users**: User accounts

### Integer Tables

All other tables use `SERIAL` (auto-incrementing integers):

- **areas**: Physical areas within entities
- **beacons**: BLE beacon devices
- **merchants**: Shops, stores, facilities
- **connections**: Inter-area connections (elevators, stairs)
- **beacon_secrets**: Private keys for beacons
- **user_public_keys**: User device public keys
- **firmwares**: Firmware versions

## Environment Variables

Add these to your `.env` file or environment:

```bash
# PostgreSQL connection (optional)
POSTGRES_URL=postgresql://user:password@localhost:5432/navign

# Run migrations on startup (optional, default: false)
POSTGRES_RUN_MIGRATIONS=true

# MongoDB connection (still required)
MONGODB_HOST=localhost:27017
MONGODB_DB_NAME=indoor-mall-nav
```

## Usage

### 1. Running with PostgreSQL

```bash
# Set environment variables
export POSTGRES_URL="postgresql://localhost/navign"
export POSTGRES_RUN_MIGRATIONS=true

# Start server
cd server
cargo run
```

### 2. Running Migrations Manually

```bash
# Using sqlx CLI
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
cd server
sqlx migrate run --database-url postgresql://localhost/navign
```

### 3. Using Repositories in Code

```rust
use crate::pg::{PgPool, EntityRepository, Repository};

// Get pool from AppState
let pg_pool = state.pg_pool.as_ref().ok_or(...)?;

// Create repository
let repo = EntityRepository::new(pg_pool.clone());

// CRUD operations
let entities = repo.get_all(0, 10).await?;
let entity = repo.get_by_id("uuid-string").await?;
let new_id = repo.create(&entity).await?;
repo.update(&entity).await?;
repo.delete("uuid-string").await?;
```

## Migration Strategy

The PostgreSQL layer is designed to coexist with MongoDB:

### Phase 1: Dual Write (Current)

- All writes go to MongoDB (existing logic)
- Optionally mirror writes to PostgreSQL
- Reads come from MongoDB

### Phase 2: Dual Read

- Write to both databases
- Read from both and compare results
- Log discrepancies

### Phase 3: PostgreSQL Primary

- Write to PostgreSQL first
- Mirror to MongoDB for safety
- Read from PostgreSQL

### Phase 4: MongoDB Removal

- Remove MongoDB code
- PostgreSQL becomes sole database

## API Changes

**None!** The existing REST API remains unchanged. The migration layer is transparent to clients.

## Testing

### Unit Tests

```bash
# Test with PostgreSQL
cd server
POSTGRES_URL="postgresql://localhost/navign_test" cargo test
```

### Integration Tests

1. Start PostgreSQL:
   ```bash
   docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:16
   ```

2. Create database:
   ```bash
   psql -U postgres -c "CREATE DATABASE navign;"
   ```

3. Run server:
   ```bash
   POSTGRES_URL="postgresql://postgres:postgres@localhost/navign" \
   POSTGRES_RUN_MIGRATIONS=true \
   cargo run
   ```

## Monitoring

Check if PostgreSQL is connected:

```bash
# Server logs will show:
# "PostgreSQL URL found, connecting to PostgreSQL..."
# "Successfully connected to PostgreSQL"
# "PostgreSQL migrations completed successfully"

# If no PostgreSQL URL:
# "No PostgreSQL URL configured, using MongoDB only"
```

## Schema Differences

### MongoDB → PostgreSQL Mapping

| MongoDB           | PostgreSQL       | Notes                          |
|-------------------|------------------|--------------------------------|
| ObjectId          | UUID (entities)  | For entities and users         |
| ObjectId          | SERIAL (others)  | For areas, beacons, etc.       |
| Embedded objects  | JSONB            | For floors, images, etc.       |
| References        | Foreign Keys     | With CASCADE delete            |
| No schema         | Strict schema    | All fields defined             |
| Flexible types    | Strong types     | Type safety enforced           |

## Repository Pattern

Each table has a repository implementing the `Repository` trait:

```rust
#[async_trait]
pub trait Repository<T> {
    async fn get_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<T>>;
    async fn create(&self, entity: &T) -> Result<String>;
    async fn update(&self, entity: &T) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn count(&self) -> Result<i64>;
}
```

UUID tables also implement:

```rust
#[async_trait]
pub trait UuidRepository<T>: Repository<T> {
    async fn get_by_uuid(&self, id: Uuid) -> Result<Option<T>>;
}
```

Integer tables also implement:

```rust
#[async_trait]
pub trait IntRepository<T>: Repository<T> {
    async fn get_by_int(&self, id: i32) -> Result<Option<T>>;
}
```

## Performance

PostgreSQL offers:

- **Better query optimization** for complex joins
- **Full-text search** built-in
- **JSONB indexes** for flexible data
- **Foreign key constraints** for data integrity
- **Transactions** with ACID guarantees

## Security

- Passwords are still hashed with bcrypt
- Private keys stored as BYTEA
- Row-level security can be added later
- SSL/TLS supported via connection URL

## Troubleshooting

### "Failed to connect to PostgreSQL"

- Check POSTGRES_URL is correct
- Verify PostgreSQL is running
- Test connection: `psql $POSTGRES_URL`

### "Failed to run PostgreSQL migrations"

- Check migrations/ directory exists
- Verify migration files are valid SQL
- Check database permissions

### "Invalid UUID format"

- Entities and users must use UUID strings
- MongoDB ObjectIds need conversion

### "Invalid integer ID format"

- Areas, beacons, etc. expect integer IDs
- Check API is sending correct type

## Future Enhancements

1. **Data Sync Tool**: Migrate existing MongoDB data to PostgreSQL
2. **GraphQL API**: Add GraphQL layer on PostgreSQL
3. **Read Replicas**: Set up PostgreSQL read replicas
4. **Partitioning**: Partition large tables by entity_id
5. **Full-Text Search**: Add tsvector columns for search
6. **PostGIS**: Add geospatial queries for positioning

## Contributing

When adding new tables:

1. Create migration in `migrations/`
2. Add model in `pg/models.rs`
3. Create repository in `pg/repository.rs`
4. Update this README

## License

Same as main project (MIT)
