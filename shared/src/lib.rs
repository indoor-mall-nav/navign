#![cfg_attr(not(any(feature = "std", feature = "sql")), no_std)]

#[cfg(all(feature = "heapless", feature = "alloc"))]
compile_error!("Features 'heapless' and 'alloc' cannot be enabled at the same time.");

#[cfg(all(not(feature = "heapless"), not(feature = "alloc")))]
compile_error!("Either feature 'heapless' or 'alloc' must be enabled.");

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod constants;
pub mod errors;

#[cfg(feature = "std")]
pub mod pathfinding;

mod ble;
mod crypto;
pub mod schema;
mod traits;

pub use ble::blufi::{
    BlufiError as BlufiProtocolError, BlufiMessage, BlufiPayload, ControlFrame, DataFrame,
    FrameControl, WifiAuthMode, WifiConnectionState, WifiOpmode,
};
pub use ble::challenge::ServerChallenge;
pub use ble::device_caps::DeviceCapabilities;
pub use ble::device_type::DeviceTypes;
pub use ble::message::BleMessage;
pub use ble::nonce::Nonce;
pub use ble::proof::Proof;
pub use traits::{depacketize::Depacketize, packetize::Packetize};

#[cfg(all(feature = "serde", feature = "alloc"))]
pub use schema::ReadQuery;

// Export core schemas
#[cfg(all(feature = "serde", feature = "alloc", feature = "mongodb"))]
pub use schema::Account;
#[cfg(feature = "alloc")]
pub use schema::{
    Area, Beacon, BeaconDevice, BeaconType, ChineseFoodCuisine, ConnectedArea, Connection,
    ConnectionType, Entity, EntityType, FacilityType, Firmware, FirmwareDevice, FirmwareQuery,
    FirmwareUploadRequest, FirmwareUploadResponse, Floor, FloorType, FoodCuisine, FoodType,
    Merchant, MerchantStyle, MerchantType, SocialMedia, SocialMediaPlatform,
};
#[cfg(all(feature = "serde", feature = "alloc"))]
pub use schema::{AuthResponse, LoginRequest, RegisterRequest, TokenClaims};

// Export BluFi provisioning schemas
#[cfg(feature = "alloc")]
pub use schema::{
    BeaconLocation, BeaconProvisioningStatus, BluFiConfig, BluFiError, BluFiErrorType,
    BluFiProvisioningResult, BluFiState, WiFiNetwork, WiFiSecurityMode,
};

// Export mobile-specific schemas
#[cfg(feature = "sql")]
pub use schema::{AreaMobile, BeaconMobile, ConnectionMobile, EntityMobile, MerchantMobile};
