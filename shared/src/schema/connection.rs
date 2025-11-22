#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "postgres")]
use crate::schema::postgis::PgPoint;

#[cfg(feature = "sql")]
use crate::traits::{IntRepository, IntRepositoryInArea};
use core::fmt::Display;

pub type ConnectedArea = (i32, f64, f64, bool);

/// Connection schema - represents connections between areas (gates, elevators, etc.)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(all(feature = "postgres", feature = "sql"), derive(sqlx::FromRow))]
#[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), derive(ts_rs::TS))]
#[cfg_attr(
    all(feature = "ts-rs", not(feature = "postgres")),
    ts(export, export_to = "generated/")
)]
pub struct Connection {
    pub id: i32,
    #[cfg(feature = "postgres")]
    pub entity_id: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub entity_id: String,
    pub name: String,
    pub description: Option<String>,
    pub r#type: ConnectionType,
    /// List of Area IDs that this connection links
    #[cfg_attr(
        feature = "ts-rs",
        ts(type = "Array<[number, number, number, boolean]>")
    )]
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub connected_areas: Vec<ConnectedArea>,
    /// List of `(start_time, end_time)` in milliseconds on a 24-hour clock
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub available_period: Vec<(i32, i32)>,
    pub tags: Vec<String>,
    /// Ground location if connection goes outside (optional)
    #[cfg(feature = "postgres")]
    pub gnd: Option<PgPoint>,
    #[cfg(not(feature = "postgres"))]
    pub gnd: Option<(f64, f64)>,
    #[cfg(feature = "chrono")]
    #[cfg_attr(
        all(feature = "serde", not(feature = "chrono")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg(not(feature = "chrono"))]
    #[cfg_attr(
        all(feature = "serde", not(feature = "chrono")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub created_at: Option<String>, // Timestamp in milliseconds
    #[cfg(feature = "chrono")]
    #[cfg_attr(
        all(feature = "serde", not(feature = "chrono")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg(not(feature = "chrono"))]
    #[cfg_attr(
        all(feature = "serde", not(feature = "chrono")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub updated_at: Option<String>, // Timestamp in milliseconds
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
#[cfg_attr(feature = "sql", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sql",
    sqlx(type_name = "VARCHAR", rename_all = "kebab-case")
)]
pub enum ConnectionType {
    Gate,
    Escalator,
    Elevator,
    Stairs,
    Rail,
    Shuttle,
}

