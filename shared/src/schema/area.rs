#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(all(feature = "alloc", not(feature = "postgres")))]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "postgres")]
use crate::schema::postgis::PgPolygon;
use core::fmt::{Display, Formatter};

/// Area schema - represents a physical area in the mall/building
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(all(feature = "postgres", feature = "sql"), derive(FromRow))]
#[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), derive(ts_rs::TS))]
#[cfg_attr(
    all(feature = "ts-rs", not(feature = "postgres")),
    ts(export, export_to = "generated/")
)]
pub struct Area {
    #[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), ts(type = "string"))]
    pub id: i32,
    #[cfg(feature = "postgres")]
    pub entity_id: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), ts(type = "string"))]
    pub entity_id: String,
    pub name: String,
    pub description: Option<String>,
    /// Unique identifier for the area for displaying in the beacon name.
    pub beacon_code: String,
    pub floor_type: Option<String>,
    pub floor_name: Option<i32>,
    #[cfg(feature = "postgres")]
    pub polygon: PgPolygon,
    #[cfg(not(feature = "postgres"))]
    pub polygon: Vec<(f64, f64)>,
    #[cfg(feature = "postgres")]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub created_at: Option<i64>, // Timestamp in milliseconds
    #[cfg(feature = "postgres")]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub updated_at: Option<i64>, // Timestamp in milliseconds
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub struct Floor {
    pub r#type: FloorType,
    pub name: u32,
}

impl From<Floor> for i32 {
    fn from(val: Floor) -> i32 {
        match val.r#type {
            FloorType::Level => val.name as i32 + 1, // Level 0 is Ground, Level 1 is First
            FloorType::Floor => val.name as i32,
            FloorType::Basement => -(val.name as i32),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub enum FloorType {
    /// European/UK style, e.g., "Ground," "First," "Second"
    Level,
    /// US style, e.g., "1st," "2nd," "3rd"
    Floor,
    /// Universal basement
    Basement,
}

impl Display for FloorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            FloorType::Level => write!(f, "Level"),
            FloorType::Floor => write!(f, "Floor"),
            FloorType::Basement => write!(f, "Basement"),
        }
    }
}

impl Display for Floor {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.r#type {
            FloorType::Level => write!(f, "L{}", self.name),
            FloorType::Floor => write!(f, "{}F", self.name),
            FloorType::Basement => write!(f, "B{}", self.name),
        }
    }
}

#[cfg(all(feature = "postgres", feature = "sql"))]
use crate::schema::repository::IntRepository;

#[cfg(all(feature = "postgres", feature = "sql"))]
#[async_trait::async_trait]
impl IntRepository for Area {
    async fn create(pool: &sqlx::PgPool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query(
            r#"INSERT INTO areas (entity_id, name, description, floor_type, floor_name, beacon_code, polygon)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#
        )
        .bind(entity)
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.floor_type)
        .bind(item.floor_name)
        .bind(&item.beacon_code)
        .bind(&item.polygon)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(
        pool: &sqlx::PgPool,
        id: i32,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            r#"SELECT id, entity_id, name, description, floor_type, floor_name, beacon_code, polygon,
                      created_at, updated_at
               FROM areas WHERE id = $1 AND entity_id = $2"#
        )
        .bind(id)
        .bind(entity)
        .fetch_optional(pool)
        .await
    }

    async fn update(pool: &sqlx::PgPool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query(
            r#"UPDATE areas
               SET name = $3, description = $4, floor_type = $5, floor_name = $6, beacon_code = $7, polygon = $8
               WHERE id = $1 AND entity_id = $2"#
        )
        .bind(item.id)
        .bind(entity)
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.floor_type)
        .bind(item.floor_name)
        .bind(&item.beacon_code)
        .bind(&item.polygon)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::PgPool, id: i32, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM areas WHERE id = $1 AND entity_id = $2")
            .bind(id)
            .bind(entity)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn list(
        pool: &sqlx::PgPool,
        offset: i64,
        limit: i64,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            r#"SELECT id, entity_id, name, description, floor_type, floor_name, beacon_code, polygon,
                      created_at, updated_at
               FROM areas WHERE entity_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#
        )
        .bind(entity)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    async fn search(
        pool: &sqlx::PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        let like_pattern = format!("%{}%", query);
        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, entity_id, name, description, floor_type, floor_name, beacon_code, polygon,
                          created_at, updated_at
                   FROM areas
                   WHERE entity_id = $1 AND (name ILIKE $2 OR description ILIKE $2 OR beacon_code ILIKE $2)
                   ORDER BY {} {}
                   LIMIT $3 OFFSET $4"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, name, description, floor_type, floor_name, beacon_code, polygon,
                          created_at, updated_at
                   FROM areas
                   WHERE entity_id = $1 AND (name LIKE $2 OR description LIKE $2 OR beacon_code LIKE $2)
                   ORDER BY {} {}
                   LIMIT $3 OFFSET $4"#,
                order_by, direction
            )
        };

        sqlx::query_as::<_, Self>(&sql)
            .bind(entity)
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }
}

