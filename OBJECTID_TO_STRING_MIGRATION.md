# ObjectID to String Migration Plan

## Problem

Current implementation converts PostgreSQL IDs (UUID/integer) to `bson::oid::ObjectId`, which is:
- A hack/workaround
- Not type-safe
- Couples PostgreSQL data to MongoDB types
- Requires helper functions like `uuid_to_object_id()` and `int_to_object_id()`

## Proposed Solution

**Change all ID fields in `navign_shared` schemas from `ObjectId` to `String`**

This allows:
- PostgreSQL to use UUID strings directly (e.g., `"550e8400-e29b-41d4-a716-446655440000"`)
- PostgreSQL to use integer strings (e.g., `"123"`)
- MongoDB to use ObjectId hex strings (e.g., `"507f1f77bcf86cd799439011"`)

## Files Requiring Changes

### Shared Schemas (navign_shared)

All these files need `pub id: ObjectId` → `pub id: String`:

1. `shared/src/schema/entity.rs`
2. `shared/src/schema/area.rs`
3. `shared/src/schema/beacon.rs`
4. `shared/src/schema/merchant.rs`
5. `shared/src/schema/connection.rs`
6. `shared/src/schema/account.rs`
7. `shared/src/schema/firmware.rs`

### Server MongoDB Code

All MongoDB code needs to convert ObjectId ↔ String:

```rust
// Before
let entity = Entity {
    id: ObjectId::new(),
    // ...
};

// After
let entity = Entity {
    id: ObjectId::new().to_hex(),
    // ...
};
```

When querying:
```rust
// Before
collection.find(doc! { "_id": ObjectId::parse_str(&id)? })

// After
collection.find(doc! { "_id": ObjectId::parse_str(&id)? })
// (ObjectId::parse_str still works with hex strings)
```

### PostgreSQL Adapters

Adapters become simpler:

```rust
// Before
pub fn pg_entity_to_entity(pg: PgEntity) -> Entity {
    Entity {
        id: uuid_to_object_id(pg.id),  // Conversion hack
        entity: uuid_to_object_id(pg.entity_id),
        // ...
    }
}

// After
pub fn pg_entity_to_entity(pg: PgEntity) -> Entity {
    Entity {
        id: pg.id.to_string(),  // Direct String
        entity: pg.entity_id.to_string(),
        // ...
    }
}
```

## Impact Assessment

### ✅ Benefits

1. **Type honesty**: String IDs can represent any ID format
2. **Database independence**: No coupling to MongoDB types
3. **Simpler code**: No conversion functions needed
4. **Flexibility**: Can switch between UUID, integer, ObjectId, etc.

### ⚠️ Breaking Changes

1. **Mobile App**: TypeScript types will change from MongoDB ObjectId strings to generic strings
2. **API Responses**: ID format changes (but still valid strings)
3. **Database Queries**: MongoDB code needs ObjectId conversion
4. **Serialization**: BSON serialization changes

### 🔧 Migration Steps

#### Phase 1: Update Shared Schemas (Breaking)
- Change all `pub id: ObjectId` to `pub id: String`
- Change all foreign key fields (`entity`, `area`, etc.) to `String`
- Update derive macros if needed

#### Phase 2: Update Server MongoDB Code
- Add `.to_hex()` when creating entities
- Add `ObjectId::parse_str()` when querying
- Update all Service trait implementations

#### Phase 3: Update PostgreSQL Code
- Simplify adapters (remove conversion functions)
- Use `.to_string()` for UUIDs and integers

#### Phase 4: Update Mobile App
- TypeScript types auto-regenerate with String IDs
- Update any ID comparison logic
- Test UUID/integer string handling

#### Phase 5: Update Tests
- Fix all tests expecting ObjectId format
- Add tests for mixed ID formats

## Alternative Approaches

### Option A: Generic ID Type
```rust
pub struct Entity<ID = ObjectId> {
    pub id: ID,
    // ...
}
```
**Pros**: Type-safe
**Cons**: Complex generics everywhere, affects API

### Option B: Feature Flags
```rust
#[cfg(feature = "mongodb")]
pub id: ObjectId,
#[cfg(feature = "postgres")]
pub id: String,
```
**Pros**: Compile-time selection
**Cons**: Can't have both databases at runtime

### Option C: Keep Current Approach (Not Recommended)
- Continue using ObjectId with conversion functions
- User explicitly rejected this

## Recommendation

**Proceed with String migration (Proposed Solution)**

This is the cleanest approach that:
- Satisfies user requirement ("modify the data structure")
- Works with dual-database setup
- Provides maximum flexibility
- Has manageable migration path

## Estimated Effort

- **Shared schemas**: 2 hours (7 files)
- **Server MongoDB code**: 4 hours (20+ files)
- **PostgreSQL adapters**: 1 hour (simplification)
- **Mobile TypeScript**: 1 hour (auto-generated)
- **Testing**: 2 hours
- **Total**: ~10 hours

## Risk Mitigation

1. **Create migration branch** (done: `claude/postgres-crud-operations-...`)
2. **Update in phases** (schemas → server → mobile)
3. **Keep MongoDB-only mode working** throughout
4. **Comprehensive testing** after each phase
5. **Document breaking changes** in changelog

## Decision Needed

**Should I proceed with this migration?**

- [ ] Yes, migrate all schemas to use String IDs
- [ ] No, keep current ObjectId approach with conversions
- [ ] Alternative approach (please specify)
