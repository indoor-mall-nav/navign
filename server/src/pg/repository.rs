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

// Similar repository implementations would be created for:
// - BeaconRepository (INTEGER)
// - MerchantRepository (INTEGER)
// - ConnectionRepository (INTEGER)
// - BeaconSecretRepository (INTEGER)
// - UserPublicKeyRepository (INTEGER)
// - FirmwareRepository (INTEGER)

// For brevity, I'm showing the pattern above. The other repositories follow
// the same structure with IntRepository trait.
