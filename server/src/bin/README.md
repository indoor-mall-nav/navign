# Server Binaries

This directory contains additional binaries for the Navign server.

## migrate

**Status:** 🚧 Work in Progress

The MongoDB to PostgreSQL migration tool is currently under development.

### Current Status

The migration tool has been scaffolded with:
- ✅ Basic structure and command-line argument parsing
- ✅ Database connection handling (MongoDB + PostgreSQL)
- ✅ Migration context with ID mapping
- ✅ Phase-based migration strategy
- ⚠️  **Requires schema mapping updates**

### Known Issues

The shared schema (`navign_shared`) has been refactored since the PostgreSQL migration layer was initially designed. The following mismatches need to be resolved:

1. **Entity Schema:**
   - Current: `tags`, `altitude_range`, `created_at`, `updated_at`
   - Expected by PostgreSQL: `address`, `floors`

2. **Area Schema:**
   - Current: `floor: Option<Floor>` where `Floor { r#type, name: u32 }`
   - Migration needs: Convert to string floor identifier

3. **Beacon Schema:**
   - Current: `device: BeaconDevice` where `BeaconDevice` is an enum (Esp32, Esp32C3, etc.)
   - Expected: `device` with `device_id`, `public_key`, `capabilities`, `unlock_method` fields
   - Field names: Uses `entity`, `area`, `merchant`, `connection` (not `*_id` versions)

4. **Merchant Schema:**
   - Current: Extensively refactored with new fields (`style`, `available_period`, `email`, `phone`, etc.)
   - Migration needs: Map to PostgreSQL schema columns

### Next Steps

To complete the migration tool:

1. **Update Entity Migration:**
   ```rust
   // Map Entity.tags -> text array or JSONB
   // Handle Entity.altitude_range (optional field)
   // Use Entity.created_at/updated_at (i64 timestamps)
   ```

2. **Update Area Migration:**
   ```rust
   // Convert Option<Floor> to string floor identifier
   // floor.map(|f| format!("{}", i32::from(f))).unwrap_or("0".to_string())
   ```

3. **Update Beacon Migration:**
   ```rust
   // Handle BeaconDevice enum -> extract device_id from enum variant
   // Beacon uses .entity, .area, .merchant, .connection (not *_id)
   ```

4. **Update Merchant Migration:**
   ```rust
   // Map new Merchant fields to PostgreSQL columns
   // Handle MerchantStyle enum Display implementation
   ```

5. **Add Schema Version Checking:**
   ```rust
   // Detect schema version mismatches before migration
   // Provide clear error messages for incompatible schemas
   ```

### Alternative Approach

Consider using a staged migration:

1. **Phase 1:** Migrate simple collections first (users, firmwares)
2. **Phase 2:** Create schema adapters in `shared/src/schema/adapters.rs`
3. **Phase 3:** Migrate complex collections with adapters
4. **Phase 4:** Add verification and rollback mechanisms

### Testing

Once schema mapping is complete, test with:

```bash
# Dry run to see migration plan
POSTGRES_URL=postgresql://localhost/navign cargo run --bin migrate -- --dry-run

# Migrate with skip-existing for incremental updates
POSTGRES_URL=postgresql://localhost/navign cargo run --bin migrate -- --skip-existing

# Full migration
POSTGRES_URL=postgresql://localhost/navign cargo run --bin migrate
```

### Documentation

See `/server/MIGRATION_GUIDE.md` for detailed migration guide once the tool is complete.

### Contributing

If you're updating this migration tool:

1. Update schema mappings in the `migrate_*` methods
2. Add tests for schema conversions
3. Update this README with current status
4. Update `/server/MIGRATION_GUIDE.md` with any new procedures
