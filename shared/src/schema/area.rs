#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(all(feature = "alloc", not(feature = "postgres")))]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "postgres")]
use crate::schema::postgis::PgPolygon;
use core::fmt::{Display, Formatter};
#[cfg(feature = "sql")]
use sqlx::FromRow;

/// Area schema - represents a physical area in the mall/building
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "sql", derive(FromRow))]
#[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), derive(ts_rs::TS))]
#[cfg_attr(
    all(feature = "ts-rs", not(feature = "postgres")),
    ts(export, export_to = "generated/")
)]
pub struct Area {
    #[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), ts(type = "string"))]
    pub id: i32,
    #[cfg(feature = "postgres")]
    pub entity: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), ts(type = "string"))]
    pub entity: String,
    pub name: String,
    pub description: Option<String>,
    /// Unique identifier for the area for displaying in the beacon name.
    pub beacon_code: String,
    pub floor: Option<Floor>,
    #[cfg(feature = "postgres")]
    pub polygon: PgPolygon,
    #[cfg(not(feature = "postgres"))]
    pub polygon: Vec<(f64, f64)>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
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

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub enum FloorType {
    /// European/UK style, e.g., "Ground," "First," "Second"
    Level,
    /// US style, e.g., "1st," "2nd," "3rd"
    Floor,
    /// Universal basement
    Basement,
}

impl Display for FloorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            FloorType::Level => write!(f, "Level"),
            FloorType::Floor => write!(f, "Floor"),
            FloorType::Basement => write!(f, "Basement"),
        }
    }
}

impl Display for Floor {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.r#type {
            FloorType::Level => write!(f, "L{}", self.name),
            FloorType::Floor => write!(f, "{}F", self.name),
            FloorType::Basement => write!(f, "B{}", self.name),
        }
    }
}
