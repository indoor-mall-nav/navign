pub mod adapters;
pub mod auth_handlers;
pub mod handlers;
pub mod models;
/// PostgreSQL migration layer
///
/// This module provides an intermediate layer for PostgreSQL migration
/// without touching the existing MongoDB logic.
pub mod pool;
pub mod repository;
pub mod route_handlers;

pub use pool::{PgPool, create_pool};
