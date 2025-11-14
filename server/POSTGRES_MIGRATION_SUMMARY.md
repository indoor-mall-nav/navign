# PostgreSQL Migration Layer - Implementation Summary

## What Was Added

This implementation adds a complete PostgreSQL migration layer as an **intermediate layer** without touching any existing MongoDB logic.

## Files Added

### 1. SQL Migrations

**`migrations/001_initial_schema.sql`**
- Complete PostgreSQL schema definition
- UUID primary keys for `entities` and `users`
- Integer (SERIAL) primary keys for all other tables
- Foreign key constraints with CASCADE deletes
- Indexes for performance
- Triggers for automatic `updated_at` timestamps
- JSONB support for flexible data (floors, images, social media, etc.)

### 2. PostgreSQL Module (`src/pg/`)

**`src/pg/mod.rs`**
- Module organization and exports

**`src/pg/pool.rs`**
- `PgPool` wrapper around sqlx connection pool
- `create_pool()` function with connection configuration
- `run_migrations()` for automatic schema migration
- Connection pooling with configurable limits

**`src/pg/models.rs`**
- PostgreSQL-specific models with correct ID types:
  - `PgEntity` - UUID
  - `PgUser` - UUID
  - `PgArea` - i32
  - `PgBeacon` - i32
  - `PgMerchant` - i32
  - `PgConnection` - i32
  - `PgBeaconSecret` - i32
  - `PgUserPublicKey` - i32
  - `PgFirmware` - i32

**`src/pg/repository.rs`**
- Repository traits:
  - `Repository<T>` - Generic CRUD operations
  - `UuidRepository<T>` - UUID-specific operations
  - `IntRepository<T>` - Integer ID operations
- Repository implementations:
  - `EntityRepository` (UUID)
  - `UserRepository` (UUID)
  - `AreaRepository` (Integer)
  - Stubs for other repositories

### 3. Server Integration

**`src/main.rs`** (Modified)
- Added `pg` module import
- Updated `AppState` to include optional `pg_pool: Option<Arc<PgPool>>`
- Added PostgreSQL connection on startup (if `POSTGRES_URL` is set)
- Optional migration runner (if `POSTGRES_RUN_MIGRATIONS=true`)
- Graceful fallback to MongoDB-only if PostgreSQL not configured

**`src/error.rs`** (Modified)
- Added `DatabaseQuery(String)` error variant for PostgreSQL query errors
- Added `NotFound(String)` generic not-found error
- Updated error status code mappings

**`Cargo.toml`** (Modified)
- Updated sqlx dependency with full feature set:
  - `runtime-tokio`
  - `postgres`
  - `uuid`
  - `chrono`
  - `json`
  - `migrate`

### 4. Documentation

**`POSTGRES_MIGRATION.md`**
- Complete migration guide
- Architecture overview
- Schema design explanation
- Environment variable configuration
- Usage examples
- Migration strategy (4 phases)
- API compatibility notes
- Testing instructions
- Troubleshooting guide

**`.env.example`**
- Example environment variable configuration
- Shows both MongoDB (required) and PostgreSQL (optional) settings

## Key Design Decisions

### 1. ID Types

- **UUID for entities and users**: These are top-level resources that may need globally unique identifiers
- **Integer for everything else**: More efficient joins, foreign keys, and indexes

### 2. Non-Breaking Changes

- All existing MongoDB code remains unchanged
- PostgreSQL is completely optional
- Server works with MongoDB alone if no PostgreSQL URL is provided
- No API changes - existing clients continue to work

### 3. Repository Pattern

- Clean abstraction over database operations
- Trait-based design for flexibility
- Async/await throughout
- Type-safe ID handling (UUID vs Integer)

### 4. Feature Flags

- Full sqlx feature set enabled for PostgreSQL
- Migrations embedded in binary
- JSONB for complex nested data
- UUID extension enabled by default

## What Was NOT Changed

