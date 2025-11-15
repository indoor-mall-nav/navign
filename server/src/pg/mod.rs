/// PostgreSQL migration layer
///
/// This module provides an intermediate layer for PostgreSQL migration
/// without touching the existing MongoDB logic.

pub mod pool;
pub mod repository;
pub mod models;

pub use pool::{PgPool, create_pool};
