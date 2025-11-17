#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use core::fmt::Display;

#[cfg(feature = "postgres")]
use crate::schema::postgres::PgPoint;

/// Entity schema - represents a physical building or complex (mall, hospital, etc.)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "sql", derive(sqlx::FromRow))]
#[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), derive(ts_rs::TS))]
#[cfg_attr(
    all(feature = "ts-rs", not(feature = "postgres")),
    ts(export, export_to = "generated/")
)]
pub struct Entity {
    #[cfg(feature = "postgres")]
    pub id: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub id: String,
    pub r#type: EntityType,
    pub name: String,
    pub description: Option<String>,
    #[cfg(feature = "postgres")]
    pub point_min: PgPoint,
    #[cfg(not(feature = "postgres"))]
    pub point_min: (f64, f64),
    #[cfg(feature = "postgres")]
    pub point_max: PgPoint,
    #[cfg(not(feature = "postgres"))]
    pub point_max: (f64, f64),
    pub altitude_min: Option<f64>,
    pub altitude_max: Option<f64>,
    pub nation: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub tags: Vec<String>,
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

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
#[cfg_attr(feature = "sql", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sql",
    sqlx(type_name = "VARCHAR", rename_all = "PascalCase")
)]
pub enum EntityType {
    Mall,
    Transportation,
    School,
    Hospital,
}

impl Display for EntityType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EntityType::Mall => write!(f, "Mall"),
            EntityType::Transportation => write!(f, "Transportation"),
            EntityType::School => write!(f, "School"),
            EntityType::Hospital => write!(f, "Hospital"),
        }
    }
}

#[cfg(all(feature = "postgres", feature = "sql"))]
use crate::schema::repository::UuidRepository;

#[cfg(all(feature = "postgres", feature = "sql"))]
#[async_trait::async_trait]
impl UuidRepository for Entity {
    async fn create(pool: &sqlx::PgPool, item: &Self) -> sqlx::Result<()> {
        sqlx::query(
            r#"INSERT INTO entities (id, type, name, description, point_min, point_max,
                                    altitude_min, altitude_max, nation, region, city, tags)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
        )
        .bind(item.id)
        .bind(item.r#type.to_string())
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.point_min)
        .bind(item.point_max)
        .bind(item.altitude_min)
        .bind(item.altitude_max)
        .bind(&item.nation)
        .bind(&item.region)
        .bind(&item.city)
        .bind(&item.tags)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn get_by_uuid(pool: &sqlx::PgPool, uuid: uuid::Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            r#"SELECT id, type, name, description, point_min, point_max,
                      altitude_min, altitude_max, nation, region, city, tags,
                      created_at, updated_at
               FROM entities WHERE id = $1"#,
        )
        .bind(uuid)
        .fetch_optional(pool)
        .await
    }

    async fn update(pool: &sqlx::PgPool, item: &Self) -> sqlx::Result<()> {
        sqlx::query(
            r#"UPDATE entities
               SET type = $2, name = $3, description = $4, point_min = $5, point_max = $6,
                   altitude_min = $7, altitude_max = $8, nation = $9, region = $10, city = $11,
                   tags = $12
               WHERE id = $1"#,
        )
        .bind(item.id)
        .bind(item.r#type.to_string())
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.point_min)
        .bind(item.point_max)
        .bind(item.altitude_min)
        .bind(item.altitude_max)
        .bind(&item.nation)
        .bind(&item.region)
        .bind(&item.city)
        .bind(&item.tags)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::PgPool, uuid: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM entities WHERE id = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn list(pool: &sqlx::PgPool, offset: i64, limit: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            r#"SELECT id, type, name, description, point_min, point_max,
                      altitude_min, altitude_max, nation, region, city, tags,
                      created_at, updated_at
               FROM entities
               ORDER BY created_at DESC
               LIMIT $1 OFFSET $2"#,
        )
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
    ) -> sqlx::Result<Vec<Self>> {
        let like_pattern = format!("%{}%", query);
        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, type, name, description, point_min, point_max,
                          altitude_min, altitude_max, nation, region, city, tags,
                          created_at, updated_at
                   FROM entities
                   WHERE name ILIKE $1 OR description ILIKE $1
                   ORDER BY {} {}
                   LIMIT $2 OFFSET $3"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, type, name, description, point_min, point_max,
                          altitude_min, altitude_max, nation, region, city, tags,
                          created_at, updated_at
                   FROM entities
                   WHERE name LIKE $1 OR description LIKE $1
                   ORDER BY {} {}
                   LIMIT $2 OFFSET $3"#,
                order_by, direction
            )
        };

        sqlx::query_as::<_, Self>(&sql)
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }
}
