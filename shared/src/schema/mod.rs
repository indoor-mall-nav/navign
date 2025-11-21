#[cfg(all(feature = "serde", feature = "alloc"))]
pub mod read_query;

#[cfg(all(feature = "serde", feature = "alloc"))]
pub use read_query::ReadQuery;

// Core schema modules
#[cfg(feature = "alloc")]
pub mod account;
#[cfg(feature = "alloc")]
pub mod area;
#[cfg(feature = "alloc")]
pub mod beacon;
#[cfg(feature = "alloc")]
pub mod beacon_secrets;
#[cfg(feature = "alloc")]
pub mod blufi;
#[cfg(feature = "alloc")]
pub mod connection;
#[cfg(feature = "alloc")]
pub mod entity;
#[cfg(feature = "alloc")]
pub mod firmware;
#[cfg(feature = "alloc")]
pub mod merchant;
#[cfg(feature = "alloc")]
pub mod user_public_keys;

// Re-export core types
#[cfg(all(feature = "alloc", feature = "serde"))]
pub use account::Account;
#[cfg(all(feature = "alloc", feature = "serde"))]
pub use account::{AuthResponse, LoginRequest, RegisterRequest, TokenClaims};
#[cfg(feature = "alloc")]
pub use area::{Area, Floor, FloorType};
#[cfg(feature = "alloc")]
pub use beacon::{Beacon, BeaconDevice, BeaconType};
#[cfg(feature = "alloc")]
pub use beacon_secrets::BeaconSecrets;
#[cfg(feature = "alloc")]
pub use blufi::{
    BeaconLocation, BeaconProvisioningStatus, BluFiConfig, BluFiError, BluFiErrorType,
    BluFiProvisioningResult, BluFiState, WiFiNetwork, WiFiSecurityMode,
};
#[cfg(feature = "alloc")]
pub use connection::{ConnectedArea, Connection, ConnectionType};
#[cfg(feature = "alloc")]
pub use entity::{Entity, EntityType};
#[cfg(feature = "alloc")]
pub use firmware::{
    Firmware, FirmwareDevice, FirmwareQuery, FirmwareUploadRequest, FirmwareUploadResponse,
};
#[cfg(feature = "alloc")]
pub use merchant::{
    ChineseFoodCuisine, FacilityType, FoodCuisine, FoodType, Merchant, MerchantStyle, MerchantType,
    SocialMedia, SocialMediaPlatform,
};
#[cfg(feature = "alloc")]
pub use user_public_keys::UserPublicKeys;

// PostgreSQL-specific exports (postgis also has WKB utilities used by SQLite)
#[cfg(feature = "geo")]
pub mod postgis;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sql")]
pub mod repository;

// SQLite-specific FromRow implementations
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
mod sqlite_from_row;

#[cfg(feature = "sql")]
pub use repository::{IntRepository, IntRepositoryInArea, UuidRepository};

#[macro_export]
macro_rules! make_uuid {
    ($var:ident) => {
        Uuid::parse_str($var).unwrap_or(Uuid::nil())
    };
}
