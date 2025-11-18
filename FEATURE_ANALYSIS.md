# Shared Library Feature Flag Analysis Report

**Analysis Date:** 2025-11-18  
**Focus:** Analyzing feature flags in `/home/user/navign/shared/Cargo.toml`

---

## Executive Summary

The shared library has **8 optional feature flags** with varying levels of usage:

| Feature | Status | Used By | Recommendation |
|---------|--------|---------|-----------------|
| **mongodb** | ❌ UNUSED | server only | **REMOVE** |
| **base64** | ✅ USED | firmware, mobile | **KEEP** |
| **chrono** | ✅ USED | postgres feature | **KEEP** |
| **crypto** | ✅ USED | firmware | **KEEP** |
| **geo** | ✅ USED | server, mobile | **KEEP** |
| **postgres** | ✅ USED | server | **KEEP** |
| **sql** | ✅ USED | mobile | **KEEP** |
| **ts-rs** | ✅ USED | TypeScript generation | **KEEP** |

---

## Detailed Feature Analysis

### 1. **mongodb** ❌ UNUSED - RECOMMEND REMOVAL

#### Feature Definition (Cargo.toml)
```toml
mongodb = ["alloc", "serde", "dep:bson"]
```

#### Dependencies Enabled
- `bson 2.15.0` (BSON serialization)
- `alloc` (heap allocation)
- `serde` (serialization)

#### Usage in Shared
- **No `#[cfg(feature = "mongodb")]` gates found**
- **No `use bson` or `bson::` imports**
- **No BSON types used anywhere**

#### Used By
- **Server** (Cargo.toml line 35): requests "mongodb" feature
- **Firmware** (Cargo.toml line 81-87): does NOT request "mongodb"
- **Mobile** (Cargo.toml line 67-72): does NOT request "mongodb"
- **Orchestrator** (Cargo.toml line 9-13): does NOT request "mongodb"

#### Server-Side Usage
The server uses MongoDB directly (not via shared):
- `/home/user/navign/server/src/kernel/unlocker/instance.rs`: uses `mongodb::Database`
- `/home/user/navign/server/src/kernel/auth/`: uses `mongodb::Database` 
- Server imports `mongodb` crate directly (line 30 in server/Cargo.toml)

**Key Finding:** The server declares `navign-shared` with `"mongodb"` feature (line 35), but:
1. Shared library doesn't use BSON or any mongodb feature gates
2. Server doesn't rely on mongodb feature from shared - it imports mongodb directly
3. This is **cargo cult dependency** - feature requested but never used

#### Recommendation
**REMOVE** the mongodb feature flag from shared/Cargo.toml:
- Not used in any code
- Adds unnecessary dependency weight
- Creates confusion about what's actually needed

---

### 2. **base64** ✅ USED - KEEP

#### Feature Definition (Cargo.toml)
```toml
base64 = ["dep:base64", "alloc"]
```

#### Dependencies Enabled
- `base64 0.22.1` (base64 encoding/decoding)
- `alloc` (heap allocation required)

#### Code Usage

**File: `/home/user/navign/shared/src/traits/packetize.rs`**
- **Lines 52-56** (alloc variant): `packetize_to_base64()` method
  ```rust
  #[cfg(feature = "base64")]
  fn packetize_to_base64(&self) -> alloc::string::String {
      use base64::Engine;
      let packet = self.packetize();
      base64::engine::general_purpose::STANDARD.encode(&packet)
  }
  ```

**File: `/home/user/navign/shared/src/traits/depacketize.rs`**
- **Lines 6-14** (depacketize variant): `depacketize_from_base64()` method
  ```rust
  #[cfg(feature = "base64")]
  fn depacketize_from_base64(b64: &str) -> Option<Self> {
      use base64::Engine;
      let decoded = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
      Self::depacketize(&decoded)
  }
  ```

#### Used By
- **Firmware** (Cargo.toml): does NOT explicitly request "base64"
  - But uses Packetize/Depacketize traits which include the methods
  - Can encode/decode BLE messages to base64 if needed
  
- **Mobile** (Cargo.toml): does NOT explicitly request "base64"
  - Uses same trait-based API
  - Can encode/decode to base64

#### Current Status
- Feature is properly gated with `#[cfg(feature = "base64")]`
- Methods are conditional - won't bloat binary if not requested
- Useful for optional base64 encoding of BLE messages

