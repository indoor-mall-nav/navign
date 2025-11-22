// CRUD repository trait implementations for shared schemas
// These implementations bridge the shared repository traits with server-specific CRUD traits

use crate::pg::repository::{IntCrudRepository, IntCrudRepositoryInArea, UuidCrudRepository};
use navign_shared::schema::{Area, Beacon, Connection, Entity, Merchant};

// Entity uses UuidCrudRepository
impl UuidCrudRepository for Entity {
    const API_ENDPOINT: &'static str = "entities";
}

// Area uses IntCrudRepository
impl IntCrudRepository for Area {
    const API_ENDPOINT: &'static str = "areas";
    const WRAPPER_NAME: &'static str = "entities";
}

// Beacon uses IntCrudRepository
impl IntCrudRepository for Beacon {
    const API_ENDPOINT: &'static str = "beacons";
    const WRAPPER_NAME: &'static str = "entities";
}
impl IntCrudRepositoryInArea for Beacon {
    const API_ENDPOINT: &'static str = "beacons";
    const WRAPPER_NAME: &'static str = "entities";
}

// Merchant uses both IntCrudRepository and IntCrudRepositoryInArea
impl IntCrudRepository for Merchant {
    const API_ENDPOINT: &'static str = "merchants";
    const WRAPPER_NAME: &'static str = "entities";
}
impl IntCrudRepositoryInArea for Merchant {
    const API_ENDPOINT: &'static str = "merchants";
    const WRAPPER_NAME: &'static str = "entities";
}

// Connection uses IntCrudRepository
impl IntCrudRepository for Connection {
    const API_ENDPOINT: &'static str = "connections";
    const WRAPPER_NAME: &'static str = "entities";
}
