use crate::error::{Result, ServerError};
use sqlx::postgres::{PgPool as SqlxPgPool, PgPoolOptions};
use std::time::Duration;
use tracing::info;

/// PostgreSQL connection pool wrapper
#[derive(Clone)]
pub struct PgPool {
    pub pool: SqlxPgPool,
}

impl PgPool {
    /// Create a new pool from an existing sqlx pool
    pub fn new(pool: SqlxPgPool) -> Self {
        Self { pool }
    }

    /// Get the underlying sqlx pool
    pub fn inner(&self) -> &SqlxPgPool {
        &self.pool
    }

    /// Run migrations
    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| {
                ServerError::DatabaseConnection(format!("Failed to run migrations: {}", e))
            })?;

        Ok(())
    }
}

/// Create a PostgreSQL connection pool
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(30))
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .connect(database_url)
        .await
        .map_err(|e| {
            ServerError::DatabaseConnection(format!(
                "Failed to connect to PostgreSQL at '{}': {}",
                database_url, e
            ))
        })?;

    info!("Successfully connected to PostgreSQL");

    Ok(PgPool::new(pool))
}
