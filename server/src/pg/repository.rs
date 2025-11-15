#![allow(dead_code)] // Not yet integrated with handlers

use super::models::*;
use super::pool::PgPool;
use crate::error::{Result, ServerError};
/// Repository traits and implementations for PostgreSQL
///
/// This provides a clean abstraction layer for database operations
/// without modifying the existing MongoDB service trait.
use async_trait::async_trait;
use sqlx::types::Uuid;

// ============================================================================
// Repository Traits
// ============================================================================

/// Generic repository trait for CRUD operations
#[async_trait]
pub trait Repository<T> {
    /// Get by ID
    async fn get_by_id(&self, id: &str) -> Result<Option<T>>;

    /// Get all with pagination
    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<T>>;

    /// Create
    async fn create(&self, entity: &T) -> Result<String>;

    /// Update
    async fn update(&self, entity: &T) -> Result<()>;

    /// Delete
    async fn delete(&self, id: &str) -> Result<()>;

    /// Count total
    async fn count(&self) -> Result<i64>;
}

/// Repository for entities with UUID
#[async_trait]
pub trait UuidRepository<T>: Repository<T> {
    async fn get_by_uuid(&self, id: Uuid) -> Result<Option<T>>;
}

/// Repository for entities with integer ID
#[async_trait]
pub trait IntRepository<T>: Repository<T> {
    async fn get_by_int(&self, id: i32) -> Result<Option<T>>;
}

// ============================================================================
// Entity Repository (UUID)
// ============================================================================

pub struct EntityRepository {
    pool: PgPool,
}

impl EntityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search_by_fields(
        &self,
        nation: Option<&str>,
        region: Option<&str>,
        city: Option<&str>,
        name: Option<&str>,
        longitude: Option<f64>,
        latitude: Option<f64>,
    ) -> Result<Vec<PgEntity>> {
        let mut query = String::from("SELECT * FROM entities WHERE 1=1");
        let mut conditions = Vec::new();

        if let Some(n) = nation {
            conditions.push(format!(" AND nation = '{}'", n));
        }
        if let Some(r) = region {
            conditions.push(format!(" AND region = '{}'", r));
        }
        if let Some(c) = city {
            conditions.push(format!(" AND city = '{}'", c));
        }
        if let Some(name_val) = name {
            conditions.push(format!(" AND name ILIKE '%{}%'", name_val));
        }
        if let Some(lon) = longitude {
            conditions.push(format!(
                " AND longitude_min <= {} AND longitude_max >= {}",
                lon, lon
            ));
        }
        if let Some(lat) = latitude {
            conditions.push(format!(
                " AND latitude_min <= {} AND latitude_max >= {}",
                lat, lat
            ));
        }

        for condition in conditions {
            query.push_str(&condition);
        }

        sqlx::query_as::<_, PgEntity>(&query)
            .fetch_all(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to search entities: {}", e)))
    }
}

#[async_trait]
impl Repository<PgEntity> for EntityRepository {
    async fn get_by_id(&self, id: &str) -> Result<Option<PgEntity>> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| ServerError::InvalidInput("Invalid UUID format".to_string()))?;
        self.get_by_uuid(uuid).await
    }

    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<PgEntity>> {
        sqlx::query_as::<_, PgEntity>(
            "SELECT * FROM entities ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch entities: {}", e)))
    }

    async fn create(&self, entity: &PgEntity) -> Result<String> {
        let result = sqlx::query_as::<_, (Uuid,)>(
            r#"
            INSERT INTO entities (type, name, description, nation, region, city, address,
                                 longitude_min, longitude_max, latitude_min, latitude_max, floors)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id
            "#,
        )
        .bind(&entity.r#type)
        .bind(&entity.name)
        .bind(&entity.description)
        .bind(&entity.nation)
        .bind(&entity.region)
        .bind(&entity.city)
        .bind(&entity.address)
        .bind(entity.longitude_min)
        .bind(entity.longitude_max)
        .bind(entity.latitude_min)
        .bind(entity.latitude_max)
        .bind(&entity.floors)
        .fetch_one(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to create entity: {}", e)))?;

        Ok(result.0.to_string())
    }

    async fn update(&self, entity: &PgEntity) -> Result<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE entities
            SET type = $1, name = $2, description = $3, nation = $4, region = $5,
                city = $6, address = $7, longitude_min = $8, longitude_max = $9,
                latitude_min = $10, latitude_max = $11, floors = $12
            WHERE id = $13
            "#,
        )
        .bind(&entity.r#type)
        .bind(&entity.name)
        .bind(&entity.description)
        .bind(&entity.nation)
        .bind(&entity.region)
        .bind(&entity.city)
        .bind(&entity.address)
        .bind(entity.longitude_min)
        .bind(entity.longitude_max)
        .bind(entity.latitude_min)
        .bind(entity.latitude_max)
        .bind(&entity.floors)
        .bind(entity.id)
        .execute(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to update entity: {}", e)))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Entity not found".to_string()));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| ServerError::InvalidInput("Invalid UUID format".to_string()))?;

        let rows_affected = sqlx::query("DELETE FROM entities WHERE id = $1")
            .bind(uuid)
            .execute(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to delete entity: {}", e)))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Entity not found".to_string()));
        }

        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        let result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM entities")
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to count entities: {}", e)))?;

        Ok(result.0)
    }
}