#### Recommendation
**KEEP** - Feature works as intended, properly gated, adds optional functionality

---

### 3. **chrono** ✅ USED - KEEP

#### Feature Definition (Cargo.toml)
```toml
chrono = ["dep:chrono"]
```

#### Dependencies Enabled
- `chrono 0.4.42` (date/time handling) - **optional**

#### Code Usage

**File: `/home/user/navign/shared/src/schema/account.rs`**
- **Lines 28, 40** (with postgres feature):
  ```rust
  #[cfg(feature = "postgres")]
  pub created_at: Option<chrono::DateTime<chrono::Utc>>,
  #[cfg(feature = "postgres")]
  pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
  ```

**File: `/home/user/navign/shared/src/schema/postgres.rs`**
- Lines 50, 52, 74, 76, 97, 99, 126, 128, 161, 163, 181, 183, 199, 201, 212, 214, 228, 230
- Used throughout all PostgreSQL model structs for timestamp fields

#### Feature Coupling

In Cargo.toml, the **postgres feature** includes chrono:
```toml
postgres = [
  ...,
  "chrono",  # Line 44
  ...
]
```

This means:
- `chrono` is only included when `postgres` is enabled
- Cannot use chrono without postgres in this crate
- This is intentional coupling - postgres models need chrono timestamps

#### Used By
- **Server** (Cargo.toml line 32-39): requests `postgres` which includes `chrono`
- **Firmware**: does NOT request postgres
- **Mobile**: requests `sql` which does NOT include postgres/chrono
- **Orchestrator**: does NOT request postgres

#### Recommendation
**KEEP** - Feature properly couples with postgres, used for timestamp handling in PostgreSQL models

**Note:** chrono is NOT enabled when using MongoDB or SQLite - timestamps are i64 (milliseconds) in those cases (see account.rs lines 34, 46)

---

### 4. **crypto** ✅ USED - KEEP

#### Feature Definition (Cargo.toml)
```toml
crypto = ["dep:sha2", "dep:hmac", "dep:p256"]
```

#### Dependencies Enabled
- `sha2 0.10.9` (SHA-256 hashing)
- `hmac 0.12.1` (HMAC authentication)
- `p256 0.13.2` (P-256 ECDSA elliptic curve)

#### Code Usage

**File: `/home/user/navign/shared/src/traits/packetize.rs`**

Heapless variant (lines 13-45):
```rust
#[cfg(feature = "crypto")]
fn get_hash(&self) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    // Hash computation
}

#[cfg(feature = "crypto")]
fn sign(&self, signing_key: &p256::ecdsa::SigningKey) -> [u8; 64] {
    use p256::ecdsa::signature::Signer;
    // P-256 ECDSA signing
}

#[cfg(feature = "crypto")]
fn verify(&self, verify_key: &p256::ecdsa::VerifyingKey, signature: &[u8; 64]) -> bool {
    use p256::ecdsa::signature::Verifier;
    // P-256 ECDSA verification
}
```

Alloc variant (lines 59-91):
- Same methods available for heap-allocated variant
- Identical implementation

#### Used By
- **Firmware** (Cargo.toml line 81-87): explicitly requests `"crypto"`
  - Used for BLE message signing/verification
  - Beacon authentication with mobile devices
  - Nonce challenge-response security

- **Server** (Cargo.toml line 32-39): 
  - Requests `postgres`, `geo`, `sql` but NOT explicitly `crypto`
  - Server has its own p256 import (line 33 in server/Cargo.toml)
  - Doesn't rely on shared's crypto feature

- **Mobile** (Cargo.toml line 67-72): does NOT request `"crypto"`

- **Orchestrator**: does NOT request `"crypto"`

#### Functional Purpose

In firmware/beacon:
- Signs BLE responses with beacon's private key
- Verifies signatures from mobile app
- Essential for access control security

#### Recommendation
**KEEP** - Feature is actively used by firmware, properly gated, essential for security

---

### 5. **geo** ✅ USED - KEEP

#### Feature Definition (Cargo.toml)
```toml
geo = ["dep:geo", "dep:earcutr", "dep:wkt", "dep:wkb", "dep:geo-traits", "dep:geo-types", "alloc"]
```

