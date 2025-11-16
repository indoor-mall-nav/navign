# Navign Server: MongoDB vs PostgreSQL CRUD Analysis

## Executive Summary

The Navign server currently uses **MongoDB as the primary database** with **PostgreSQL repository layer fully implemented but not yet integrated into API handlers**. The architecture follows a **dual-database pattern** with automatic fallback from PostgreSQL to MongoDB.

---

## 1. ARCHITECTURE OVERVIEW

### Application State (server/src/state.rs)
```rust
pub struct AppState {
    pub db: Database,                      // MongoDB Database
    pub pg_pool: Option<Arc<pg::PgPool>>,  // PostgreSQL Pool (Optional)
    pub private_key: SigningKey,
    pub prometheus_handle: PrometheusHandle,
}
```

### Connection Strategy (server/src/main.rs:171-201)
1. **MongoDB**: Always connected (required)
2. **PostgreSQL**: Optional, connected if `POSTGRES_URL` env var is set
3. **Migrations**: Auto-run if `POSTGRES_RUN_MIGRATIONS=true`

---

## 2. CRUD ENDPOINTS REQUIRING DUAL-DATABASE SUPPORT

### Status: Currently MongoDB-Only, PostgreSQL Layer Ready

#### ENTITY ENDPOINTS
| Endpoint | Method | MongoDB Handler | PostgreSQL Status |
|----------|--------|-----------------|-------------------|
| `/api/entities` | GET | Entity::search_entity_handler | ✅ get_entities (pg/handlers.rs:58) |
| `/api/entities/{id}` | GET | Entity::get_one_handler | ✅ get_entity_by_id (pg/handlers.rs:99) |
| `/api/entities` | POST | Entity::create_handler | ✅ create_entity (pg/handlers.rs:126) |
| `/api/entities` | PUT | Entity::update_handler | ✅ update_entity (pg/handlers.rs:151) |
| `/api/entities/{id}` | DELETE | Entity::delete_handler | ✅ delete_entity (pg/handlers.rs:175) |

#### AREA ENDPOINTS
| Endpoint | Method | MongoDB Handler | PostgreSQL Status |
|----------|--------|-----------------|-------------------|
| `/api/entities/{eid}/areas` | GET | Area::get_handler | ✅ get_areas_by_entity (pg/handlers.rs:201) |
| `/api/entities/{eid}/areas/{id}` | GET | Area::get_one_handler | ✅ get_area_by_id (pg/handlers.rs:241) |
| `/api/entities/{eid}/areas` | POST | Area::create_handler | Partial |
| `/api/entities/{eid}/areas` | PUT | Area::update_handler | Partial |
| `/api/entities/{eid}/areas/{id}` | DELETE | Area::delete_handler | Partial |

#### BEACON ENDPOINTS
| Endpoint | Method | MongoDB Handler | PostgreSQL Status |
|----------|--------|-----------------|-------------------|
| `/api/entities/{eid}/beacons` | GET | Beacon::get_handler | ✅ get_beacons_by_entity (pg/handlers.rs:272) |
| `/api/entities/{eid}/beacons/{id}` | GET | Beacon::get_one_handler | Partial |
| `/api/entities/{eid}/beacons` | POST | Beacon::create_handler | Partial |
| `/api/entities/{eid}/beacons` | PUT | Beacon::update_handler | Partial |
| `/api/entities/{eid}/beacons/{id}` | DELETE | Beacon::delete_handler | Partial |

#### MERCHANT ENDPOINTS
| Endpoint | Method | MongoDB Handler | PostgreSQL Status |
|----------|--------|-----------------|-------------------|
| `/api/entities/{eid}/merchants` | GET | Merchant::get_handler | Repository ready |
| `/api/entities/{eid}/merchants/{id}` | GET | Merchant::get_one_handler | Repository ready |
| `/api/entities/{eid}/merchants` | POST | Merchant::create_handler | Repository ready |
| `/api/entities/{eid}/merchants` | PUT | Merchant::update_handler | Repository ready |
| `/api/entities/{eid}/merchants/{id}` | DELETE | Merchant::delete_handler | Repository ready |

#### CONNECTION ENDPOINTS
| Endpoint | Method | MongoDB Handler | PostgreSQL Status |
|----------|--------|-----------------|-------------------|
| `/api/entities/{eid}/connections` | GET | Connection::get_handler | Repository ready |
| `/api/entities/{eid}/connections/{id}` | GET | Connection::get_one_handler | Repository ready |
| `/api/entities/{eid}/connections` | POST | Connection::create_handler | Repository ready |
| `/api/entities/{eid}/connections` | PUT | Connection::update_handler | Repository ready |
| `/api/entities/{eid}/connections/{id}` | DELETE | Connection::delete_handler | Repository ready |