#[async_trait]
impl UuidRepository<PgEntity> for EntityRepository {
    async fn get_by_uuid(&self, id: Uuid) -> Result<Option<PgEntity>> {
        sqlx::query_as::<_, PgEntity>("SELECT * FROM entities WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch entity: {}", e)))
    }
}

// ============================================================================
// User Repository (UUID)
// ============================================================================

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_username(&self, username: &str) -> Result<Option<PgUser>> {
        sqlx::query_as::<_, PgUser>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to fetch user by username: {}", e))
            })
    }

    pub async fn get_by_email(&self, email: &str) -> Result<Option<PgUser>> {
        sqlx::query_as::<_, PgUser>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to fetch user by email: {}", e))
            })
    }
}

#[async_trait]
impl Repository<PgUser> for UserRepository {
    async fn get_by_id(&self, id: &str) -> Result<Option<PgUser>> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| ServerError::InvalidInput("Invalid UUID format".to_string()))?;
        self.get_by_uuid(uuid).await
    }

    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<PgUser>> {
        sqlx::query_as::<_, PgUser>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch users: {}", e)))
    }

    async fn create(&self, user: &PgUser) -> Result<String> {
        let result = sqlx::query_as::<_, (Uuid,)>(
            r#"
            INSERT INTO users (username, email, phone, google, wechat, hashed_password, activated, privileged)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.phone)
        .bind(&user.google)
        .bind(&user.wechat)
        .bind(&user.hashed_password)
        .bind(user.activated)
        .bind(user.privileged)
        .fetch_one(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to create user: {}", e)))?;

        Ok(result.0.to_string())
    }

    async fn update(&self, user: &PgUser) -> Result<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE users
            SET username = $1, email = $2, phone = $3, google = $4, wechat = $5,
                hashed_password = $6, activated = $7, privileged = $8
            WHERE id = $9
            "#,
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.phone)
        .bind(&user.google)
        .bind(&user.wechat)
        .bind(&user.hashed_password)
        .bind(user.activated)
        .bind(user.privileged)
        .bind(user.id)
        .execute(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to update user: {}", e)))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| ServerError::InvalidInput("Invalid UUID format".to_string()))?;

        let rows_affected = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(uuid)
            .execute(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to delete user: {}", e)))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        let result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM users")
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to count users: {}", e)))?;

        Ok(result.0)
    }
}

#[async_trait]
impl UuidRepository<PgUser> for UserRepository {
    async fn get_by_uuid(&self, id: Uuid) -> Result<Option<PgUser>> {
        sqlx::query_as::<_, PgUser>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch user: {}", e)))
    }
}

// ============================================================================
// Area Repository (INTEGER)
// ============================================================================

pub struct AreaRepository {
    pool: PgPool,
}

