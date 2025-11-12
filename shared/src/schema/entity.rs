#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "mongodb")]
use bson::oid::ObjectId;

#[cfg(all(feature = "mongodb", feature = "serde"))]
use bson::serde_helpers::serialize_object_id_as_hex_string;

use core::fmt::Display;

/// Entity schema - represents a physical building or complex (mall, hospital, etc.)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "mongodb", derive(Default))]
pub struct Entity {
    #[cfg(feature = "mongodb")]
    #[cfg_attr(
        all(feature = "mongodb", feature = "serde"),
        serde(rename = "_id", serialize_with = "serialize_object_id_as_hex_string",)
    )]
    pub id: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub id: String,
    pub r#type: EntityType,
    pub name: String,
    pub description: Option<String>,
    pub longitude_range: (f64, f64), // (min_longitude, max_longitude)
    pub latitude_range: (f64, f64),  // (min_latitude, max_latitude)
    pub altitude_range: Option<(f64, f64)>, // (min_altitude, max_altitude)
    pub nation: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub tags: Vec<String>,
    pub created_at: i64, // Timestamp in milliseconds
    pub updated_at: i64, // Timestamp in milliseconds
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "mongodb", derive(Default))]
pub enum EntityType {
    #[cfg_attr(feature = "mongodb", default)]
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

// Mobile-specific version for SQLite storage
#[cfg(feature = "sql")]
pub mod mobile {
    use super::EntityType;
    #[cfg(feature = "alloc")]
    use alloc::string::String;
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};
    use sqlx::FromRow;

    #[derive(Debug, Clone, FromRow)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct EntityMobile {
        pub id: String,
        pub r#type: String,
        pub name: String,
        pub description: Option<String>,
        pub longitude_min: f64,
        pub longitude_max: f64,
        pub latitude_min: f64,
        pub latitude_max: f64,
        pub altitude_min: Option<f64>,
        pub altitude_max: Option<f64>,
        pub nation: Option<String>,
        pub region: Option<String>,
        pub city: Option<String>,
        pub tags: String, // JSON array stored as string
        pub created_at: i64,
        pub updated_at: i64,
    }

    impl EntityMobile {
        pub fn entity_type(&self) -> EntityType {
            match self.r#type.as_str() {
                "mall" | "Mall" => EntityType::Mall,
                "transportation" | "Transportation" => EntityType::Transportation,
                "school" | "School" => EntityType::School,
                "hospital" | "Hospital" => EntityType::Hospital,
                _ => EntityType::Mall,
            }
        }

        #[cfg(feature = "sql")]
        pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS entities (
                    id VARCHAR(24) PRIMARY KEY,
                    type TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    longitude_min REAL NOT NULL,
                    longitude_max REAL NOT NULL,
                    latitude_min REAL NOT NULL,
                    latitude_max REAL NOT NULL,
                    altitude_min REAL,
                    altitude_max REAL,
                    nation TEXT,
                    region TEXT,
                    city TEXT,
                    tags TEXT NOT NULL,
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
                INSERT OR REPLACE INTO entities 
                (id, type, name, description, longitude_min, longitude_max, latitude_min, latitude_max, 
                 altitude_min, altitude_max, nation, region, city, tags, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&self.id)
            .bind(&self.r#type)
            .bind(&self.name)
            .bind(&self.description)
            .bind(self.longitude_min)
            .bind(self.longitude_max)
            .bind(self.latitude_min)
            .bind(self.latitude_max)
            .bind(self.altitude_min)
            .bind(self.altitude_max)
            .bind(&self.nation)
            .bind(&self.region)
            .bind(&self.city)
            .bind(&self.tags)
            .bind(self.created_at)
            .bind(self.updated_at)
            .execute(pool)
            .await?;
            Ok(())
        }

        #[cfg(feature = "sql")]
        pub async fn get_by_id(
            pool: &sqlx::SqlitePool,
            id: &str,
        ) -> Result<Option<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM entities WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn get_all(pool: &sqlx::SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM entities")
                .fetch_all(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn delete(pool: &sqlx::SqlitePool, id: &str) -> Result<(), sqlx::Error> {
            sqlx::query("DELETE FROM entities WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
            Ok(())
        }
    }
}