impl Display for ConnectionType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConnectionType::Gate => write!(f, "gate"),
            ConnectionType::Escalator => write!(f, "escalator"),
            ConnectionType::Elevator => write!(f, "elevator"),
            ConnectionType::Stairs => write!(f, "stairs"),
            ConnectionType::Rail => write!(f, "rail"),
            ConnectionType::Shuttle => write!(f, "shuttle"),
        }
    }
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl IntRepository<sqlx::Postgres> for Connection {
    async fn create(pool: &sqlx::PgPool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        // Serialize connected_areas to JSON
        let connected_areas_json = serde_json::to_value(&item.connected_areas)
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        // Serialize available_period to JSON
        let available_period_json = serde_json::to_value(&item.available_period)
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        // Serialize tags to JSON
        let tags_json =
            serde_json::to_value(&item.tags).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        sqlx::query(
            r#"INSERT INTO connections (entity_id, name, description, type, connected_areas,
                                        available_period, tags, gnd)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(entity)
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.r#type)
        .bind(connected_areas_json)
        .bind(available_period_json)
        .bind(tags_json)
        .bind(item.gnd)
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
            r#"SELECT id, entity_id, name, description, type, connected_areas,
                      available_period, tags, gnd, created_at, updated_at
               FROM connections WHERE id = $1 AND entity_id = $2"#,
        )
        .bind(id)
        .bind(entity)
        .fetch_optional(pool)
        .await
    }

    async fn update(
        pool: &sqlx::PgPool,
        id: i32,
        item: &Self,
        entity: uuid::Uuid,
    ) -> sqlx::Result<()> {
        // Serialize connected_areas to JSON
        let connected_areas_json = serde_json::to_value(&item.connected_areas)
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        // Serialize available_period to JSON
        let available_period_json = serde_json::to_value(&item.available_period)
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        // Serialize tags to JSON
        let tags_json =
            serde_json::to_value(&item.tags).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        sqlx::query(
            r#"UPDATE connections
               SET name = $3, description = $4, type = $5, connected_areas = $6,
                   available_period = $7, tags = $8, gnd = $9
               WHERE id = $1 AND entity_id = $2"#,
        )
        .bind(id)
        .bind(entity)
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.r#type)
        .bind(connected_areas_json)
        .bind(available_period_json)
        .bind(tags_json)
        .bind(item.gnd)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::PgPool, id: i32, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM connections WHERE id = $1 AND entity_id = $2")
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
            r#"SELECT id, entity_id, name, description, type, connected_areas,
                      available_period, tags, gnd, created_at, updated_at
               FROM connections WHERE entity_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
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
                r#"SELECT id, entity_id, name, description, type, connected_areas,
                          available_period, tags, gnd, created_at, updated_at
                   FROM connections
                   WHERE entity_id = $1 AND (name ILIKE $2 OR description ILIKE $2)
                   ORDER BY {} {}
                   LIMIT $3 OFFSET $4"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, name, description, type, connected_areas,
                          available_period, tags, gnd, created_at, updated_at
                   FROM connections
                   WHERE entity_id = $1 AND (name LIKE $2 OR description LIKE $2)
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

    async fn count(
        pool: &sqlx::PgPool,
        entity: uuid::Uuid,
        query: &str,
        case_insensitive: bool,
    ) -> sqlx::Result<i64> {
        let like_pattern = format!("%{}%", query);

        let sql = if case_insensitive {
            r#"SELECT COUNT(*) as count
               FROM connections
               WHERE entity_id = $1 AND (name ILIKE $2 OR description ILIKE $2)"#
        } else {
            r#"SELECT COUNT(*) as count
               FROM connections
               WHERE entity_id = $1 AND (name LIKE $2 OR description LIKE $2)"#
        };

        let row: (i64,) = sqlx::query_as(sql)
            .bind(entity)
            .bind(&like_pattern)
            .fetch_one(pool)
            .await?;

        Ok(row.0)
    }
}

#[async_trait::async_trait]
#[cfg(feature = "postgres")]
impl IntRepositoryInArea<sqlx::Postgres> for Connection {
    async fn search_in_area(
        pool: &sqlx::PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        area: i32,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        let like_pattern = format!("%{}%", query);
        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, entity_id, name, description, type, connected_areas,
                          available_period, tags, gnd, created_at, updated_at
                   FROM connections
                   WHERE entity_id = $1 AND $2 = ANY (SELECT (ca).0 FROM
                       jsonb_to_recordset(connected_areas) AS ca(area_id int, x double precision, y double precision, primary_connection boolean))
                       AND (name ILIKE $3 OR description ILIKE $3)
                   ORDER BY {} {}
                   LIMIT $4 OFFSET $5"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, name, description, type, connected_areas,
                          available_period, tags, gnd, created_at, updated_at
                   FROM connections
                   WHERE entity_id = $1 AND $2 = ANY (SELECT (ca).0 FROM
                       jsonb_to_recordset(connected_areas) AS ca(area_id int, x double precision, y double precision, primary_connection boolean))
                       AND (name LIKE $3 OR description LIKE $3)
                   ORDER BY {} {}
                   LIMIT $4 OFFSET $5"#,
                order_by, direction
            )
        };

        sqlx::query_as::<_, Self>(&sql)
            .bind(entity)
            .bind(area)
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    async fn count_in_area(
        pool: &sqlx::PgPool,
        entity: uuid::Uuid,
        area: i32,
        query: &str,
        case_insensitive: bool,
    ) -> sqlx::Result<i64> {
        let like_pattern = format!("%{}%", query);

        let sql = if case_insensitive {
            r#"SELECT COUNT(*) as count
               FROM connections
               WHERE entity_id = $1 AND $2 = ANY (SELECT (ca).0 FROM
                   jsonb_to_recordset(connected_areas) AS ca(area_id int, x double precision, y double precision, primary_connection boolean))
                   AND (name ILIKE $3 OR description ILIKE $3)"#
        } else {
            r#"SELECT COUNT(*) as count
               FROM connections
               WHERE entity_id = $1 AND $2 = ANY (SELECT (ca).0 FROM
                   jsonb_to_recordset(connected_areas) AS ca(area_id int, x double precision, y double precision, primary_connection boolean))
                   AND (name LIKE $3 OR description LIKE $3)"#
        };

        let row: (i64,) = sqlx::query_as(sql)
            .bind(entity)
            .bind(area)
            .bind(&like_pattern)
            .fetch_one(pool)
            .await?;

        Ok(row.0)
    }
}