impl AreaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_entity(
        &self,
        entity_id: Uuid,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<PgArea>> {
        sqlx::query_as::<_, PgArea>(
            "SELECT * FROM areas WHERE entity_id = $1 ORDER BY name LIMIT $2 OFFSET $3",
        )
        .bind(entity_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch areas: {}", e)))
    }

    pub async fn get_by_floor(&self, entity_id: Uuid, floor: &str) -> Result<Vec<PgArea>> {
        sqlx::query_as::<_, PgArea>(
            "SELECT * FROM areas WHERE entity_id = $1 AND floor = $2 ORDER BY name",
        )
        .bind(entity_id)
        .bind(floor)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch areas by floor: {}", e)))
    }
}

#[async_trait]
impl Repository<PgArea> for AreaRepository {
    async fn get_by_id(&self, id: &str) -> Result<Option<PgArea>> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;
        self.get_by_int(int_id).await
    }

    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<PgArea>> {
        sqlx::query_as::<_, PgArea>(
            "SELECT * FROM areas ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch areas: {}", e)))
    }

    async fn create(&self, area: &PgArea) -> Result<String> {
        let result = sqlx::query_as::<_, (i32,)>(
            r#"
            INSERT INTO areas (entity_id, name, description, floor, beacon_code, polygon, centroid)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(area.entity_id)
        .bind(&area.name)
        .bind(&area.description)
        .bind(&area.floor)
        .bind(&area.beacon_code)
        .bind(&area.polygon)
        .bind(area.centroid)
        .fetch_one(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to create area: {}", e)))?;

        Ok(result.0.to_string())
    }

    async fn update(&self, area: &PgArea) -> Result<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE areas
            SET entity_id = $1, name = $2, description = $3, floor = $4,
                beacon_code = $5, polygon = $6, centroid = $7
            WHERE id = $8
            "#,
        )
        .bind(area.entity_id)
        .bind(&area.name)
        .bind(&area.description)
        .bind(&area.floor)
        .bind(&area.beacon_code)
        .bind(&area.polygon)
        .bind(area.centroid)
        .bind(area.id)
        .execute(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to update area: {}", e)))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Area not found".to_string()));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;

        let rows_affected = sqlx::query("DELETE FROM areas WHERE id = $1")
            .bind(int_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to delete area: {}", e)))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Area not found".to_string()));
        }

        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        let result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM areas")
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to count areas: {}", e)))?;

        Ok(result.0)
    }
}

#[async_trait]
impl IntRepository<PgArea> for AreaRepository {
    async fn get_by_int(&self, id: i32) -> Result<Option<PgArea>> {
        sqlx::query_as::<_, PgArea>("SELECT * FROM areas WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch area: {}", e)))
    }
}

// ============================================================================
// Beacon Repository (INTEGER)
// ============================================================================

pub struct BeaconRepository {
    pool: PgPool,
}

impl BeaconRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_entity(
        &self,
        entity_id: Uuid,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<PgBeacon>> {
        sqlx::query_as::<_, PgBeacon>(
            "SELECT * FROM beacons WHERE entity_id = $1 ORDER BY name LIMIT $2 OFFSET $3",
        )
        .bind(entity_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch beacons: {}", e)))
    }

    pub async fn get_by_device_id(&self, device_id: &str) -> Result<Option<PgBeacon>> {
        sqlx::query_as::<_, PgBeacon>("SELECT * FROM beacons WHERE device_id = $1")
            .bind(device_id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to fetch beacon by device_id: {}", e))
            })
    }

    pub async fn get_by_area(&self, area_id: i32) -> Result<Vec<PgBeacon>> {
        sqlx::query_as::<_, PgBeacon>("SELECT * FROM beacons WHERE area_id = $1 ORDER BY name")
            .bind(area_id)
            .fetch_all(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to fetch beacons by area: {}", e))
            })
    }

    pub async fn get_by_floor(&self, entity_id: Uuid, floor: &str) -> Result<Vec<PgBeacon>> {
        sqlx::query_as::<_, PgBeacon>(
            "SELECT * FROM beacons WHERE entity_id = $1 AND floor = $2 ORDER BY name",
        )
        .bind(entity_id)
        .bind(floor)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch beacons by floor: {}", e)))
    }
}

