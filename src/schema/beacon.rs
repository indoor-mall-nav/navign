use crate::AppState;
use crate::schema::metadata::{PaginationResponse, PaginationResponseMetadata};
use crate::schema::service::{OneInArea, Service};
use crate::shared::ReadQuery;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use bson::doc;
use bson::oid::ObjectId;
use futures::TryStreamExt;
use mongodb::Database;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Beacon {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// Reference to the Entity
    pub entity: ObjectId,
    /// Reference to the Area where the beacon is located
    pub area: ObjectId,
    /// Optional reference to the Merchant associated with the beacon.
    pub merchant: Option<ObjectId>,
    /// Optional reference to the Connection associated with the beacon.
    pub connection: Option<ObjectId>,
    /// The ssid of the beacon, typically used for display purposes in BLE scanning.
    /// Format:
    /// ```
    /// BEACON-<area_id>-<beacon_id>
    /// ```
    /// where `<area_id>` is the ID of the area and `<beacon_id>` is the unique identifier of the beacon.
    /// They are incremental value from 0 and, the area id uses 2-byte hex encoding,
    /// whereas the beacon id uses 4-byte hex encoding.
    pub name: String,
    /// The displaying name of the beacon, which can be used for user-friendly identification.
    /// This can be the name of the area, merchant, or a custom name.
    pub description: Option<String>,
    /// The type of the beacon, which can indicate its purpose or functionality.
    pub r#type: BeaconType,
    /// The location of the beacon, represented as a pair of coordinates (longitude, latitude).
    pub location: (f64, f64),
    pub device: BeaconDevice,
    pub mac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BeaconDevice {
    Esp32,
    Esp32C3,
    Esp32C5,
    Esp32C6,
    Esp32S3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
/// Represents the type of beacon, which can indicate its purpose or functionality.
pub enum BeaconType {
    /// A beacon that is used for navigation or location-based services.
    Navigation,
    /// A beacon that is used for proximity marketing or advertising.
    Marketing,
    /// A beacon that is used for asset tracking or inventory management.
    Tracking,
    /// A beacon that is used for environmental monitoring, such as temperature or humidity.
    Environmental,
    /// A beacon that is used for security purposes, such as access control or intrusion detection.
    Security,
    /// A beacon that is used for other purposes not covered by the above categories.
    Other,
}

impl Service for Beacon {
    fn get_id(&self) -> String {
        self.id.to_hex()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    fn get_collection_name() -> &'static str {
        "beacons"
    }

    fn require_unique_name() -> bool {
        true
    }
}

impl OneInArea for Beacon {}