---

## 3. AVAILABLE POSTGRESQL REPOSITORY METHODS

### Repository Trait System
```rust
// Generic base trait
#[async_trait]
pub trait Repository<T> {
    async fn get_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<T>>;
    async fn create(&self, entity: &T) -> Result<String>;
    async fn update(&self, entity: &T) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn count(&self) -> Result<i64>;
}

// For UUID-based entities (Entity, User)
#[async_trait]
pub trait UuidRepository<T>: Repository<T> {
    async fn get_by_uuid(&self, id: Uuid) -> Result<Option<T>>;
}

// For integer ID entities (Area, Beacon, Merchant, Connection, etc)
#[async_trait]
pub trait IntRepository<T>: Repository<T> {
    async fn get_by_int(&self, id: i32) -> Result<Option<T>>;
}
```

### 3.1 EntityRepository (UUID-based)
**File:** server/src/pg/repository.rs:55-231

**Implemented Methods:**
- `new(pool: PgPool) -> Self`
- `search_by_fields()` - Complex search with nation, region, city, name, coordinates
- `get_by_id(&str)` → `Option<PgEntity>`
- `get_all(offset, limit)` → `Vec<PgEntity>`
- `create(&PgEntity)` → String (UUID)
- `update(&PgEntity)` → `()`
- `delete(&str)` → `()`
- `count()` → i64
- `get_by_uuid(Uuid)` → `Option<PgEntity>`

**Key Features:**
- Search with geographic bounds (longitude/latitude)
- Case-insensitive name search (ILIKE)
- Pagination with offset/limit

### 3.2 UserRepository (UUID-based)
**File:** server/src/pg/repository.rs:237-378

**Implemented Methods:**
- Standard Repository methods (get_by_id, get_all, create, update, delete, count)
- `get_by_uuid(Uuid)` → `Option<PgUser>`
- `get_by_username(&str)` → `Option<PgUser>`
- `get_by_email(&str)` → `Option<PgUser>`

### 3.3 AreaRepository (Integer ID)
**File:** server/src/pg/repository.rs:382-533

**Implemented Methods:**
- Standard Repository methods
- `get_by_int(i32)` → `Option<PgArea>`
- `get_by_entity(Uuid, offset, limit)` → `Vec<PgArea>` ⭐
- `get_by_floor(Uuid, floor: &str)` → `Vec<PgArea>`
- `get_by_entity_and_beacon_code(Uuid, code)` → `Option<PgArea>`
- `search_by_name(Uuid, pattern)` → `Vec<PgArea>`

### 3.4 BeaconRepository (Integer ID)
**File:** server/src/pg/repository.rs:535-721

**Implemented Methods:**
- Standard Repository methods
- `get_by_int(i32)` → `Option<PgBeacon>`
- `get_by_entity(Uuid, offset, limit)` → `Vec<PgBeacon>` ⭐
- `get_by_area(i32, offset, limit)` → `Vec<PgBeacon>`
- `get_by_device_id(&str)` → `Option<PgBeacon>` ⭐
- `get_by_device_type(Uuid, device_type: &str)` → `Vec<PgBeacon>`
- `get_by_type(beacon_type: &str)` → `Vec<PgBeacon>`
- `search_by_name(Uuid, pattern)` → `Vec<PgBeacon>`
- `get_by_coordinates(entity_id, x, y, radius)` → `Vec<PgBeacon>`

### 3.5 MerchantRepository (Integer ID)
**File:** server/src/pg/repository.rs:722-932

**Implemented Methods:**
- Standard Repository methods
- `get_by_int(i32)` → `Option<PgMerchant>`
- `get_by_entity(Uuid, offset, limit)` → `Vec<PgMerchant>`
- `get_by_area(i32, offset, limit)` → `Vec<PgMerchant>`
- `get_by_type(Uuid, merchant_type: &str)` → `Vec<PgMerchant>`
- `search_by_name(Uuid, pattern)` → `Vec<PgMerchant>`
- `get_by_coordinates(entity_id, x, y, radius)` → `Vec<PgMerchant>`

### 3.6 ConnectionRepository (Integer ID)
**File:** server/src/pg/repository.rs:933-1089