#[async_trait]
impl Repository<PgBeacon> for BeaconRepository {
    async fn get_by_id(&self, id: &str) -> Result<Option<PgBeacon>> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;
        self.get_by_int(int_id).await
    }

    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<PgBeacon>> {
        sqlx::query_as::<_, PgBeacon>(
            "SELECT * FROM beacons ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch beacons: {}", e)))
    }

    async fn create(&self, beacon: &PgBeacon) -> Result<String> {
        let result = sqlx::query_as::<_, (i32,)>(
            r#"
            INSERT INTO beacons (entity_id, area_id, merchant_id, connection_id, name, description,
                                type, device_id, floor, location, public_key, capabilities, unlock_method)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id
            "#,
        )
        .bind(beacon.entity_id)
        .bind(beacon.area_id)
        .bind(beacon.merchant_id)
        .bind(beacon.connection_id)
        .bind(&beacon.name)
        .bind(&beacon.description)
        .bind(&beacon.r#type)
        .bind(&beacon.device_id)
        .bind(&beacon.floor)
        .bind(&beacon.location)
        .bind(&beacon.public_key)
        .bind(&beacon.capabilities)
        .bind(&beacon.unlock_method)
        .fetch_one(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to create beacon: {}", e)))?;

        Ok(result.0.to_string())
    }

    async fn update(&self, beacon: &PgBeacon) -> Result<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE beacons
            SET entity_id = $1, area_id = $2, merchant_id = $3, connection_id = $4,
                name = $5, description = $6, type = $7, device_id = $8, floor = $9,
                location = $10, public_key = $11, capabilities = $12, unlock_method = $13
            WHERE id = $14
            "#,
        )
        .bind(beacon.entity_id)
        .bind(beacon.area_id)
        .bind(beacon.merchant_id)
        .bind(beacon.connection_id)
        .bind(&beacon.name)
        .bind(&beacon.description)
        .bind(&beacon.r#type)
        .bind(&beacon.device_id)
        .bind(&beacon.floor)
        .bind(&beacon.location)
        .bind(&beacon.public_key)
        .bind(&beacon.capabilities)
        .bind(&beacon.unlock_method)
        .bind(beacon.id)
        .execute(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to update beacon: {}", e)))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Beacon not found".to_string()));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;

        let rows_affected = sqlx::query("DELETE FROM beacons WHERE id = $1")
            .bind(int_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to delete beacon: {}", e)))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Beacon not found".to_string()));
        }

        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        let result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM beacons")
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to count beacons: {}", e)))?;

        Ok(result.0)
    }
}

#[async_trait]
impl IntRepository<PgBeacon> for BeaconRepository {
    async fn get_by_int(&self, id: i32) -> Result<Option<PgBeacon>> {
        sqlx::query_as::<_, PgBeacon>("SELECT * FROM beacons WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch beacon: {}", e)))
    }
}

// ============================================================================
// Merchant Repository (INTEGER)
// ============================================================================

pub struct MerchantRepository {
    pool: PgPool,
}

impl MerchantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_entity(
        &self,
        entity_id: Uuid,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<PgMerchant>> {
        sqlx::query_as::<_, PgMerchant>(
            "SELECT * FROM merchants WHERE entity_id = $1 ORDER BY name LIMIT $2 OFFSET $3",
        )
        .bind(entity_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch merchants: {}", e)))
    }

    pub async fn get_by_area(&self, area_id: i32) -> Result<Vec<PgMerchant>> {
        sqlx::query_as::<_, PgMerchant>("SELECT * FROM merchants WHERE area_id = $1 ORDER BY name")
            .bind(area_id)
            .fetch_all(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to fetch merchants by area: {}", e))
            })
    }

    pub async fn get_by_floor(&self, entity_id: Uuid, floor: &str) -> Result<Vec<PgMerchant>> {
        sqlx::query_as::<_, PgMerchant>(
            "SELECT * FROM merchants WHERE entity_id = $1 AND floor = $2 ORDER BY name",
        )
        .bind(entity_id)
        .bind(floor)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| {
            ServerError::DatabaseQuery(format!("Failed to fetch merchants by floor: {}", e))
        })
    }

    pub async fn get_by_type(
        &self,
        entity_id: Uuid,
        merchant_type: &str,
    ) -> Result<Vec<PgMerchant>> {
        sqlx::query_as::<_, PgMerchant>(
            "SELECT * FROM merchants WHERE entity_id = $1 AND type = $2 ORDER BY name",
        )
        .bind(entity_id)
        .bind(merchant_type)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| {
            ServerError::DatabaseQuery(format!("Failed to fetch merchants by type: {}", e))
        })
    }
}

