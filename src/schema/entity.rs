use crate::schema::service::Service;
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

impl Service for Entity {
    fn get_id(&self) -> String {
        self._id.to_hex()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_id(&mut self, id: String) {
        self._id = ObjectId::parse_str(&id).expect("Invalid ObjectId format");
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
        "entities"
    }

    fn require_unique_name() -> bool {
        false
    }
}
