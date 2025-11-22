#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(feature = "postgres")]
use crate::schema::postgis::PgPoint;
#[cfg(feature = "sql")]
use crate::traits::{IntRepository, IntRepositoryInArea};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "postgres")]
use sqlx::PgPool;
#[cfg(feature = "sqlite")]
use sqlx::SqlitePool;
#[cfg(feature = "postgres")]
use uuid::Uuid;

/// Beacon schema - represents a physical BLE beacon device
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "postgres", derive(sqlx::FromRow))]
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

impl core::fmt::Display for BeaconDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BeaconDevice::Esp32 => write!(f, "esp32"),
            BeaconDevice::Esp32C3 => write!(f, "esp32c3"),
            BeaconDevice::Esp32C5 => write!(f, "esp32c5"),
            BeaconDevice::Esp32C6 => write!(f, "esp32c6"),
            BeaconDevice::Esp32S3 => write!(f, "esp32s3"),
        }
    }
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

impl core::fmt::Display for BeaconType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BeaconType::Navigation => write!(f, "navigation"),
            BeaconType::Marketing => write!(f, "marketing"),
            BeaconType::Tracking => write!(f, "tracking"),
            BeaconType::Environmental => write!(f, "environmental"),
            BeaconType::Security => write!(f, "security"),
            BeaconType::Other => write!(f, "other"),
        }
    }
}

impl Beacon {
    pub fn location(&self) -> (f64, f64) {
        #[cfg(feature = "postgres")]
        {
            (self.location.lon(), self.location.lat())
        }
        #[cfg(not(feature = "postgres"))]
        {
            self.location
        }
    }
}

#[cfg(feature = "postgres")]
#[allow(unused)]
fn beacon_from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Beacon> {
    use sqlx::Row;

    let type_json: sqlx::types::Json<serde_json::Value> = row.try_get("type")?;
    let r#type: BeaconType =
        serde_json::from_value(type_json.0).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    let device_json: sqlx::types::Json<serde_json::Value> = row.try_get("device")?;
    let device: BeaconDevice =
        serde_json::from_value(device_json.0).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    Ok(Beacon {
        id: row.try_get("id")?,
        entity_id: row.try_get("entity_id")?,
        area_id: row.try_get("area_id")?,
        merchant_id: row.try_get("merchant_id")?,
        connection_id: row.try_get("connection_id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        r#type,
        location: row.try_get("location")?,
        device,
        mac: row.try_get("mac")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl IntRepository<sqlx::Postgres> for Beacon {
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

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl IntRepositoryInArea<sqlx::Postgres> for Beacon {
    async fn search_in_area(
        pool: &PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        area: i32,
        entity: Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        let like_pattern = format!("%{}%", query);
        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description, type, location, device, mac,
                          created_at, updated_at
                   FROM beacons
                   WHERE entity_id = $1 AND area_id = $2 AND (name ILIKE $3 OR description ILIKE $3 OR mac ILIKE $3)
                   ORDER BY {} {}
                   LIMIT $4 OFFSET $5"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description, type, location, device, mac,
                          created_at, updated_at
                   FROM beacons
                   WHERE entity_id = $1 AND area_id = $2 AND (name LIKE $3 OR description LIKE $3 OR mac LIKE $3)
                   ORDER BY {} {}
                   LIMIT $4 OFFSET $5"#,
                order_by, direction
            )
        };

        let rows = sqlx::query(&sql)
            .bind(entity)
            .bind(area)
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        rows.iter().map(beacon_from_row).collect()
    }
}

// SQLite repository implementation for Beacon
#[cfg(feature = "sqlite")]
use crate::schema::postgis::point_to_wkb;

#[cfg(feature = "sqlite")]
#[async_trait::async_trait]
impl IntRepository<sqlx::Sqlite> for Beacon {
    async fn create(pool: &sqlx::SqlitePool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        let location_wkb = point_to_wkb(item.location)
            .map_err(|e| sqlx::Error::Encode(format!("WKB: {}", e).into()))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        sqlx::query(
            r#"INSERT INTO beacons (entity_id, area_id, merchant_id, connection_id, name, description,
                                   type, location_wkb, device, mac, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
        )
        .bind(entity.to_string())
        .bind(item.area_id)
        .bind(item.merchant_id)
        .bind(item.connection_id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.r#type.to_string())
        .bind(location_wkb)
        .bind(item.device.to_string())
        .bind(&item.mac)
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
            r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description,
                      type, location_wkb, device, mac, created_at, updated_at
               FROM beacons WHERE id = ?1 AND entity_id = ?2"#,
        )
        .bind(id)
        .bind(entity.to_string())
        .fetch_optional(pool)
        .await
    }

    async fn update(pool: &sqlx::SqlitePool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        let location_wkb = point_to_wkb(item.location)
            .map_err(|e| sqlx::Error::Encode(format!("WKB: {}", e).into()))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        sqlx::query(
            r#"UPDATE beacons
               SET area_id = ?3, merchant_id = ?4, connection_id = ?5, name = ?6, description = ?7,
                   type = ?8, location_wkb = ?9, device = ?10, mac = ?11, updated_at = ?12
               WHERE id = ?1 AND entity_id = ?2"#,
        )
        .bind(item.id)
        .bind(entity.to_string())
        .bind(item.area_id)
        .bind(item.merchant_id)
        .bind(item.connection_id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.r#type.to_string())
        .bind(location_wkb)
        .bind(item.device.to_string())
        .bind(&item.mac)
        .bind(now)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::SqlitePool, id: i32, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM beacons WHERE id = ?1 AND entity_id = ?2")
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
            r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description,
                      type, location_wkb, device, mac, created_at, updated_at
               FROM beacons WHERE entity_id = ?1
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
                r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description,
                          type, location_wkb, device, mac, created_at, updated_at
                   FROM beacons
                   WHERE entity_id = ?1 AND (name LIKE ?2 COLLATE NOCASE OR description LIKE ?2 COLLATE NOCASE OR mac LIKE ?2 COLLATE NOCASE)
                   ORDER BY {} {}
                   LIMIT ?3 OFFSET ?4"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description,
                          type, location_wkb, device, mac, created_at, updated_at
                   FROM beacons
                   WHERE entity_id = ?1 AND (name LIKE ?2 OR description LIKE ?2 OR mac LIKE ?2)
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

#[cfg(feature = "sqlite")]
#[async_trait::async_trait]
impl IntRepositoryInArea<sqlx::Sqlite> for Beacon {
    async fn search_in_area(
        pool: &SqlitePool,
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
                r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description,
                          type, location_wkb, device, mac, created_at, updated_at
                   FROM beacons
                   WHERE entity_id = ?1 AND area_id = ?2 AND (name LIKE ?3 COLLATE NOCASE OR description LIKE ?3 COLLATE NOCASE OR mac LIKE ?3 COLLATE NOCASE)
                   ORDER BY {} {}
                   LIMIT ?4 OFFSET ?5"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, area_id, merchant_id, connection_id, name, description,
                          type, location_wkb, device, mac, created_at, updated_at
                   FROM beacons
                   WHERE entity_id = ?1 AND area_id = ?2 AND (name LIKE ?3 OR description LIKE ?3 OR mac LIKE ?3)
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
}
