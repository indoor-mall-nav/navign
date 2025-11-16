#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Beacon schema - represents a physical BLE beacon device
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), derive(ts_rs::TS))]
#[cfg_attr(
    all(feature = "ts-rs", not(feature = "postgres")),
    ts(export, export_to = "generated/")
)]
pub struct Beacon {
    pub id: i32,
    #[cfg(feature = "postgres")]
    pub entity: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), ts(type = "string"))]
    pub entity: String,
    pub area: i32,
    /// Optional reference to the Merchant associated with the beacon.
    #[cfg_attr(
        all(feature = "ts-rs", not(feature = "postgres")),
        ts(type = "number | null")
    )]
    pub merchant: Option<i32>,
    /// Optional reference to the Connection associated with the beacon.
    #[cfg_attr(
        all(feature = "ts-rs", not(feature = "postgres")),
        ts(type = "number | null")
    )]
    pub connection: Option<i32>,
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
