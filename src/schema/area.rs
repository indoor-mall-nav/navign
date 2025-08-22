use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Area {
    _id: ObjectId,
    entity: ObjectId, // Reference to the Entity
    name: String,
    description: Option<String>,
    /// Unique identifier for the area for displaying in the beacon name.
    beacon_code: String,
    floor: Option<Floor>,     // Floor number or name
    polygon: Vec<(f64, f64)>, // List of (x, y) pairs of coordinates
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Floor {
    r#type: FloorType,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FloorType {
    /// European/UK style, e.g., "Ground," "First," "Second"
    Level,
    /// US style, e.g., "1st," "2nd," "3rd"
    Floor,
    /// Universal basement
    Basement,
}
