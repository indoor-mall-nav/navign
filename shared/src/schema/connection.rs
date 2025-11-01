#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "mongodb")]
use bson::oid::ObjectId;

use core::fmt::Display;

/// ConnectedArea type for MongoDB
#[cfg(feature = "mongodb")]
pub type ConnectedArea = (ObjectId, f64, f64, bool);

/// ConnectedArea type for non-MongoDB (String-based IDs)
#[cfg(not(feature = "mongodb"))]
pub type ConnectedArea = (String, f64, f64, bool);

/// Connection schema - represents connections between areas (gates, elevators, etc.)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "mongodb", derive(Default))]
pub struct Connection {
    #[cfg_attr(feature = "serde", serde(rename = "_id"))]
    #[cfg(feature = "mongodb")]
    pub id: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub id: String,
    
    /// Reference to the Entity
    #[cfg(feature = "mongodb")]
    pub entity: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub entity: String,
    
    pub name: String,
    pub description: Option<String>,
    pub r#type: ConnectionType,
    /// List of Area IDs that this connection links
    pub connected_areas: Vec<ConnectedArea>,
    /// List of `(start_time, end_time)` in milliseconds on a 24-hour clock
    pub available_period: Vec<(i32, i32)>,
    pub tags: Vec<String>,
    pub gnd: Option<(f64, f64)>, // Ground (x, y) coordinates if it connects to outside
}

impl Connection {
    pub fn get_connected_areas(&self) -> &Vec<ConnectedArea> {
        &self.connected_areas
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "mongodb", derive(Default))]
pub enum ConnectionType {
    Gate,
    #[cfg_attr(feature = "mongodb", default)]
    Escalator,
    Elevator,
    Stairs,
    Rail,
    Shuttle,
}

impl Display for ConnectionType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConnectionType::Gate => write!(f, "gate"),
            ConnectionType::Escalator => write!(f, "escalator"),
            ConnectionType::Elevator => write!(f, "elevator"),
            ConnectionType::Stairs => write!(f, "stairs"),
            ConnectionType::Rail => write!(f, "rail"),
            ConnectionType::Shuttle => write!(f, "shuttle"),
        }
    }
}

// Mobile-specific version for SQLite storage
#[cfg(feature = "sql")]
pub mod mobile {
    use super::ConnectionType;
    #[cfg(feature = "alloc")]
    use alloc::string::String;
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;
    use sqlx::FromRow;
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, FromRow)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ConnectionMobile {
        pub id: String,
        pub entity: String,
        pub name: String,
        pub description: Option<String>,
        pub r#type: String,
        /// JSON array: [{"area": "id", "x": 0.0, "y": 0.0, "enabled": true}, ...]
        pub connected_areas: String,
        /// JSON array: [[start, end], ...]
        pub available_period: String,
        pub tags: String, // JSON array
        pub gnd_x: Option<f64>,
        pub gnd_y: Option<f64>,
    }

    impl ConnectionMobile {
        pub fn connection_type(&self) -> ConnectionType {
            match self.r#type.as_str() {
                "gate" => ConnectionType::Gate,
                "escalator" => ConnectionType::Escalator,
                "elevator" => ConnectionType::Elevator,
                "stairs" => ConnectionType::Stairs,
                "rail" => ConnectionType::Rail,
                "shuttle" => ConnectionType::Shuttle,
                _ => ConnectionType::Escalator,
            }
        }

        #[cfg(feature = "sql")]
        pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS connections (
                    id VARCHAR(24) PRIMARY KEY,
                    entity VARCHAR(24) NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    type TEXT NOT NULL,
                    connected_areas TEXT NOT NULL,
                    available_period TEXT NOT NULL,
                    tags TEXT NOT NULL,
                    gnd_x REAL,
                    gnd_y REAL
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
                INSERT OR REPLACE INTO connections 
                (id, entity, name, description, type, connected_areas, available_period, tags, gnd_x, gnd_y)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&self.id)
            .bind(&self.entity)
            .bind(&self.name)
            .bind(&self.description)
            .bind(&self.r#type)
            .bind(&self.connected_areas)
            .bind(&self.available_period)
            .bind(&self.tags)
            .bind(self.gnd_x)
            .bind(self.gnd_y)
            .execute(pool)
            .await?;
            Ok(())
        }

        #[cfg(feature = "sql")]
        pub async fn get_by_id(pool: &sqlx::SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM connections WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn get_all(pool: &sqlx::SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM connections")
                .fetch_all(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn delete(pool: &sqlx::SqlitePool, id: &str) -> Result<(), sqlx::Error> {
            sqlx::query("DELETE FROM connections WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
            Ok(())
        }
    }
}