#### Dependencies Enabled
- `geo 0.29.3` (geographic operations)
- `earcutr 0.5.0` (polygon triangulation - Earcut algorithm)
- `wkt 0.14.0` (Well-Known Text format)
- `wkb 0.9.1` (Well-Known Binary format)
- `geo-traits 0.3.0` (geometry trait definitions)
- `geo-types 0.7` (geometry types)
- `alloc` (heap allocation required)

#### Code Usage

**File: `/home/user/navign/shared/src/pathfinding/polygon.rs`**

Lines 69-73: Imports when geo feature enabled
```rust
#[cfg(feature = "geo")]
use geo::{BoundingRect, Contains, Coord, LineString, Point, Polygon as GeoPolygon};

#[cfg(feature = "geo")]
use earcutr;
```

Throughout file (lines 156, 208, 304, 364-550, 637-763):
- 24 `#[cfg(feature = "geo")]` gates
- Used for:
  - Triangulation-based pathfinding (Earcut algorithm)
  - Polygon bounds checking
  - Point-in-polygon tests
  - Visibility graph construction

**File: `/home/user/navign/shared/src/pathfinding/mod.rs`**
- Line 20: Re-exports triangulation mesh when geo enabled
  ```rust
  #[cfg(feature = "geo")]
  pub use polygon::TriangulationMesh;
  ```

**File: `/home/user/navign/shared/src/schema/postgis.rs`**

Lines 6-11: PostGIS support
```rust
#[cfg(all(feature = "postgres", feature = "geo"))]
use geo_traits::to_geo::{ToGeoPoint, ToGeoPolygon};

#[cfg(feature = "postgres")]
use geo_types::{Point, Polygon};
```

Lines 50-266: PostGIS point/polygon serialization for PostgreSQL

#### Used By
- **Server** (Cargo.toml line 32-39): explicitly requests `"geo"`
  - Advanced pathfinding with triangulation
  - PostgreSQL PostGIS geometry handling
  - Polygon operations for navigation

- **Mobile** (Cargo.toml line 67-72): NOT explicitly listed BUT inherits through `sql`
  - Actually: Mobile requests `sql`, not `geo` directly
  - But can receive triangulation meshes from server

- **Firmware**: does NOT request "geo" (not needed in embedded)

- **Orchestrator**: does NOT request "geo"

#### Pathfinding Usage

**Grid-based (no feature required):**
- Basic Manhattan-style pathfinding with rectangular blocks
- Works with heapless/embedded

**Triangulation-based (requires `geo` feature):**
- For irregular polygon shapes
- Earcut algorithm for fast triangulation
- More natural paths around obstacles
- Used in server for complex floor plans

#### Recommendation
**KEEP** - Feature properly used by server for advanced pathfinding and PostGIS support

---

### 6. **postgres** ✅ USED - KEEP

#### Feature Definition (Cargo.toml)
```toml
postgres = [
  "std",
  "serde",
  "geo",
  "dep:sqlx",
  "dep:async-trait",
  "sqlx?/postgres",
  "sqlx?/uuid",
  "sqlx?/chrono",
  "sqlx?/json",
  "dep:uuid",
  "dep:serde_json",
  "chrono",
]
```

#### Dependencies Enabled
- `sqlx 0.8.6` (with postgres driver, uuid, chrono, json support)
- `async-trait 0.1.89` (async trait support)
- `uuid 1.18.1` (UUID type for entity IDs)
- `serde_json 1.0` (JSON serialization)
- `chrono 0.4.42` (for timestamp fields)
- `geo` feature (PostGIS support)
- `std`, `serde` (standard library and serialization)

#### Code Usage

**File: `/home/user/navign/shared/src/schema/postgres.rs` (156 lines)**

Provides PostgreSQL-specific models:
- `PgEntity` (line 34-58): UUID-based entity with PostGIS point fields
- `PgUser` (line 59-76): UUID-based user accounts
- `PgArea` (line 77-108): Integer-based areas with PostGIS polygons
- `PgBeacon` (line 109-140): Integer-based beacons
- `PgConnection` (line 141-169): Integer-based connections
- `PgMerchant` (line 170-202): Integer-based merchants

All use sqlx `FromRow` derive, UUID types, chrono timestamps

**File: `/home/user/navign/shared/src/schema/postgis.rs` (276 lines)**

PostGIS geometry wrapper types:
- `PgPoint` (lines 24-210): Point wrapper for GEOMETRY(POINT, 4326)
- `PgPolygon` (lines 211-274): Polygon wrapper for PostGIS polygons
- Implements sqlx `Type`, `Encode`, `Decode` traits for WKB serialization

