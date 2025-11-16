#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use core::fmt::Display;

#[cfg(feature = "postgres")]
use crate::schema::postgres::PgPoint;

/// Entity schema - represents a physical building or complex (mall, hospital, etc.)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(all(feature = "ts-rs", not(feature = "postgres")), derive(ts_rs::TS))]
#[cfg_attr(
    all(feature = "ts-rs", not(feature = "postgres")),
    ts(export, export_to = "generated/")
)]
pub struct Entity {
    #[cfg(feature = "postgres")]
    pub id: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub id: String,
    pub r#type: EntityType,
    pub name: String,
    pub description: Option<String>,
    #[cfg(feature = "postgres")]
    pub point_min: PgPoint,
    #[cfg(not(feature = "postgres"))]
    pub point_min: (f64, f64),
    #[cfg(feature = "postgres")]
    pub point_max: PgPoint,
    #[cfg(not(feature = "postgres"))]
    pub point_max: (f64, f64),
    pub altitude_range: Option<(f64, f64)>, // (min_altitude, max_altitude)
    pub nation: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub tags: Vec<String>,
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
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub enum EntityType {
    Mall,
    Transportation,
    School,
    Hospital,
}

impl Display for EntityType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EntityType::Mall => write!(f, "Mall"),
            EntityType::Transportation => write!(f, "Transportation"),
            EntityType::School => write!(f, "School"),
            EntityType::Hospital => write!(f, "Hospital"),
        }
    }
}
