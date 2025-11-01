#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "mongodb")]
use bson::oid::ObjectId;

use core::fmt::{Display, Formatter};

/// Area schema - represents a physical area in the mall/building
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "mongodb", derive(Default))]
pub struct Area {
    #[cfg_attr(feature = "serde", serde(rename = "_id"))]
    #[cfg(feature = "mongodb")]
    pub id: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub id: String,
    
    #[cfg(feature = "mongodb")]
    pub entity: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub entity: String,
    
    pub name: String,
    pub description: Option<String>,
    /// Unique identifier for the area for displaying in the beacon name.
    pub beacon_code: String,
    pub floor: Option<Floor>,     // Floor number or name
    pub polygon: Vec<(f64, f64)>, // List of (x, y) pairs of coordinates
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "mongodb", derive(Default))]
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
#[cfg_attr(feature = "mongodb", derive(Default))]
pub enum FloorType {
    /// European/UK style, e.g., "Ground," "First," "Second"
    Level,
    /// US style, e.g., "1st," "2nd," "3rd"
    #[cfg_attr(feature = "mongodb", default)]
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

// Mobile-specific version for SQLite storage
#[cfg(feature = "sql")]
pub mod mobile {
    use super::{Floor, FloorType};
    #[cfg(feature = "alloc")]
    use alloc::string::String;
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;
    use sqlx::FromRow;
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, FromRow)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct AreaMobile {
        pub id: String,
        pub entity: String,
        pub name: String,
        pub description: Option<String>,
        pub beacon_code: String,
        pub floor_type: Option<String>,
        pub floor_name: Option<i32>,
        /// Stored as WKT POLYGON string
        pub polygon: String,
    }

    impl AreaMobile {
        pub fn floor(&self) -> Option<Floor> {
            match (self.floor_type.as_ref(), self.floor_name) {
                (Some(ft), Some(fn_)) => {
                    let floor_type = match ft.as_str() {
                        "level" | "Level" => FloorType::Level,
                        "floor" | "Floor" => FloorType::Floor,
                        "basement" | "Basement" => FloorType::Basement,
                        _ => FloorType::Floor,
                    };
                    Some(Floor {
                        r#type: floor_type,
                        name: fn_ as u32,
                    })
                }
                _ => None,
            }
        }

        #[cfg(feature = "sql")]
        pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS areas (
                    id VARCHAR(24) PRIMARY KEY,
                    entity VARCHAR(24) NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    beacon_code TEXT NOT NULL,
                    floor_type TEXT,
                    floor_name INTEGER,
                    polygon TEXT NOT NULL
                )
                "#,
            )
            .execute(pool)
            .await?;
            Ok(())
        }

        #[cfg(feature = "sql")]
        pub async fn insert(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO areas (id, entity, name, description, beacon_code, floor_type, floor_name, polygon)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&self.id)
            .bind(&self.entity)
            .bind(&self.name)
            .bind(&self.description)
            .bind(&self.beacon_code)
            .bind(&self.floor_type)
            .bind(&self.floor_name)
            .bind(&self.polygon)
            .execute(pool)
            .await?;
            Ok(())
        }

        #[cfg(feature = "sql")]
        pub async fn get_by_id(pool: &sqlx::SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM areas WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn get_all(pool: &sqlx::SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM areas")
                .fetch_all(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn delete(pool: &sqlx::SqlitePool, id: &str) -> Result<(), sqlx::Error> {
            sqlx::query("DELETE FROM areas WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
            Ok(())
        }
    }
}
