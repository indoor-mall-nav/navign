use crate::schema::service::Service;
use async_trait::async_trait;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Area {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub entity: ObjectId, // Reference to the Entity
    pub name: String,
    pub description: Option<String>,
    /// Unique identifier for the area for displaying in the beacon name.
    pub beacon_code: String,
    pub floor: Option<Floor>,     // Floor number or name
    pub polygon: Vec<(f64, f64)>, // List of (x, y) pairs of coordinates
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct Floor {
    pub r#type: FloorType,
    pub name: u32,
}

impl From<Floor> for i32 {
    fn from(val: Floor) -> i32 {
        match val.r#type {
            FloorType::Level => val.name as i32 + 1, // Level 0 is Ground, Level 1 is First
            FloorType::Floor => val.name as i32,
            FloorType::Basement => -(val.name as i32),
        }
    }
}

#[derive(Debug, Clone, Serialize, Copy, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum FloorType {
    /// European/UK style, e.g., "Ground," "First," "Second"
    Level,
    /// US style, e.g., "1st," "2nd," "3rd"
    #[default]
    Floor,
    /// Universal basement
    Basement,
}

impl Display for FloorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FloorType::Level => write!(f, "Level"),
            FloorType::Floor => write!(f, "Floor"),
            FloorType::Basement => write!(f, "Basement"),
        }
    }
}

impl Display for Floor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.r#type {
            FloorType::Level => write!(f, "L{}", self.name),
            FloorType::Floor => write!(f, "{}F", self.name),
            FloorType::Basement => write!(f, "B{}", self.name),
        }
    }
}

#[async_trait]
impl Service for Area {
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
        "areas"
    }

    fn require_unique_name() -> bool {
        false
    }
}