// SQLite repository implementation for Area
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
use crate::schema::postgis::polygon_to_wkb;
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
use crate::schema::repository::IntRepository;

#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
#[async_trait::async_trait]
impl IntRepository for Area {
    async fn create(pool: &sqlx::SqlitePool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        let polygon_wkb = polygon_to_wkb(&item.polygon)
            .map_err(|e| sqlx::Error::Encode(format!("WKB: {}", e).into()))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        sqlx::query(
            r#"INSERT INTO areas (entity_id, name, description, floor_type, floor_name, beacon_code, polygon_wkb,
                                created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
        )
        .bind(entity.to_string())
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.floor_type)
        .bind(item.floor_name)
        .bind(&item.beacon_code)
        .bind(polygon_wkb)
        .bind(item.created_at.unwrap_or(now))
        .bind(item.updated_at.unwrap_or(now))
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(
        pool: &sqlx::SqlitePool,
        id: i32,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            r#"SELECT id, entity_id, name, description, floor_type, floor_name, beacon_code, polygon_wkb,
                      created_at, updated_at
               FROM areas WHERE id = ?1 AND entity_id = ?2"#,
        )
        .bind(id)
        .bind(entity.to_string())
        .fetch_optional(pool)
        .await
    }

    async fn update(pool: &sqlx::SqlitePool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        let polygon_wkb = polygon_to_wkb(&item.polygon)
            .map_err(|e| sqlx::Error::Encode(format!("WKB: {}", e).into()))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        sqlx::query(
            r#"UPDATE areas
               SET name = ?3, description = ?4, floor_type = ?5, floor_name = ?6, beacon_code = ?7,
                   polygon_wkb = ?8, updated_at = ?9
               WHERE id = ?1 AND entity_id = ?2"#,
        )
        .bind(item.id)
        .bind(entity.to_string())
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.floor_type)
        .bind(item.floor_name)
        .bind(&item.beacon_code)
        .bind(polygon_wkb)
        .bind(now)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::SqlitePool, id: i32, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM areas WHERE id = ?1 AND entity_id = ?2")
            .bind(id)
            .bind(entity.to_string())
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn list(
        pool: &sqlx::SqlitePool,
        offset: i64,
        limit: i64,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            r#"SELECT id, entity_id, name, description, floor_type, floor_name, beacon_code, polygon_wkb,
                      created_at, updated_at
               FROM areas WHERE entity_id = ?1
               ORDER BY created_at DESC
               LIMIT ?2 OFFSET ?3"#,
        )
        .bind(entity.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    async fn search(
        pool: &sqlx::SqlitePool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        let like_pattern = format!("%{}%", query);
        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, entity_id, name, description, floor_type, floor_name, beacon_code, polygon_wkb,
                          created_at, updated_at
                   FROM areas
                   WHERE entity_id = ?1 AND (name LIKE ?2 COLLATE NOCASE OR description LIKE ?2 COLLATE NOCASE OR beacon_code LIKE ?2 COLLATE NOCASE)
                   ORDER BY {} {}
                   LIMIT ?3 OFFSET ?4"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, name, description, floor_type, floor_name, beacon_code, polygon_wkb,
                          created_at, updated_at
                   FROM areas
                   WHERE entity_id = ?1 AND (name LIKE ?2 OR description LIKE ?2 OR beacon_code LIKE ?2)
                   ORDER BY {} {}
                   LIMIT ?3 OFFSET ?4"#,
                order_by, direction
            )
        };

        sqlx::query_as::<_, Self>(&sql)
            .bind(entity.to_string())
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }
}
