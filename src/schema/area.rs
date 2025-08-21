use serde::{Serialize, Deserialize};
use bson::oid::ObjectId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Area {
    _id: ObjectId,
    entity: ObjectId, // Reference to the Entity
    name: String,
    description: Option<String>,
    floor: Option<Floor>, // Floor number or name
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