// CRUD repository trait implementations for shared schemas
// These implementations bridge the shared repository traits with server-specific CRUD traits

use crate::pg::repository::{IntCrudRepository, IntCrudRepositoryInArea, UuidCrudRepository};
use navign_shared::schema::{Area, Beacon, Connection, Entity, Merchant};

// Entity uses UuidCrudRepository
impl UuidCrudRepository for Entity {}

// Area uses IntCrudRepository
impl IntCrudRepository for Area {}

// Beacon uses IntCrudRepository
impl IntCrudRepository for Beacon {}
impl IntCrudRepositoryInArea for Beacon {}

// Merchant uses both IntCrudRepository and IntCrudRepositoryInArea
impl IntCrudRepository for Merchant {}
impl IntCrudRepositoryInArea for Merchant {}

// Connection uses IntCrudRepository
impl IntCrudRepository for Connection {}