// SQLite repository implementation for Connection
#[cfg(feature = "sqlite")]
use crate::schema::postgis::point_to_wkb;

#[cfg(feature = "sqlite")]
#[async_trait::async_trait]
impl IntRepository<sqlx::Sqlite> for Connection {
    async fn create(pool: &sqlx::SqlitePool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        let gnd_wkb = item
            .gnd
            .map(point_to_wkb)
            .transpose()
            .map_err(|e| sqlx::Error::Encode(format!("WKB encode: {}", e).into()))?;
        let connected_areas_json = serde_json::to_string(&item.connected_areas)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let available_period_json = serde_json::to_string(&item.available_period)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let tags_json = serde_json::to_string(&item.tags)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            r#"INSERT INTO connections (entity_id, name, description, type, connected_areas,
                                        available_period, tags, gnd_wkb, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        )
        .bind(entity.to_string())
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.r#type.to_string())
        .bind(connected_areas_json)
        .bind(available_period_json)
        .bind(tags_json)
        .bind(gnd_wkb)
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
            r#"SELECT id, entity_id, name, description, type, connected_areas,
                      available_period, tags, gnd_wkb, created_at, updated_at
               FROM connections WHERE id = ?1 AND entity_id = ?2"#,
        )
        .bind(id)
        .bind(entity.to_string())
        .fetch_optional(pool)
        .await
    }

    async fn update(
        pool: &sqlx::SqlitePool,
        id: i32,
        item: &Self,
        entity: uuid::Uuid,
    ) -> sqlx::Result<()> {
        let gnd_wkb = item
            .gnd
            .map(point_to_wkb)
            .transpose()
            .map_err(|e| sqlx::Error::Encode(format!("WKB encode: {}", e).into()))?;
        let connected_areas_json = serde_json::to_string(&item.connected_areas)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let available_period_json = serde_json::to_string(&item.available_period)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let tags_json = serde_json::to_string(&item.tags)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            r#"UPDATE connections
               SET name = ?3, description = ?4, type = ?5, connected_areas = ?6,
                   available_period = ?7, tags = ?8, gnd_wkb = ?9, updated_at = ?10
               WHERE id = ?1 AND entity_id = ?2"#,
        )
        .bind(id)
        .bind(entity.to_string())
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.r#type.to_string())
        .bind(connected_areas_json)
        .bind(available_period_json)
        .bind(tags_json)
        .bind(gnd_wkb)
        .bind(now)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::SqlitePool, id: i32, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM connections WHERE id = ?1 AND entity_id = ?2")
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
            r#"SELECT id, entity_id, name, description, type, connected_areas,
                      available_period, tags, gnd_wkb, created_at, updated_at
               FROM connections WHERE entity_id = ?1
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
                r#"SELECT id, entity_id, name, description, type, connected_areas,
                          available_period, tags, gnd_wkb, created_at, updated_at
                   FROM connections
                   WHERE entity_id = ?1 AND (name LIKE ?2 COLLATE NOCASE OR description LIKE ?2 COLLATE NOCASE)
                   ORDER BY {} {}
                   LIMIT ?3 OFFSET ?4"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, name, description, type, connected_areas,
                          available_period, tags, gnd_wkb, created_at, updated_at
                   FROM connections
                   WHERE entity_id = ?1 AND (name LIKE ?2 OR description LIKE ?2)
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

    async fn count(
        pool: &sqlx::SqlitePool,
        entity: uuid::Uuid,
        query: &str,
        case_insensitive: bool,
    ) -> sqlx::Result<i64> {
        let like_pattern = format!("%{}%", query);

        let sql = if case_insensitive {
            r#"SELECT COUNT(*) as count
               FROM connections
               WHERE entity_id = ?1 AND (name LIKE ?2 COLLATE NOCASE OR description LIKE ?2 COLLATE NOCASE)"#
        } else {
            r#"SELECT COUNT(*) as count
               FROM connections
               WHERE entity_id = ?1 AND (name LIKE ?2 OR description LIKE ?2)"#
        };

        let row: (i64,) = sqlx::query_as(sql)
            .bind(entity.to_string())
            .bind(&like_pattern)
            .fetch_one(pool)
            .await?;

        Ok(row.0)
    }
}

