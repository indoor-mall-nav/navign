#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "mongodb")]
use bson::oid::ObjectId;

#[cfg(all(feature = "mongodb", feature = "serde"))]
use bson::serde_helpers::serialize_object_id_as_hex_string;

#[cfg(all(feature = "mongodb", feature = "serde"))]
use super::utils::serialize_option_object_id_as_hex_string;

/// Beacon schema - represents a physical BLE beacon device
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub struct Beacon {
    #[cfg(feature = "mongodb")]
    #[cfg_attr(
        all(feature = "mongodb", feature = "serde"),
        serde(rename = "_id", serialize_with = "serialize_object_id_as_hex_string",)
    )]
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub id: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub id: String,
    /// Reference to the Entity
    #[cfg(feature = "mongodb")]
    #[cfg_attr(
        all(feature = "mongodb", feature = "serde"),
        serde(serialize_with = "serialize_object_id_as_hex_string",)
    )]
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub entity: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub entity: String,
    /// Reference to the Area where the beacon is located
    #[cfg(feature = "mongodb")]
    #[cfg_attr(
        all(feature = "mongodb", feature = "serde"),
        serde(serialize_with = "serialize_object_id_as_hex_string",)
    )]
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub area: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub area: String,
    /// Optional reference to the Merchant associated with the beacon.
    #[cfg(feature = "mongodb")]
    #[cfg_attr(
        all(feature = "mongodb", feature = "serde"),
        serde(serialize_with = "serialize_option_object_id_as_hex_string")
    )]
    #[cfg_attr(feature = "ts-rs", ts(type = "string | null"))]
    pub merchant: Option<ObjectId>,
    #[cfg(not(feature = "mongodb"))]
    pub merchant: Option<String>,
    /// Optional reference to the Connection associated with the beacon.
    #[cfg(feature = "mongodb")]
    #[cfg_attr(
        all(feature = "mongodb", feature = "serde"),
        serde(serialize_with = "serialize_option_object_id_as_hex_string")
    )]
    #[cfg_attr(feature = "ts-rs", ts(type = "string | null"))]
    pub connection: Option<ObjectId>,
    #[cfg(not(feature = "mongodb"))]
    pub connection: Option<String>,
    /// The ssid of the beacon, typically used for display purposes in BLE scanning.
    pub name: String,
    /// The displaying name of the beacon, which can be used for user-friendly identification.
    pub description: Option<String>,
    /// The type of the beacon, which can indicate its purpose or functionality.
    pub r#type: BeaconType,
    /// The location of the beacon, represented as a pair of coordinates (longitude, latitude).
    pub location: (f64, f64),
    pub device: BeaconDevice,
    pub mac: String,
    pub created_at: i64, // Timestamp in milliseconds
    pub updated_at: i64, // Timestamp in milliseconds
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
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
pub enum BeaconType {
    Navigation,
    Marketing,
    Tracking,
    Environmental,
    Security,
    Other,
}

// Mobile-specific version for SQLite storage
#[cfg(feature = "sql")]
pub mod mobile {
    use super::{BeaconDevice, BeaconType};
    #[cfg(feature = "alloc")]
    use alloc::string::String;
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};
    use sqlx::FromRow;

    #[derive(Debug, Clone, FromRow)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct BeaconMobile {
        pub id: String,
        pub entity: String,
        pub area: String,
        pub merchant: Option<String>,
        pub connection: Option<String>,
        pub name: String,
        pub description: Option<String>,
        pub r#type: String,
        /// Stored as WKT POINT string
        pub location: String,
        pub device: String,
        pub mac: String,
        pub created_at: i64,
        pub updated_at: i64,
    }

    impl BeaconMobile {
        pub fn beacon_type(&self) -> BeaconType {
            match self.r#type.as_str() {
                "navigation" => BeaconType::Navigation,
                "marketing" => BeaconType::Marketing,
                "tracking" => BeaconType::Tracking,
                "environmental" => BeaconType::Environmental,
                "security" => BeaconType::Security,
                _ => BeaconType::Other,
            }
        }

        pub fn beacon_device(&self) -> BeaconDevice {
            match self.device.as_str() {
                "esp32" => BeaconDevice::Esp32,
                "esp32c3" => BeaconDevice::Esp32C3,
                "esp32c5" => BeaconDevice::Esp32C5,
                "esp32c6" => BeaconDevice::Esp32C6,
                "esp32s3" => BeaconDevice::Esp32S3,
                _ => BeaconDevice::Esp32C3,
            }
        }

        #[cfg(feature = "sql")]
        pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS beacons (
                    id VARCHAR(24) PRIMARY KEY,
                    entity VARCHAR(24) NOT NULL,
                    area VARCHAR(24) NOT NULL,
                    merchant VARCHAR(24),
                    connection VARCHAR(24),
                    name TEXT NOT NULL,
                    description TEXT,
                    type TEXT NOT NULL,
                    location TEXT NOT NULL,
                    device TEXT NOT NULL,
                    mac TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                )
                "#,
            )
            .execute(pool)
            .await?;
            Ok(())
        }

        #[cfg(feature = "sql")]
        pub async fn insert(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO beacons (id, entity, area, merchant, connection, name, description, type, location, device, mac, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&self.id)
            .bind(&self.entity)
            .bind(&self.area)
            .bind(&self.merchant)
            .bind(&self.connection)
            .bind(&self.name)
            .bind(&self.description)
            .bind(&self.r#type)
            .bind(&self.location)
            .bind(&self.device)
            .bind(&self.mac)
            .bind(self.created_at)
            .bind(self.updated_at)
            .execute(pool)
            .await?;
            Ok(())
        }

        #[cfg(feature = "sql")]
        pub async fn get_by_id(
            pool: &sqlx::SqlitePool,
            id: &str,
        ) -> Result<Option<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM beacons WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn get_by_mac(
            pool: &sqlx::SqlitePool,
            mac: &str,
        ) -> Result<Option<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM beacons WHERE mac = ?")
                .bind(mac)
                .fetch_optional(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn get_all(pool: &sqlx::SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM beacons")
                .fetch_all(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn get_by_area(
            pool: &sqlx::SqlitePool,
            area: &str,
        ) -> Result<Vec<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM beacons WHERE area = ?")
                .bind(area)
                .fetch_all(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn delete(pool: &sqlx::SqlitePool, id: &str) -> Result<(), sqlx::Error> {
            sqlx::query("DELETE FROM beacons WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
            Ok(())
        }
    }
}