**File: `/home/user/navign/shared/src/schema/connection.rs`**

Lines 9, 27, 45, 49, 61: PostgreSQL-gated fields
```rust
#[cfg(feature = "postgres")]
pub struct PgConnection {
    pub id: i32,
    // ...
}
```

**File: `/home/user/navign/shared/src/schema/entity.rs`**

PostgreSQL struct definition (lines 11-46)

**File: `/home/user/navign/shared/src/schema/merchant.rs`**

PostgreSQL struct definition (lines 6-64)

**File: `/home/user/navign/shared/src/schema/beacon_secrets.rs`**

PostgreSQL model (lines 18-49)

**File: `/home/user/navign/shared/src/schema/user_public_keys.rs`**

PostgreSQL model (lines 13-48)

**File: `/home/user/navign/shared/src/schema/account.rs`**

PostgreSQL Account variant (lines 14-15, 23-40)

**File: `/home/user/navign/shared/src/schema/firmware.rs`**

PostgreSQL firmware models (lines 16-49)

**File: `/home/user/navign/shared/src/schema/repository.rs`**

PostgreSQL repository trait implementations:
- `UuidRepository` trait for UUID-based entities
- `IntRepository` trait for Integer-based entities
- `IntRepositoryInArea` for area-scoped queries
- Async methods using sqlx::PgPool

#### Used By
- **Server** (Cargo.toml line 32-39): explicitly requests `"postgres"`
  - Uses PostgreSQL models for database operations
  - Dual-database support (MongoDB + PostgreSQL)
  - Full CRUD repository implementations

- **Mobile**: does NOT request "postgres"
  - Uses SQLite via `sql` feature
  - Doesn't need PostgreSQL models

- **Firmware**: does NOT request "postgres"

- **Orchestrator**: does NOT request "postgres"

#### Migration Status
PostgreSQL layer is **fully implemented** (from CLAUDE.md):
- Phase 1 (Current): Layer exists, MongoDB is primary
- Phase 2 (Upcoming): Dual-write mode
- Phase 3 (Future): Dual-read mode  
- Phase 4 (Final): PostgreSQL only

#### Recommendation
**KEEP** - Feature is actively used, fully implemented, essential for PostgreSQL migration strategy

---

### 7. **sql** ✅ USED - KEEP

#### Feature Definition (Cargo.toml)
```toml
sql = ["std", "serde", "geo", "dep:sqlx", "dep:async-trait", "dep:uuid", "dep:serde_json"]
```

#### Dependencies Enabled
- `sqlx 0.8.6` (SQLite + PostgreSQL support)
- `async-trait 0.1.89` (async trait support)
- `uuid 1.18.1` (UUID type)
- `serde_json 1.0` (JSON serialization)
- `geo` (geographic types for PostGIS)
- `std`, `serde` (standard library and serialization)

#### Code Usage

**File: `/home/user/navign/shared/src/schema/repository.rs` (36 lines)**

Defines repository traits used by both SQLite and PostgreSQL:
```rust
#[cfg(feature = "sql")]
pub use repository::{IntRepository, IntRepositoryInArea, UuidRepository};
```

- `UuidRepository`: For UUID-based entities (users, accounts)
- `IntRepository`: For integer-based entities (areas, beacons, merchants)
- `IntRepositoryInArea`: Scoped queries within an area

All async methods using sqlx:
- `create()`, `get_by_uuid()`, `update()`, `delete()`, `list()`, `search()`

**File: `/home/user/navign/shared/src/schema/area.rs`**

Line 14: FromRow implementation
```rust
use sqlx::FromRow;
```

Implements from_row for SQLite/PostgreSQL

**File: `/home/user/navign/shared/src/schema/sqlite_from_row.rs` (23 lines)**

SQLite-specific FromRow implementations:
- Lines 7-9: sqlx imports
- Implements `FromRow` for SQLite schema (when not using postgres)
- Located at: `/home/user/navign/shared/src/schema/sqlite_from_row.rs`

**File: `/home/user/navign/shared/src/schema/account.rs`**

Line 12: SQL trait derivation
```rust
#[cfg(feature = "sql")]
derive(sqlx::FromRow)
```

Lines 127, 183, 243: `use sqlx::Row` for field extraction

**File: `/home/user/navign/shared/src/schema/merchant.rs`**

