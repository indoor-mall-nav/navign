# PostgreSQL Handlers Implementation

## Overview

This directory contains PostgreSQL-based API handlers that provide dual-database support, allowing the server to use either PostgreSQL or MongoDB (fallback) for data operations.

## Files

- `handlers.rs` - CRUD handlers for all major entities (Entity, Area, Beacon, Merchant, Connection)
- `auth_handlers.rs` - Authentication handlers (register, login) with PostgreSQL support
- `route_handlers.rs` - Complex pathfinding handler with manual SQL queries
- `adapters.rs` - Conversion functions between PostgreSQL models and shared schemas
- `repository.rs` - PostgreSQL repository implementations with SQL queries
- `models.rs` - PostgreSQL-specific data models

## Implementation Status

### ✅ Fully Implemented

- **Entity CRUD** - All operations (GET, POST, PUT, DELETE)
- **Area READ** - Get operations
- **Beacon READ** - Get operations
- **Merchant READ** - Get operations
- **Connection READ** - Get operations
- **Authentication** - Register and login with PostgreSQL
- **Pathfinding** - Complex route finding with manual SQL queries

### ⚠️ Partial Implementation

- **Area CREATE/UPDATE** - Adapter requires entity_id extraction
- **Beacon CREATE/UPDATE** - Adapter requires entity_id, area_id, floor, etc.
- **Merchant CREATE/UPDATE** - Adapter requires entity_id and area_id
- **Connection CREATE/UPDATE** - Adapter requires entity_id

### Known Limitations

1. **ID Extraction Challenge**: Create/update operations require extracting UUID/integer IDs from string representations, which adds complexity.

2. **Missing Repository Method**: UserRepository's `create` method returns a `String` (UUID), but we need to handle this in auth handlers.

3. **Schema Conversion**: Some fields in MongoDB schemas don't exist in PostgreSQL schemas and vice versa, requiring placeholder values.

## Usage Examples

### Enabling PostgreSQL

Set environment variables:

```bash
POSTGRES_URL=postgresql://user:password@localhost:5432/navign
POSTGRES_RUN_MIGRATIONS=true
```

### Dual-Database Pattern

All handlers follow this pattern:

```rust
pub async fn my_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        // PostgreSQL implementation with manual SQL
        let repo = MyRepository::new(pg_pool.as_ref().clone());
        let result = repo.get_by_id(&id).await?;
        Ok(Json(result))
    } else {
        // MongoDB fallback
        let result = MyEntity::get_one_by_id(&state.db, &id).await?;
        Ok(Json(result))
    }
}
```

### Complex Query Example (Pathfinding)

```rust
// Manual SQL query for fetching related data
let pg_areas = sqlx::query_as::<_, PgArea>(
    "SELECT * FROM areas WHERE entity_id = $1 ORDER BY name",
)
.bind(entity_uuid)
.fetch_all(pg_pool.inner())
.await?;
```

## Integration with main.rs

### Current Approach

The server uses MongoDB Service trait handlers by default. To switch to PostgreSQL handlers, routes in `main.rs` need to be updated:

```rust
// Current (MongoDB Service trait)
.route("/api/entities", get(Entity::get_handler))

// Future (PostgreSQL dual-database)
.route("/api/entities", get(pg::handlers::get_entities))
```

### Migration Strategy

1. **Phase 1** (Current): PostgreSQL handlers exist but aren't wired into routes
2. **Phase 2**: Wire PostgreSQL handlers into routes, test dual-database mode
3. **Phase 3**: Fix create/update operations with proper ID handling
4. **Phase 4**: Remove MongoDB dependencies once PostgreSQL is stable

## Next Steps

1. **Fix Create/Update Operations**:
   - Extract entity_id from request context
   - Handle UUID/Integer ID conversions properly
   - Update adapters to handle missing context

2. **Wire Handlers into Routes**:
   - Update `main.rs` to use PostgreSQL handlers
   - Add feature flags for database selection

3. **Add Tests**:
   - Unit tests for adapter conversions
   - Integration tests for dual-database mode
   - End-to-end API tests

4. **Documentation**:
   - API migration guide
   - Database schema comparison
   - Performance benchmarks

## Contributing

When adding new handlers:

1. Follow the dual-database pattern (check `pg_pool`, fallback to MongoDB)
2. Use manual SQL queries for complex operations
3. Add proper error handling with `ServerError` types
4. Update this README with implementation status
5. Add tests for both PostgreSQL and MongoDB paths

## References

- PostgreSQL repository: `server/src/pg/repository.rs`
- MongoDB Service trait: `server/src/schema/service.rs`
- Existing pathfinding: `server/src/kernel/route/mod.rs`
- Authentication: `server/src/kernel/auth/handlers.rs`