**Implemented Methods:**
- Standard Repository methods
- `get_by_int(i32)` → `Option<PgConnection>`
- `get_by_entity(Uuid, offset, limit)` → `Vec<PgConnection>`
- `get_by_type(Uuid, connection_type: &str)` → `Vec<PgConnection>`
- `get_connected_areas(from_area_id, to_area_id)` → `Option<PgConnection>`

### 3.7 BeaconSecretRepository, UserPublicKeyRepository, FirmwareRepository
Also fully implemented with standard CRUD + entity-specific search methods

---

## 4. MONGODB HANDLER PATTERNS

### Service Trait Pattern (server/src/schema/service.rs:285-420)

All MongoDB entities (Entity, Area, Beacon, Merchant, Connection) implement the `Service` trait:

```rust
#[async_trait]
pub trait Service: Serialize + DeserializeOwned + Send + Sync + Clone {
    fn get_id(&self) -> String;
    fn get_collection_name() -> &'static str;
    fn require_unique_name() -> bool;

    // CRUD Methods
    async fn get_one_by_id(db: &Database, id: &str) -> Option<Self>;
    async fn get_all(db: &Database) -> Result<Vec<Self>>;
    async fn create(&self, db: &Database) -> Result<ObjectId>;
    async fn update(&self, db: &Database) -> Result<()>;
    async fn delete_by_id(db: &Database, id: &str) -> Result<()>;

    // Handler Methods
    async fn get_handler(State(state): State<AppState>, ...) -> impl IntoResponse;
    async fn get_one_handler(State(state): State<AppState>, ...) -> impl IntoResponse;
    async fn create_handler(State(state): State<AppState>, ...) -> impl IntoResponse;
    async fn update_handler(State(state): State<AppState>, ...) -> impl IntoResponse;
    async fn delete_handler(State(state): State<AppState>, ...) -> impl IntoResponse;
}
```

### Handler Signatures

#### GET_HANDLER (List with Search/Pagination)
```rust
async fn get_handler(
    State(state): State<AppState>,
    Query(ReadQuery {
        offset, limit, query, sort, asc, case_sensitive
    }): Query<ReadQuery>,
    Path(entity): Path<String>,  // entity_id for scoped queries
) -> impl IntoResponse
```

#### GET_ONE_HANDLER (Fetch by ID)
```rust
async fn get_one_handler(
    State(state): State<AppState>,
    Path(id): Path<(String, String)>,  // (entity_id, item_id)
) -> impl IntoResponse
```

#### CREATE_HANDLER
```rust
async fn create_handler(
    State(state): State<AppState>,
    axum::Json(service): axum::Json<Self>,
) -> impl IntoResponse
```

#### UPDATE_HANDLER
```rust
async fn update_handler(
    State(state): State<AppState>,
    axum::Json(service): axum::Json<Self>,
) -> impl IntoResponse
```

#### DELETE_HANDLER
```rust
async fn delete_handler(
    State(state): State<AppState>,
    Path(id): Path<(String, String)>,
) -> impl IntoResponse
```

### OneInArea Trait (server/src/schema/service.rs:422-510)

For entities scoped to areas (Beacon, Merchant):

```rust
pub trait OneInArea: Service {
    async fn get_all_in_area_handler(
        State(state): State<AppState>,
        Query(params): Query<ReadQuery>,
        Path((entity, area)): Path<(String, String)>,
    ) -> impl IntoResponse
}
```

---

## 5. EXISTING DUAL-DATABASE HANDLER EXAMPLES

### Location: server/src/pg/handlers.rs (309 lines)

**Implemented Examples:**

#### 1. Entity Handlers (Complete)
```rust
pub async fn get_entities(
    State(state): State<AppState>,
    Query(query): Query<EntitySearchQuery>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        // PostgreSQL path: Convert PgEntity → Entity
        let repo = EntityRepository::new(pg_pool.as_ref().clone());
        let pg_entities = repo.search_by_fields(...).await?;
        let entities: Vec<Entity> = pg_entities.into_iter()
            .map(pg_entity_to_entity).collect();
        Ok(Json(entities))
    } else {
        // MongoDB fallback
        let entities = Entity::search_entity_by_fields(&state.db, ...).await?;
        Ok(Json(entities))
    }
}
```

**Pattern:**
1. Check if PostgreSQL pool exists: `state.pg_pool.as_ref()`
2. If yes: Create repository, fetch PgEntity, convert to Entity using adapters
3. If no: Use MongoDB Service methods
4. Return same response type for both paths

