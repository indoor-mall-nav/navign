use std::fmt::{Display, Formatter};
use crate::schema::service::Service;
use async_trait::async_trait;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

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

impl Area {
    pub fn get_object_id(&self) -> ObjectId {
        self.id
    }

    pub fn get_floor(&self) -> Option<&Floor> {
        self.floor.as_ref()
    }

    pub fn get_polygon(&self) -> &Vec<(f64, f64)> {
        &self.polygon
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Floor {
    pub r#type: FloorType,
    pub name: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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