#[async_trait]
impl Repository<PgMerchant> for MerchantRepository {
    async fn get_by_id(&self, id: &str) -> Result<Option<PgMerchant>> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;
        self.get_by_int(int_id).await
    }

    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<PgMerchant>> {
        sqlx::query_as::<_, PgMerchant>(
            "SELECT * FROM merchants ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch merchants: {}", e)))
    }

    async fn create(&self, merchant: &PgMerchant) -> Result<String> {
        let result = sqlx::query_as::<_, (i32,)>(
            r#"
            INSERT INTO merchants (entity_id, area_id, name, description, chain, type, logo, images,
                                  social_media, floor, location, merchant_style, food_type, food_cuisine,
                                  chinese_food_cuisine, facility_type, rating, reviews, opening_hours)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING id
            "#,
        )
        .bind(merchant.entity_id)
        .bind(merchant.area_id)
        .bind(&merchant.name)
        .bind(&merchant.description)
        .bind(&merchant.chain)
        .bind(&merchant.r#type)
        .bind(&merchant.logo)
        .bind(&merchant.images)
        .bind(&merchant.social_media)
        .bind(&merchant.floor)
        .bind(&merchant.location)
        .bind(&merchant.merchant_style)
        .bind(&merchant.food_type)
        .bind(&merchant.food_cuisine)
        .bind(&merchant.chinese_food_cuisine)
        .bind(&merchant.facility_type)
        .bind(merchant.rating)
        .bind(merchant.reviews)
        .bind(&merchant.opening_hours)
        .fetch_one(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to create merchant: {}", e)))?;

        Ok(result.0.to_string())
    }

    async fn update(&self, merchant: &PgMerchant) -> Result<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE merchants
            SET entity_id = $1, area_id = $2, name = $3, description = $4, chain = $5,
                type = $6, logo = $7, images = $8, social_media = $9, floor = $10,
                location = $11, merchant_style = $12, food_type = $13, food_cuisine = $14,
                chinese_food_cuisine = $15, facility_type = $16, rating = $17, reviews = $18,
                opening_hours = $19
            WHERE id = $20
            "#,
        )
        .bind(merchant.entity_id)
        .bind(merchant.area_id)
        .bind(&merchant.name)
        .bind(&merchant.description)
        .bind(&merchant.chain)
        .bind(&merchant.r#type)
        .bind(&merchant.logo)
        .bind(&merchant.images)
        .bind(&merchant.social_media)
        .bind(&merchant.floor)
        .bind(&merchant.location)
        .bind(&merchant.merchant_style)
        .bind(&merchant.food_type)
        .bind(&merchant.food_cuisine)
        .bind(&merchant.chinese_food_cuisine)
        .bind(&merchant.facility_type)
        .bind(merchant.rating)
        .bind(merchant.reviews)
        .bind(&merchant.opening_hours)
        .bind(merchant.id)
        .execute(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to update merchant: {}", e)))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Merchant not found".to_string()));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;

        let rows_affected = sqlx::query("DELETE FROM merchants WHERE id = $1")
            .bind(int_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to delete merchant: {}", e)))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Merchant not found".to_string()));
        }

        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        let result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM merchants")
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to count merchants: {}", e)))?;

        Ok(result.0)
    }
}

#[async_trait]
impl IntRepository<PgMerchant> for MerchantRepository {
    async fn get_by_int(&self, id: i32) -> Result<Option<PgMerchant>> {
        sqlx::query_as::<_, PgMerchant>("SELECT * FROM merchants WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch merchant: {}", e)))
    }
}

// ============================================================================
// Connection Repository (INTEGER)
// ============================================================================

pub struct ConnectionRepository {
    pool: PgPool,
}

impl ConnectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_entity(
        &self,
        entity_id: Uuid,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<PgConnection>> {
        sqlx::query_as::<_, PgConnection>(
            "SELECT * FROM connections WHERE entity_id = $1 ORDER BY name LIMIT $2 OFFSET $3",
        )
        .bind(entity_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch connections: {}", e)))
    }

    pub async fn get_by_type(
        &self,
        entity_id: Uuid,
        connection_type: &str,
    ) -> Result<Vec<PgConnection>> {
        sqlx::query_as::<_, PgConnection>(
            "SELECT * FROM connections WHERE entity_id = $1 AND type = $2 ORDER BY name",
        )
        .bind(entity_id)
        .bind(connection_type)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| {
            ServerError::DatabaseQuery(format!("Failed to fetch connections by type: {}", e))
        })
    }
}

