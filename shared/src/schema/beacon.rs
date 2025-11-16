#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "postgres")]
use crate::schema::postgis::PgPoint;

/// Beacon schema - represents a physical BLE beacon device
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "sql", derive(sqlx::FromRow))]
#[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), derive(ts_rs::TS))]
#[cfg_attr(
    all(feature = "ts-rs", not(feature = "postgres")),
    ts(export, export_to = "generated/")
)]
pub struct Beacon {
    pub id: i32,
    #[cfg(feature = "postgres")]
    pub entity_id: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), ts(type = "string"))]
    pub entity_id: String,
    pub area_id: i32,
    /// Optional reference to the Merchant associated with the beacon.
    #[cfg_attr(
        all(feature = "ts-rs", not(feature = "postgres")),
        ts(type = "number | null")
    )]
    pub merchant_id: Option<i32>,
    /// Optional reference to the Connection associated with the beacon.
    #[cfg_attr(
        all(feature = "ts-rs", not(feature = "postgres")),
        ts(type = "number | null")
    )]
    pub connection_id: Option<i32>,
    /// The ssid of the beacon, typically used for display purposes in BLE scanning.
    pub name: String,
    /// The displaying name of the beacon, which can be used for user-friendly identification.
    pub description: Option<String>,
    /// The type of the beacon, which can indicate its purpose or functionality.
    pub r#type: BeaconType,
    /// The location of the beacon, represented as a pair of coordinates (longitude, latitude).
    #[cfg(feature = "postgres")]
    pub location: PgPoint,
    #[cfg(not(feature = "postgres"))]
    pub location: (f64, f64),
    pub device: BeaconDevice,
    pub mac: String,
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
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
#[cfg_attr(feature = "sql", derive(sqlx::Type))]
#[cfg_attr(feature = "sql", sqlx(type_name = "VARCHAR", rename_all = "lowercase"))]
pub enum BeaconDevice {
    Esp32,
    Esp32C3,
    Esp32C5,
    Esp32C6,
    Esp32S3,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
#[cfg_attr(feature = "sql", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sql",
    sqlx(type_name = "VARCHAR", rename_all = "kebab-case")
)]
pub enum BeaconType {
    Navigation,
    Marketing,
    Tracking,
    Environmental,
    Security,
    Other,
}

#[cfg(all(feature = "postgres", feature = "sql"))]
use crate::schema::repository::IntRepository;

#[cfg(all(feature = "postgres", feature = "sql"))]
#[async_trait::async_trait]
impl IntRepository for Beacon {
    async fn create(pool: &sqlx::PgPool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query(
            r#"INSERT INTO beacons (entity_id, area_id, merchant_id, connection_id, name, description, type, location, device, mac)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#
        )
        .bind(entity)
        .bind(item.area_id)
        .bind(item.merchant_id)
        .bind(item.connection_id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.r#type)
        .bind(item.location)
        .bind(&item.device)
        .bind(&item.mac)
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
            r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description, type, location, device, mac,
                      created_at, updated_at
               FROM beacons WHERE id = $1 AND entity_id = $2"#
        )
        .bind(id)
        .bind(entity)
        .fetch_optional(pool)
        .await
    }

    async fn update(pool: &sqlx::PgPool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query(
            r#"UPDATE beacons
               SET area_id = $3, merchant_id = $4, connection_id = $5, name = $6, description = $7,
                   type = $8, location = $9, device = $10, mac = $11
               WHERE id = $1 AND entity_id = $2"#,
        )
        .bind(item.id)
        .bind(entity)
        .bind(item.area_id)
        .bind(item.merchant_id)
        .bind(item.connection_id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.r#type)
        .bind(item.location)
        .bind(&item.device)
        .bind(&item.mac)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::PgPool, id: i32, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM beacons WHERE id = $1 AND entity_id = $2")
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
            r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description, type, location, device, mac,
                      created_at, updated_at
               FROM beacons WHERE entity_id = $1
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
                r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description, type, location, device, mac,
                          created_at, updated_at
                   FROM beacons
                   WHERE entity_id = $1 AND (name ILIKE $2 OR description ILIKE $2 OR mac ILIKE $2)
                   ORDER BY {} {}
                   LIMIT $3 OFFSET $4"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description, type, location, device, mac,
                          created_at, updated_at
                   FROM beacons
                   WHERE entity_id = $1 AND (name LIKE $2 OR description LIKE $2 OR mac LIKE $2)
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