Line 296: `use sqlx::Row` for custom field extraction

#### Used By
- **Mobile** (Cargo.toml line 67-72): explicitly requests `"sql"`
  - Uses SQLite for local caching
  - Stores entities, areas, merchants, beacons
  - SQLitePool for database operations
  
- **Server** (Cargo.toml line 32-39): requests "postgres" which includes "sql"
  - Uses sqlx for database access
  - Supports both PostgreSQL and SQLite

- **Firmware**: does NOT request "sql"

- **Orchestrator**: does NOT request "sql"

#### Database Support

Mobile uses SQLite:
- Local offline cache
- Entities, areas, merchants, beacons
- via tauri-plugin-sql
- Fast queries for map rendering

Server uses:
- PostgreSQL (primary with postgres feature)
- Can also use SQLite (via sql feature without postgres)

#### Recommendation
**KEEP** - Feature actively used by mobile for SQLite, provides database abstraction

---

### 8. **ts-rs** ✅ USED - KEEP

#### Feature Definition (Cargo.toml)
```toml
ts-rs = ["dep:ts-rs", "std", "serde"]
```

#### Dependencies Enabled
- `ts-rs 10.0.0` (TypeScript type generation)
  - With `serde-compat` and `chrono-impl` features
- `std` (standard library)
- `serde` (serialization)

#### Code Usage

**File: `/home/user/navign/shared/src/bin/gen_ts_schema.rs` (50 lines)**

Binary that generates TypeScript definitions:
```rust
use ts_rs::TS;

fn main() {
    Entity::export_all().expect("Failed to export Entity");
    EntityType::export_all().expect("Failed to export EntityType");
    // ... exports for all 21 types
}
```

**Exported Types (21 total):**
1. Entity / EntityType
2. Area / Floor / FloorType
3. Beacon / BeaconDevice / BeaconType
4. Merchant / MerchantType / MerchantStyle / FoodType / FoodCuisine / ChineseFoodCuisine / FacilityType / SocialMedia / SocialMediaPlatform
5. Connection / ConnectionType
6. Firmware / FirmwareDevice

**Schema Files:**
All core schema types are decorated with `#[derive(TS)]`:
- `/home/user/navign/shared/src/schema/entity.rs`
- `/home/user/navign/shared/src/schema/area.rs`
- `/home/user/navign/shared/src/schema/beacon.rs`
- `/home/user/navign/shared/src/schema/merchant.rs`
- `/home/user/navign/shared/src/schema/connection.rs`
- `/home/user/navign/shared/src/schema/firmware.rs`

#### Binary Invocation

From `shared/Cargo.toml`:
```toml
[[bin]]
name = "gen-ts-schema"
path = "src/bin/gen_ts_schema.rs"
required-features = ["ts-rs"]
```

From `justfile`:
```bash
just gen-ts-schema
# Runs: cargo run --bin gen-ts-schema --features ts-rs
```

#### Output

Generates TypeScript definitions in `/home/user/navign/shared/bindings/generated/`:
- 21 `.d.ts` files
- Auto-sync with Rust types
- Zero-maintenance approach

#### Used By

**Development Process:**
- Runs during TypeScript schema generation
- Mobile (vue/tauri) imports from generated types

**Compile-time Only:**
- Only needed when `ts-rs` feature is enabled
- Binary is optional
- Not included in default build

#### Recent Changes