#[async_trait]
impl Repository<PgConnection> for ConnectionRepository {
    async fn get_by_id(&self, id: &str) -> Result<Option<PgConnection>> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;
        self.get_by_int(int_id).await
    }

    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<PgConnection>> {
        sqlx::query_as::<_, PgConnection>(
            "SELECT * FROM connections ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch connections: {}", e)))
    }

    async fn create(&self, connection: &PgConnection) -> Result<String> {
        let result = sqlx::query_as::<_, (i32,)>(
            r#"
            INSERT INTO connections (entity_id, name, description, type, connected_areas)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(connection.entity_id)
        .bind(&connection.name)
        .bind(&connection.description)
        .bind(&connection.r#type)
        .bind(&connection.connected_areas)
        .fetch_one(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to create connection: {}", e)))?;

        Ok(result.0.to_string())
    }

    async fn update(&self, connection: &PgConnection) -> Result<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE connections
            SET entity_id = $1, name = $2, description = $3, type = $4, connected_areas = $5
            WHERE id = $6
            "#,
        )
        .bind(connection.entity_id)
        .bind(&connection.name)
        .bind(&connection.description)
        .bind(&connection.r#type)
        .bind(&connection.connected_areas)
        .bind(connection.id)
        .execute(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to update connection: {}", e)))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Connection not found".to_string()));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;

        let rows_affected = sqlx::query("DELETE FROM connections WHERE id = $1")
            .bind(int_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to delete connection: {}", e)))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Connection not found".to_string()));
        }

        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        let result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM connections")
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to count connections: {}", e))
            })?;

        Ok(result.0)
    }
}

#[async_trait]
impl IntRepository<PgConnection> for ConnectionRepository {
    async fn get_by_int(&self, id: i32) -> Result<Option<PgConnection>> {
        sqlx::query_as::<_, PgConnection>("SELECT * FROM connections WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch connection: {}", e)))
    }
}

// ============================================================================
// Beacon Secret Repository (INTEGER)
// ============================================================================

use super::models::extras::PgBeaconSecret;

pub struct BeaconSecretRepository {
    pool: PgPool,
}

impl BeaconSecretRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_beacon_id(&self, beacon_id: i32) -> Result<Option<PgBeaconSecret>> {
        sqlx::query_as::<_, PgBeaconSecret>("SELECT * FROM beacon_secrets WHERE beacon_id = $1")
            .bind(beacon_id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to fetch beacon secret: {}", e))
            })
    }
}

#[async_trait]
impl Repository<PgBeaconSecret> for BeaconSecretRepository {
    async fn get_by_id(&self, id: &str) -> Result<Option<PgBeaconSecret>> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;
        self.get_by_int(int_id).await
    }

    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<PgBeaconSecret>> {
        sqlx::query_as::<_, PgBeaconSecret>(
            "SELECT * FROM beacon_secrets ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch beacon secrets: {}", e)))
    }

    async fn create(&self, secret: &PgBeaconSecret) -> Result<String> {
        let result = sqlx::query_as::<_, (i32,)>(
            r#"
            INSERT INTO beacon_secrets (beacon_id, private_key)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(secret.beacon_id)
        .bind(&secret.private_key)
        .fetch_one(self.pool.inner())
        .await
        .map_err(|e| {
            ServerError::DatabaseQuery(format!("Failed to create beacon secret: {}", e))
        })?;

        Ok(result.0.to_string())
    }

    async fn update(&self, secret: &PgBeaconSecret) -> Result<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE beacon_secrets
            SET beacon_id = $1, private_key = $2
            WHERE id = $3
            "#,
        )
        .bind(secret.beacon_id)
        .bind(&secret.private_key)
        .bind(secret.id)
        .execute(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to update beacon secret: {}", e)))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Beacon secret not found".to_string()));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;

        let rows_affected = sqlx::query("DELETE FROM beacon_secrets WHERE id = $1")
            .bind(int_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to delete beacon secret: {}", e))
            })?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Beacon secret not found".to_string()));
        }

        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        let result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM beacon_secrets")
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to count beacon secrets: {}", e))
            })?;

        Ok(result.0)
    }
}