- ❌ No MongoDB code modified
- ❌ No existing API endpoints changed
- ❌ No existing service trait modified
- ❌ No database.rs file touched
- ❌ No schema/*.rs files changed (MongoDB models)

## Environment Variables

### Required (MongoDB)

```bash
MONGODB_HOST=localhost:27017
MONGODB_DB_NAME=indoor-mall-nav
```

### Optional (PostgreSQL)

```bash
POSTGRES_URL=postgresql://user:password@localhost:5432/navign
POSTGRES_RUN_MIGRATIONS=true  # Auto-run migrations on startup
```

### Other

```bash
SERVER_BIND_ADDR=0.0.0.0:3000
RUST_LOG=info
RATE_LIMIT_PER_SECOND=100
RATE_LIMIT_BURST_SIZE=200
```

## Usage Example

### Without PostgreSQL (Default)

```bash
cd server
cargo run
# Uses MongoDB only
```

### With PostgreSQL

```bash
# Set environment
export POSTGRES_URL="postgresql://localhost/navign"
export POSTGRES_RUN_MIGRATIONS=true

cd server
cargo run

# Logs will show:
# "PostgreSQL URL found, connecting to PostgreSQL..."
# "Successfully connected to PostgreSQL"
# "Running PostgreSQL migrations..."
# "PostgreSQL migrations completed successfully"
```

### Using Repositories

```rust
// In a handler
async fn my_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    // Check if PostgreSQL is available
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        // Use PostgreSQL
        let repo = EntityRepository::new(pg_pool.clone());
        let entities = repo.get_all(0, 10).await?;
        // ...
    } else {
        // Fall back to MongoDB (existing logic)
        let entities = Entity::get_all(&state.db).await?;
        // ...
    }

    // Return response
    Ok(Json(entities))
}
```

## Migration Strategy

This implementation enables a 4-phase migration:

1. **Phase 1 (Current)**: PostgreSQL layer exists but not used. MongoDB only.
2. **Phase 2**: Dual-write - Write to both databases, read from MongoDB
3. **Phase 3**: Dual-read - Write to both, read from PostgreSQL
4. **Phase 4**: PostgreSQL only - Remove MongoDB code

## Testing

### Compilation

```bash
cd server
cargo check  # ✓ Succeeds with warnings (unused code)
cargo build  # ✓ Builds successfully
```

### With PostgreSQL

```bash
# Start PostgreSQL
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:16

# Create database
psql -U postgres -c "CREATE DATABASE navign;"

# Run server with PostgreSQL
POSTGRES_URL="postgresql://postgres:postgres@localhost/navign" \
POSTGRES_RUN_MIGRATIONS=true \
cargo run
```

### Verify Tables

```sql
-- Connect to database
psql -U postgres navign

-- List tables
\dt

-- Expected tables:
-- entities, users, areas, beacons, merchants, connections
-- beacon_secrets, user_public_keys, firmwares

-- Check entity table
SELECT * FROM entities LIMIT 5;

-- Check ID types
\d entities  -- id should be UUID
\d areas     -- id should be integer
```

## Benefits

1. **Zero Risk**: Existing MongoDB logic untouched
2. **Optional**: Can run without PostgreSQL
3. **Gradual**: Can migrate at your own pace
4. **Type-Safe**: Strong typing with UUID and integers
5. **Performant**: Proper indexes and foreign keys
6. **Standards**: Uses standard SQL with JSONB for flexibility

## Future Work

### Next Steps

1. **Implement remaining repositories**:
   - BeaconRepository
   - MerchantRepository
   - ConnectionRepository
   - BeaconSecretRepository
   - UserPublicKeyRepository
   - FirmwareRepository

2. **Add data sync tool**:
   - Migrate existing MongoDB data to PostgreSQL
   - Verify data integrity
   - Handle ID conversions (ObjectId → UUID/Integer)

3. **Dual-write handlers**:
   - Update API handlers to write to both databases
   - Add configuration flag for dual-write mode

4. **Integration tests**:
   - Test repository CRUD operations
   - Test foreign key constraints
   - Test cascade deletes
   - Performance benchmarks

5. **Production hardening**:
   - Add retry logic
   - Add connection pooling monitoring
   - Add query logging
   - Add performance metrics

## Warnings in Compilation

The following warnings are expected and can be ignored:

- "struct PgMerchant is never constructed"
- "struct PgBeacon is never constructed"
- "struct PgConnection is never constructed"
- etc.

These structs will be used once the corresponding repositories are implemented.

## Maintenance

When adding new tables:

1. Create migration in `migrations/XXX_description.sql`
2. Add model in `src/pg/models.rs`
3. Add repository in `src/pg/repository.rs`
4. Update documentation

## License

MIT (same as project)

---

**Status**: ✅ Implementation Complete
**Compilation**: ✅ Success (with expected warnings)
**MongoDB**: ✅ Untouched
**API Compatibility**: ✅ 100% backward compatible
**Documentation**: ✅ Complete
