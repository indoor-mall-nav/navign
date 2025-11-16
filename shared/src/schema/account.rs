#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use alloc::string::String;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    all(feature = "serde", not(feature = "postgres")),
    derive(Serialize, Deserialize)
)]
#[cfg_attr(feature = "postgres", derive(sqlx::FromRow))]
pub struct Account {
    #[cfg(feature = "postgres")]
    pub id: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    pub id: String,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    pub activated: bool,
    pub privileged: bool,
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

/// Request schema for user registration
#[cfg(all(feature = "alloc", feature = "serde"))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Request schema for user login
#[cfg(all(feature = "alloc", feature = "serde"))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response schema for authentication
#[cfg(all(feature = "alloc", feature = "serde"))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

/// Token claims for JWT authentication
#[cfg(all(feature = "alloc", feature = "serde"))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TokenClaims {
    pub sub: String, // Subject (user ID)
    pub username: String,
    pub exp: i64, // Expiration time
    pub iat: i64, // Issued at
}
