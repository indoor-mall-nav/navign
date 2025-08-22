use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    _id: ObjectId,
    r#type: EntityType,
    name: String,
    description: Option<String>,
    longitude_range: (f64, f64),        // (min_longitude, max_longitude)
    latitude_range: (f64, f64),         // (min_latitude, max_latitude)
    altitude_range: Option<(f64, f64)>, // (min_altitude, max_altitude)
    nation: Option<String>,
    region: Option<String>,
    city: Option<String>,
    tags: Vec<String>,
    created_at: i64, // Timestamp in milliseconds
    updated_at: i64, // Timestamp in milliseconds
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Mall,
    Transportation,
    School,
    Hospital,
}
