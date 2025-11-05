#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "mongodb")]
use bson::oid::ObjectId;

#[cfg(all(feature = "alloc", feature = "serde"))]
use alloc::string::String;

/// Account schema representing a user account in the system
/// Note: This schema is primarily for use with the mongodb feature.
/// When mongodb feature is disabled, the id field is not available.
#[cfg(all(feature = "alloc", feature = "serde", feature = "mongodb"))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Account {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    pub activated: bool,
    pub privileged: bool,
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