#### 2. Area Handlers (Complete)
```rust
pub async fn get_areas_by_entity(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        let repo = AreaRepository::new(pg_pool.as_ref().clone());
        let uuid = Uuid::parse_str(&entity_id)?;
        let pg_areas = repo.get_by_entity(uuid, pagination.offset, pagination.limit).await?;
        let areas: Vec<Area> = pg_areas.into_iter().map(pg_area_to_area).collect();
        Ok(Json(areas))
    } else {
        // MongoDB: Manual collection.find() with filters
        let oid = ObjectId::from_str(&entity_id)?;
        let collection = state.db.collection::<Area>("areas");
        let cursor = collection.find(doc! { "entity": oid })
            .limit(pagination.limit)
            .skip(pagination.offset as u64)
            .await?;
        let areas: Vec<Area> = cursor.try_collect().await?;
        Ok(Json(areas))
    }
}
```

#### 3. Beacon Handlers (Skeleton)
```rust
pub async fn get_beacons_by_entity(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        let repo = BeaconRepository::new(pg_pool.as_ref().clone());
        let uuid = Uuid::parse_str(&entity_id)?;
        let pg_beacons = repo.get_by_entity(uuid, pagination.offset, pagination.limit).await?;
        let beacons: Vec<Beacon> = pg_beacons.into_iter().map(pg_beacon_to_beacon).collect();
        Ok(Json(beacons))
    } else {
        // MongoDB path (same as Area pattern)
    }
}
```

---

## 6. SCHEMA CONVERSION ADAPTERS

### Location: server/src/pg/adapters.rs (248 lines)

**Pattern:** Each entity has bidirectional converters:

```rust
// PostgreSQL → Shared Schema
pub fn pg_entity_to_entity(pg: PgEntity) -> Entity { ... }
pub fn pg_area_to_area(pg: PgArea) -> Area { ... }
pub fn pg_beacon_to_beacon(pg: PgBeacon) -> Beacon { ... }
pub fn pg_merchant_to_merchant(pg: PgMerchant) -> Merchant { ... }
pub fn pg_connection_to_connection(pg: PgConnection) -> Connection { ... }

// Shared Schema → PostgreSQL
pub fn entity_to_pg_entity(entity: Entity) -> PgEntity { ... }
pub fn area_to_pg_area(area: Area, entity_id: Uuid) -> PgArea { ... }
pub fn beacon_to_pg_beacon(beacon: Beacon, entity_id: Uuid, area_id: i32, ...) -> PgBeacon { ... }
// Note: Some converters require additional context (foreign key IDs)
```

**Special Cases:**
- **Floor Parsing:** Area handler converts between PostgreSQL string floors and shared Floor structs
- **Placeholder IDs:** When converting PgEntity → Entity, ObjectIds are placeholders (client should track UUIDs separately)
- **PostGIS Points:** Beacon location stored as PostGIS point in PostgreSQL

---

## 7. POSTGRESQL MODELS

### Location: server/src/pg/models.rs (re-exports from navign_shared)

Available PostgreSQL models:
- `PgEntity` - UUID primary key
- `PgUser` - UUID primary key
- `PgArea` - Integer primary key, foreign key to Entity
- `PgBeacon` - Integer primary key, foreign key to Entity + Area
- `PgMerchant` - Integer primary key, foreign key to Entity + Area
- `PgConnection` - Integer primary key, foreign key to areas
- `PgBeaconSecret` - Integer primary key
- `PgUserPublicKey` - Integer primary key
- `PgFirmware` - Integer primary key

---

## 8. IMPLEMENTATION RECOMMENDATIONS

### Priority 1: High-Impact Endpoints (User-Facing)