From CLAUDE.md (Mobile TypeScript Schema Migration #103):
- TypeScript generation **consolidated** from separate `ts-schema/` crate (4,838 lines removed) 
- **Integrated** into `shared/src/bin/gen_ts_schema.rs` (113 lines added)
- Removes redundant crate
- Single source of truth in shared library

#### Recommendation
**KEEP** - Feature is actively used for automatic TypeScript type generation, enables mobile development

---

## Feature Dependencies Chart

```
┌─────────────┐
│   default   │
│   = ["std"] │
└──────┬──────┘
       │
       ▼
┌─────────────────────────┐
│ std (core feature)      │
│ Enables: alloc          │
└──────┬──────────────────┘
       │
       ├─────────────────────────────────────────┐
       │                                         │
       ▼                                         ▼
   ┌────────────┐                          ┌──────────┐
   │ serde      │                          │ alloc    │
   │ (optional) │                          │ (core)   │
   └─────┬──────┘                          └──────────┘
         │
         ├──────────────────────┬──────────────────────┐
         │                      │                      │
         ▼                      ▼                      ▼
    ┌─────────────┐      ┌──────────┐         ┌──────────────┐
    │ mongodb     │      │ postgres │         │ base64       │
    │ (UNUSED)    │      │ (USED)   │         │ (USED)       │
    │ ✗ REMOVE    │      │ ✓ KEEP   │         │ ✓ KEEP       │
    └─────────────┘      └────┬─────┘         └──────────────┘
                              │
                              ├─> chrono (USED)
                              ├─> geo (USED)
                              └─> uuid, serde_json
                      
    sql (USED)
    └─> geo (USED, advanced pathfinding)
    └─> uuid, serde_json
    └─> sqlx (database)
    
    ts-rs (USED)
    └─> std, serde (TypeScript generation)
    
    crypto (USED - firmware)
    └─> sha2, hmac, p256 (cryptographic operations)
```

---

## Feature Request Summary by Component

### Server
```toml
navign-shared = { 
  features = ["std", "serde", "mongodb", "geo", "postgres", "sql"]
}
```
- Uses: postgres, geo, sql ✅
- Unused: mongodb ❌

### Firmware  
```toml
navign-shared = {
  features = ["heapless", "crypto", "defmt", "postcard", "serde"]
}
```
- Uses: crypto ✅
- All requested features used

### Mobile (Tauri)
```toml
navign-shared = {
  features = ["std", "serde", "sql", "postcard"]
}
```
- Uses: sql ✅
- All requested features used

### Orchestrator
```toml
navign-shared = {
  features = ["std", "serde", "alloc"]
}
```
- Uses: basic types only
- All requested features used

---

## Recommendations

### 1. **REMOVE mongodb Feature** (High Priority)

**Action:** Remove `mongodb = ["alloc", "serde", "dep:bson"]` from shared/Cargo.toml

**Rationale:**
- Never used in shared library code
- No #[cfg(feature = "mongodb")] gates anywhere
- Server doesn't depend on shared's mongodb feature
- Server imports mongodb directly

**Impact:**
- Reduces shared crate dependencies
- Makes feature set clearer
- No breaking changes (nothing uses it from shared)
- Saves ~2KB from compiled binary

**Code Changes:**
```diff
# shared/Cargo.toml
[features]
default = ["std"]
base64 = ["dep:base64", "alloc"]
chrono = ["dep:chrono"]
crypto = ["dep:sha2", "dep:hmac", "dep:p256"]
heapless = ["dep:heapless", "postcard?/heapless"]
- mongodb = ["alloc", "serde", "dep:bson"]
postcard = ["dep:postcard", "serde"]
...

[dependencies]
- bson = { version = "2.15.0", optional = true, features = ["serde_with"] }

# server/Cargo.toml
navign-shared = {
  features = ["std", "serde", "geo", "postgres", "sql"]
- # Remove "mongodb"
}
```

### 2. **KEEP All Other Features**

All other 7 features are actively used and properly implemented:
- **base64**: Optional encoding functionality, properly gated
- **chrono**: PostgreSQL timestamp support
- **crypto**: Firmware cryptographic operations
- **geo**: Advanced pathfinding and PostGIS support
- **postgres**: PostgreSQL migration layer
- **sql**: SQLite support for mobile
- **ts-rs**: TypeScript type generation

---

## Summary Table

| Feature | Status | Gates | Used By | Size Impact | Action |
|---------|--------|-------|---------|-------------|--------|
| mongodb | DEAD | 0 | server (unused) | ~50KB (bson) | REMOVE |
| base64 | LIVE | 2 | firmware, mobile | 50KB | KEEP |
| chrono | LIVE | 0 | postgres models | 40KB | KEEP |
| crypto | LIVE | 8 | firmware | 60KB | KEEP |
| geo | LIVE | 24 | server, pathfinding | 100KB+ | KEEP |
| postgres | LIVE | 10+ | server | 80KB+ | KEEP |
| sql | LIVE | 4 | mobile, server | 80KB+ | KEEP |
| ts-rs | LIVE | 0 | (build-time) | 0 (dev-dep) | KEEP |

---

## Files Requiring Updates

If removing mongodb feature:

1. `/home/user/navign/shared/Cargo.toml` (lines 12-45)
2. `/home/user/navign/server/Cargo.toml` (line 35)

