#[cfg(all(feature = "serde", feature = "alloc"))]
pub mod read_query;

#[cfg(all(feature = "serde", feature = "alloc"))]
pub use read_query::ReadQuery;

// Core schema modules
#[cfg(feature = "alloc")]
pub mod area;
#[cfg(feature = "alloc")]
pub mod beacon;
#[cfg(feature = "alloc")]
pub mod connection;
#[cfg(feature = "alloc")]
pub mod entity;
#[cfg(feature = "alloc")]
pub mod merchant;

// Re-export core types
#[cfg(feature = "alloc")]
pub use area::{Area, Floor, FloorType};
#[cfg(feature = "alloc")]
pub use beacon::{Beacon, BeaconDevice, BeaconType};
#[cfg(feature = "alloc")]
pub use connection::{Connection, ConnectedArea, ConnectionType};
#[cfg(feature = "alloc")]
pub use entity::{Entity, EntityType};
#[cfg(feature = "alloc")]
pub use merchant::{
    ChineseFoodCuisine, FacilityType, FoodCuisine, FoodType, Merchant, MerchantStyle, MerchantType,
    SocialMedia, SocialMediaPlatform,
};

// Mobile-specific exports
#[cfg(feature = "sql")]
pub use area::mobile::AreaMobile;
#[cfg(feature = "sql")]
pub use beacon::mobile::BeaconMobile;
#[cfg(feature = "sql")]
pub use connection::mobile::ConnectionMobile;
#[cfg(feature = "sql")]
pub use entity::mobile::EntityMobile;
#[cfg(feature = "sql")]
pub use merchant::mobile::MerchantMobile;
