use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Beacon {
    _id: ObjectId,
    /// Reference to the Entity
    entity: ObjectId,
    /// Unique identifier for the beacon, typically a UUID or similar
    beacon_id: String,
    /// Reference to the Area where the beacon is located
    area: ObjectId,
    /// Optional reference to the Merchant associated with the beacon.
    merchant: Option<ObjectId>,
    /// The ssid of the beacon, typically used for display purposes in BLE scanning.
    /// Format:
    /// ```
    /// BEACON-<area_id>-<beacon_id>
    /// ```
    /// where `<area_id>` is the ID of the area and `<beacon_id>` is the unique identifier of the beacon.
    /// They are incremental value from 0 and, the area id uses 2-byte hex encoding,
    /// whereas the beacon id uses 4-byte hex encoding.
    ssid: String,
    /// The displaying name of the beacon, which can be used for user-friendly identification.
    /// This can be the name of the area, merchant, or a custom name.
    name: Option<String>,
    /// The description of the beacon, which can provide additional context or information.
    description: Option<String>,
    /// The type of the beacon, which can indicate its purpose or functionality.
    r#type: BeaconType,
    /// The location of the beacon, represented as a pair of coordinates (longitude, latitude).
    location: (f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
