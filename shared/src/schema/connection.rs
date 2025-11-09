#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "sql")]
use uuid::Uuid;

use core::fmt::Display;

/// ConnectedArea type - (area_id, x, y, enabled)
#[cfg(feature = "sql")]
pub type ConnectedArea = (i64, f64, f64, bool);

/// ConnectedArea type for non-SQL (String-based IDs)
#[cfg(not(feature = "sql"))]
pub type ConnectedArea = (String, f64, f64, bool);

/// Connection schema - represents connections between areas (gates, elevators, etc.)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Connection {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg(feature = "sql")]
    pub id: Option<Uuid>,
    #[cfg(not(feature = "sql"))]
    pub id: String,
    /// Reference to the Entity (UUID)
    #[cfg(feature = "sql")]
    pub entity: Uuid,
    #[cfg(not(feature = "sql"))]
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
    pub created_at: i64,         // Timestamp in milliseconds
    pub updated_at: i64,         // Timestamp in milliseconds
}

impl Connection {
    pub fn get_connected_areas(&self) -> &Vec<ConnectedArea> {
        &self.connected_areas
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum ConnectionType {
    Gate,
    #[default]
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
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};
    use sqlx::FromRow;
    use uuid::Uuid;

    #[derive(Debug, Clone, FromRow)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ConnectionMobile {
        pub id: Uuid,
        pub entity: Uuid,
        pub name: String,
        pub description: Option<String>,
        pub r#type: String,
        /// JSON array: [{"area": "id", "x": 0.0, "y": 0.0, "enabled": true}, ...]
        pub connected_areas: String,
        /// JSON array: [[start, end], ...]
        pub available_period: String,
        pub tags: String, // JSON array
        /// Stored as WKT POINT string
        pub gnd: Option<String>,
        pub created_at: i64,
        pub updated_at: i64,
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
                    id TEXT PRIMARY KEY,
                    entity TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    type TEXT NOT NULL,
                    connected_areas TEXT NOT NULL,
                    available_period TEXT NOT NULL,
                    tags TEXT NOT NULL,
                    gnd TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
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
                (id, entity, name, description, type, connected_areas, available_period, tags, gnd, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            .bind(&self.gnd)
            .bind(self.created_at)
            .bind(self.updated_at)
            .execute(pool)
            .await?;
            Ok(())
        }

        #[cfg(feature = "sql")]
        pub async fn get_by_id(
            pool: &sqlx::SqlitePool,
            id: &Uuid,
        ) -> Result<Option<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM connections WHERE id = ?")
                .bind(id.to_string())
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
        pub async fn delete(pool: &sqlx::SqlitePool, id: &Uuid) -> Result<(), sqlx::Error> {
            sqlx::query("DELETE FROM connections WHERE id = ?")
                .bind(id.to_string())
                .execute(pool)
                .await?;
            Ok(())
        }
    }
}
