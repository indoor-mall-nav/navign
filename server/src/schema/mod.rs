#![allow(unused)]
pub mod authentication;
mod beacon_secrets;
mod metadata;
pub mod polygon;
pub mod user;
mod user_public;

pub use beacon_secrets::BeaconSecrets;
pub use user::User;
pub use user_public::UserPublicKeys;

// Re-export commonly used types from shared
pub use navign_shared::schema::{
    Area, Beacon, BeaconDevice, BeaconType, Connection, ConnectionType, Entity, EntityType,
    Floor, FloorType, Merchant, UnlockInstance, UnlockStage, AuthenticationType,
};