**These should be migrated first (they're fully supported):**

1. ✅ **Entity CRUD** - All handlers implemented in pg/handlers.rs
2. ✅ **Area GET** - get_areas_by_entity, get_area_by_id implemented
3. ✅ **Beacon GET** - get_beacons_by_entity handler skeleton exists

**Action:** Replace main.rs routes to use dual-database handlers instead of Service trait

### Priority 2: Complete Area/Beacon Create/Update/Delete

**Current status:** PostgreSQL repositories ready, handlers need implementation

**Pattern to follow:**
```rust
pub async fn create_area(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
    Json(area): Json<Area>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        let repo = AreaRepository::new(pg_pool.as_ref().clone());
        let uuid = Uuid::parse_str(&entity_id)?;
        let pg_area = area_to_pg_area(area, uuid);
        let id = repo.create(&pg_area).await?;
        Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
    } else {
        area.create(&state.db).await?;
        Ok((StatusCode::CREATED, Json(area)))
    }
}
```

### Priority 3: Merchant & Connection CRUD

**Current status:** Repositories fully implemented, just need handlers

### Priority 4: Search/Filter Optimization

**Leverage PostgreSQL-specific features:**
- `EntityRepository::search_by_fields()` - Complex geographic queries
- `BeaconRepository::get_by_coordinates()` - Radius search using PostGIS
- Full-text search support (if added to schema)

---

## 9. DATABASE CONFIGURATION

### MongoDB (Required)
```bash
MONGODB_HOST=localhost:27017
MONGODB_DB_NAME=navign
```

### PostgreSQL (Optional)
```bash
POSTGRES_URL=postgresql://user:password@localhost:5432/navign
POSTGRES_RUN_MIGRATIONS=true  # Auto-run migrations on startup
```

### Connection Pool Settings (server/src/pg/pool.rs)
- Max connections: 10
- Min connections: 2
- Acquire timeout: 10s
- Idle timeout: 30s
- Max lifetime: 30 minutes

---

## 10. QUICK REFERENCE: ENTITIES AND THEIR ID TYPES

| Entity | ID Type | PostgreSQL | MongoDB |
|--------|---------|------------|---------|
| Entity | UUID | `PgEntity.id: Uuid` | `Entity.id: ObjectId` |
| User | UUID | `PgUser.id: Uuid` | `User.id: ObjectId` |
| Area | Integer | `PgArea.id: i32` | `Area.id: ObjectId` |
| Beacon | Integer | `PgBeacon.id: i32` | `Beacon.id: ObjectId` |
| Merchant | Integer | `PgMerchant.id: i32` | `Merchant.id: ObjectId` |
| Connection | Integer | `PgConnection.id: i32` | `Connection.id: ObjectId` |
| BeaconSecret | Integer | `PgBeaconSecret.id: i32` | N/A |
| UserPublicKey | Integer | `PgUserPublicKey.id: i32` | N/A |
| Firmware | Integer | `PgFirmware.id: i32` | N/A |

---

## 11. FILE LOCATIONS SUMMARY

| Component | Location | Status |
|-----------|----------|--------|
| App State | server/src/state.rs | ✅ Ready |
| DB Connections | server/src/database.rs (MongoDB), server/src/pg/pool.rs (PostgreSQL) | ✅ Ready |
| Repositories | server/src/pg/repository.rs (1565 lines) | ✅ Complete |
| Models | server/src/pg/models.rs | ✅ Ready |
| Adapters | server/src/pg/adapters.rs (248 lines) | ✅ Complete |
| Dual Handlers | server/src/pg/handlers.rs (309 lines) | ⚠️ Partial (Entity, Area complete; others skeleton) |
| MongoDB Handlers | server/src/schema/service.rs | ✅ Active |
| Routes | server/src/main.rs (lines 215-350) | 📝 Needs update |

---

## 12. NEXT STEPS

1. **Replace main.rs routes** - Swap Service trait handlers with dual-database versions
2. **Complete pg/handlers.rs** - Add missing Beacon, Merchant, Connection handlers
3. **Add tests** - Test both MongoDB and PostgreSQL code paths
4. **Migration workflow** - Implement dual-write mode (write to both, read from MongoDB)
5. **Cutover** - Transition to PostgreSQL-only when migration is complete

---

## APPENDIX: Error Handling Pattern

All dual-database handlers use consistent error handling:

```rust
use crate::error::{Result, ServerError};

// Result type already includes ServerError
pub async fn my_handler(...) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        let repo = MyRepository::new(pg_pool.as_ref().clone());
        // Errors automatically convert to Result type
        let item = repo.get_by_id(&id).await?;
        Ok(Json(item))
    } else {
        // MongoDB errors wrapped in ServerError::DatabaseQuery
        let item = MyEntity::get_one_by_id(&state.db, &id)
            .await
            .ok_or_else(|| ServerError::NotFound(...))?;
        Ok(Json(item))
    }
}
```

Error types used:
- `ServerError::NotFound` - Item doesn't exist
- `ServerError::DatabaseQuery` - Query execution failed
- `ServerError::InvalidInput` - Invalid ID format
- `ServerError::DatabaseConnection` - Connection failed