#[async_trait]
impl IntRepository<PgBeaconSecret> for BeaconSecretRepository {
    async fn get_by_int(&self, id: i32) -> Result<Option<PgBeaconSecret>> {
        sqlx::query_as::<_, PgBeaconSecret>("SELECT * FROM beacon_secrets WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to fetch beacon secret: {}", e))
            })
    }
}

// ============================================================================
// User Public Key Repository (INTEGER)
// ============================================================================

use super::models::extras::PgUserPublicKey;

pub struct UserPublicKeyRepository {
    pool: PgPool,
}

impl UserPublicKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<PgUserPublicKey>> {
        sqlx::query_as::<_, PgUserPublicKey>(
            "SELECT * FROM user_public_keys WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch user public keys: {}", e)))
    }

    pub async fn get_by_device_id(&self, device_id: &str) -> Result<Option<PgUserPublicKey>> {
        sqlx::query_as::<_, PgUserPublicKey>("SELECT * FROM user_public_keys WHERE device_id = $1")
            .bind(device_id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!(
                    "Failed to fetch user public key by device_id: {}",
                    e
                ))
            })
    }

    pub async fn get_by_user_and_device(
        &self,
        user_id: Uuid,
        device_id: &str,
    ) -> Result<Option<PgUserPublicKey>> {
        sqlx::query_as::<_, PgUserPublicKey>(
            "SELECT * FROM user_public_keys WHERE user_id = $1 AND device_id = $2",
        )
        .bind(user_id)
        .bind(device_id)
        .fetch_optional(self.pool.inner())
        .await
        .map_err(|e| {
            ServerError::DatabaseQuery(format!(
                "Failed to fetch user public key by user and device: {}",
                e
            ))
        })
    }
}

#[async_trait]
impl Repository<PgUserPublicKey> for UserPublicKeyRepository {
    async fn get_by_id(&self, id: &str) -> Result<Option<PgUserPublicKey>> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;
        self.get_by_int(int_id).await
    }

    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<PgUserPublicKey>> {
        sqlx::query_as::<_, PgUserPublicKey>(
            "SELECT * FROM user_public_keys ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch user public keys: {}", e)))
    }

    async fn create(&self, key: &PgUserPublicKey) -> Result<String> {
        let result = sqlx::query_as::<_, (i32,)>(
            r#"
            INSERT INTO user_public_keys (user_id, public_key, device_id, device_name)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(key.user_id)
        .bind(&key.public_key)
        .bind(&key.device_id)
        .bind(&key.device_name)
        .fetch_one(self.pool.inner())
        .await
        .map_err(|e| {
            ServerError::DatabaseQuery(format!("Failed to create user public key: {}", e))
        })?;

        Ok(result.0.to_string())
    }

    async fn update(&self, key: &PgUserPublicKey) -> Result<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE user_public_keys
            SET user_id = $1, public_key = $2, device_id = $3, device_name = $4
            WHERE id = $5
            "#,
        )
        .bind(key.user_id)
        .bind(&key.public_key)
        .bind(&key.device_id)
        .bind(&key.device_name)
        .bind(key.id)
        .execute(self.pool.inner())
        .await
        .map_err(|e| {
            ServerError::DatabaseQuery(format!("Failed to update user public key: {}", e))
        })?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound(
                "User public key not found".to_string(),
            ));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;

        let rows_affected = sqlx::query("DELETE FROM user_public_keys WHERE id = $1")
            .bind(int_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to delete user public key: {}", e))
            })?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound(
                "User public key not found".to_string(),
            ));
        }

        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        let result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM user_public_keys")
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to count user public keys: {}", e))
            })?;

        Ok(result.0)
    }
}

#[async_trait]
impl IntRepository<PgUserPublicKey> for UserPublicKeyRepository {
    async fn get_by_int(&self, id: i32) -> Result<Option<PgUserPublicKey>> {
        sqlx::query_as::<_, PgUserPublicKey>("SELECT * FROM user_public_keys WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to fetch user public key: {}", e))
            })
    }
}

