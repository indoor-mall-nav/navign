#![allow(dead_code)] // Not yet integrated with handlers

//! PostgreSQL-specific models with PostGIS support
//!
//! These models use:
//! - UUID for entities and users
//! - i32 (SERIAL) for all other tables
//! - PostGIS GEOMETRY(POINT, 4326) for all coordinates
//!
//! PostGIS points use WGS84 (SRID 4326) which is the standard for GPS coordinates.

#[cfg(feature = "postgres")]
pub use area::PgArea;
#[cfg(feature = "postgres")]
pub use beacon::PgBeacon;
#[cfg(feature = "postgres")]
pub use connection::PgConnection;
#[cfg(feature = "postgres")]
pub use entity::PgEntity;
#[cfg(feature = "postgres")]
pub use merchant::PgMerchant;
#[cfg(feature = "postgres")]
pub use user::PgUser;

/// Re-export PgPoint for PostGIS GEOMETRY(POINT, 4326)
#[cfg(feature = "postgres")]
pub use super::postgis::PgPoint;

#[cfg(feature = "postgres")]
mod entity {
    use serde::{Deserialize, Serialize};
    use sqlx::{FromRow, types::Json};

    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct PgEntity {
        pub id: sqlx::types::Uuid,
        pub r#type: String,
        pub name: String,
        pub description: Option<String>,
        pub nation: Option<String>,
        pub region: Option<String>,
        pub city: Option<String>,
        pub address: Option<String>,
        pub longitude_min: f64,
        pub longitude_max: f64,
        pub latitude_min: f64,
        pub latitude_max: f64,
        pub floors: Json<Vec<serde_json::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }
}

#[cfg(feature = "postgres")]
mod user {
    use serde::{Deserialize, Serialize};
    use sqlx::FromRow;

    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct PgUser {
        pub id: sqlx::types::Uuid,
        pub username: String,
        pub email: String,
        pub phone: Option<String>,
        pub google: Option<String>,
        pub wechat: Option<String>,
        #[serde(skip_serializing)]
        pub hashed_password: String,
        pub activated: bool,
        pub privileged: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }
}

#[cfg(feature = "postgres")]
mod area {
    use super::PgPoint;
    use serde::{Deserialize, Serialize};
    use sqlx::{FromRow, types::Json};

    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct PgArea {
        pub id: i32,
        pub entity_id: sqlx::types::Uuid,
        pub name: String,
        pub description: Option<String>,
        pub floor: String,
        pub beacon_code: String,
        pub polygon: Json<serde_json::Value>, // GeoJSON or WKT
        pub centroid: Option<PgPoint>,        // PostGIS GEOMETRY(POINT, 4326)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }
}

#[cfg(feature = "postgres")]
mod beacon {
    use super::PgPoint;
    use serde::{Deserialize, Serialize};
    use sqlx::{FromRow, types::Json};

    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct PgBeacon {
        pub id: i32,
        pub entity_id: sqlx::types::Uuid,
        pub area_id: i32,
        pub merchant_id: Option<i32>,
        pub connection_id: Option<i32>,
        pub name: String,
        pub description: Option<String>,
        pub r#type: String,
        pub device_id: String,
        pub floor: String,
        pub location: PgPoint, // PostGIS GEOMETRY(POINT, 4326)
        pub public_key: Option<String>,
        pub capabilities: Json<Vec<String>>,
        pub unlock_method: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }
}

#[cfg(feature = "postgres")]
mod merchant {
    use super::PgPoint;
    use serde::{Deserialize, Serialize};
    use sqlx::{FromRow, types::Json};

    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct PgMerchant {
        pub id: i32,
        pub entity_id: sqlx::types::Uuid,
        pub area_id: i32,
        pub name: String,
        pub description: Option<String>,
        pub chain: Option<String>,
        pub r#type: String,
        pub logo: Option<String>,
        pub images: Json<Vec<String>>,
        pub social_media: Json<Vec<serde_json::Value>>,
        pub floor: String,
        pub location: PgPoint, // PostGIS GEOMETRY(POINT, 4326)
        pub merchant_style: Option<String>,
        pub food_type: Option<String>,
        pub food_cuisine: Option<String>,
        pub chinese_food_cuisine: Option<String>,
        pub facility_type: Option<String>,
        pub rating: Option<f64>,
        pub reviews: Option<i32>,
        pub opening_hours: Option<Json<serde_json::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }
}

#[cfg(feature = "postgres")]
mod connection {
    use serde::{Deserialize, Serialize};
    use sqlx::{FromRow, types::Json};

    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct PgConnection {
        pub id: i32,
        pub entity_id: sqlx::types::Uuid,
        pub name: String,
        pub description: Option<String>,
        pub r#type: String,
        pub connected_areas: Json<Vec<serde_json::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }
}

#[cfg(feature = "postgres")]
pub mod extras {
    use serde::{Deserialize, Serialize};
    use sqlx::FromRow;

    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct PgBeaconSecret {
        pub id: i32,
        pub beacon_id: i32,
        #[serde(skip_serializing)]
        pub private_key: Vec<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct PgUserPublicKey {
        pub id: i32,
        pub user_id: sqlx::types::Uuid,
        pub public_key: String,
        pub device_id: String,
        pub device_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct PgFirmware {
        pub id: i32,
        pub version: String,
        pub chip: String,
        pub file_name: String,
        pub file_size: i64,
        pub checksum: String,
        pub release_notes: Option<String>,
        pub is_stable: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }
}
