#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// User public keys schema - stores user's public keys for device authentication
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "sql", derive(sqlx::FromRow))]
pub struct UserPublicKeys {
    pub id: i32,
    #[cfg(feature = "postgres")]
    pub user_id: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    pub user_id: String,
    /// The public key in PEM format
    pub public_key: String,
    /// Device identifier (unique per device)
    pub device_id: String,
    /// Optional device name for display purposes
    pub device_name: Option<String>,
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

impl UserPublicKeys {
    /// Get user public key by device_id
    #[cfg(all(feature = "postgres", feature = "sql"))]
    pub async fn get_by_device_id(pool: &sqlx::PgPool, device_id: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, UserPublicKeys>(
            "SELECT id, user_id, public_key, device_id, device_name, created_at, updated_at
             FROM user_public_keys
             WHERE device_id = $1"
        )
        .bind(device_id)
        .fetch_optional(pool)
        .await
    }
}