// ============================================================================
// Firmware Repository (INTEGER)
// ============================================================================

use super::models::extras::PgFirmware;

pub struct FirmwareRepository {
    pool: PgPool,
}

impl FirmwareRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_version(&self, version: &str) -> Result<Option<PgFirmware>> {
        sqlx::query_as::<_, PgFirmware>("SELECT * FROM firmwares WHERE version = $1")
            .bind(version)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| {
                ServerError::DatabaseQuery(format!("Failed to fetch firmware by version: {}", e))
            })
    }

    pub async fn get_by_chip(&self, chip: &str) -> Result<Vec<PgFirmware>> {
        sqlx::query_as::<_, PgFirmware>(
            "SELECT * FROM firmwares WHERE chip = $1 ORDER BY created_at DESC",
        )
        .bind(chip)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| {
            ServerError::DatabaseQuery(format!("Failed to fetch firmwares by chip: {}", e))
        })
    }

    pub async fn get_stable_by_chip(&self, chip: &str) -> Result<Vec<PgFirmware>> {
        sqlx::query_as::<_, PgFirmware>(
            "SELECT * FROM firmwares WHERE chip = $1 AND is_stable = true ORDER BY created_at DESC",
        )
        .bind(chip)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch stable firmwares: {}", e)))
    }

    pub async fn get_latest_stable(&self, chip: &str) -> Result<Option<PgFirmware>> {
        sqlx::query_as::<_, PgFirmware>(
            "SELECT * FROM firmwares WHERE chip = $1 AND is_stable = true ORDER BY created_at DESC LIMIT 1",
        )
        .bind(chip)
        .fetch_optional(self.pool.inner())
        .await
        .map_err(|e| {
            ServerError::DatabaseQuery(format!("Failed to fetch latest stable firmware: {}", e))
        })
    }
}

#[async_trait]
impl Repository<PgFirmware> for FirmwareRepository {
    async fn get_by_id(&self, id: &str) -> Result<Option<PgFirmware>> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;
        self.get_by_int(int_id).await
    }

    async fn get_all(&self, offset: i64, limit: i64) -> Result<Vec<PgFirmware>> {
        sqlx::query_as::<_, PgFirmware>(
            "SELECT * FROM firmwares ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch firmwares: {}", e)))
    }

    async fn create(&self, firmware: &PgFirmware) -> Result<String> {
        let result = sqlx::query_as::<_, (i32,)>(
            r#"
            INSERT INTO firmwares (version, chip, file_name, file_size, checksum, release_notes, is_stable)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(&firmware.version)
        .bind(&firmware.chip)
        .bind(&firmware.file_name)
        .bind(firmware.file_size)
        .bind(&firmware.checksum)
        .bind(&firmware.release_notes)
        .bind(firmware.is_stable)
        .fetch_one(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to create firmware: {}", e)))?;

        Ok(result.0.to_string())
    }

    async fn update(&self, firmware: &PgFirmware) -> Result<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE firmwares
            SET version = $1, chip = $2, file_name = $3, file_size = $4,
                checksum = $5, release_notes = $6, is_stable = $7
            WHERE id = $8
            "#,
        )
        .bind(&firmware.version)
        .bind(&firmware.chip)
        .bind(&firmware.file_name)
        .bind(firmware.file_size)
        .bind(&firmware.checksum)
        .bind(&firmware.release_notes)
        .bind(firmware.is_stable)
        .bind(firmware.id)
        .execute(self.pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to update firmware: {}", e)))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Firmware not found".to_string()));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let int_id: i32 = id
            .parse()
            .map_err(|_| ServerError::InvalidInput("Invalid integer ID format".to_string()))?;

        let rows_affected = sqlx::query("DELETE FROM firmwares WHERE id = $1")
            .bind(int_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to delete firmware: {}", e)))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ServerError::NotFound("Firmware not found".to_string()));
        }

        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        let result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM firmwares")
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to count firmwares: {}", e)))?;

        Ok(result.0)
    }
}

#[async_trait]
impl IntRepository<PgFirmware> for FirmwareRepository {
    async fn get_by_int(&self, id: i32) -> Result<Option<PgFirmware>> {
        sqlx::query_as::<_, PgFirmware>("SELECT * FROM firmwares WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch firmware: {}", e)))
    }
}
