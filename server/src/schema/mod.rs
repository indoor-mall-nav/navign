#![allow(unused)]
pub mod authentication;
mod metadata;

// Re-export commonly used types from shared
pub use navign_shared::schema::{
    Area, AuthenticationType, Beacon, BeaconDevice, BeaconType, Connection, ConnectionType, Entity,
    EntityType, Floor, FloorType, Merchant, UnlockInstance, UnlockStage,
};