#[async_trait::async_trait]
#[cfg(feature = "sqlite")]
impl IntRepositoryInArea<sqlx::Sqlite> for Connection {
    async fn search_in_area(
        pool: &sqlx::SqlitePool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        area: i32,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        let like_pattern = format!("%{}%", query);

        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, entity_id, name, description, type, connected_areas,
                          available_period, tags, gnd_wkb, created_at, updated_at
                   FROM connections
                   WHERE entity_id = ?1 AND EXISTS (
                       SELECT 1 FROM json_each(connected_areas)
                       WHERE json_extract(value, '$[0]') = ?2
                   ) AND (name LIKE ?3 COLLATE NOCASE OR description LIKE ?3 COLLATE NOCASE)
                   ORDER BY {} {}
                   LIMIT ?4 OFFSET ?5"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, name, description, type, connected_areas,
                          available_period, tags, gnd_wkb, created_at, updated_at
                   FROM connections
                   WHERE entity_id = ?1 AND EXISTS (
                       SELECT 1 FROM json_each(connected_areas)
                       WHERE json_extract(value, '$[0]') = ?2
                   ) AND (name LIKE ?3 OR description LIKE ?3)
                   ORDER BY {} {}
                   LIMIT ?4 OFFSET ?5"#,
                order_by, direction
            )
        };

        sqlx::query_as::<_, Self>(&sql)
            .bind(entity.to_string())
            .bind(area)
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    async fn count_in_area(
        pool: &sqlx::SqlitePool,
        entity: uuid::Uuid,
        area: i32,
        query: &str,
        case_insensitive: bool,
    ) -> sqlx::Result<i64> {
        let like_pattern = format!("%{}%", query);

        let sql = if case_insensitive {
            r#"SELECT COUNT(*) as count
               FROM connections
               WHERE entity_id = ?1 AND EXISTS (
                   SELECT 1 FROM json_each(connected_areas)
                   WHERE json_extract(value, '$[0]') = ?2
               ) AND (name LIKE ?3 COLLATE NOCASE OR description LIKE ?3 COLLATE NOCASE)"#
        } else {
            r#"SELECT COUNT(*) as count
               FROM connections
               WHERE entity_id = ?1 AND EXISTS (
                   SELECT 1 FROM json_each(connected_areas)
                   WHERE json_extract(value, '$[0]') = ?2
               ) AND (name LIKE ?3 OR description LIKE ?3)"#
        };

        let row: (i64,) = sqlx::query_as(sql)
            .bind(entity.to_string())
            .bind(area)
            .bind(&like_pattern)
            .fetch_one(pool)
            .await?;

        Ok(row.0)
    }
}
