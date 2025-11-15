use crate::error::{Result, ServerError};
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgPool as SqlxPgPool, PgPoolOptions};
use std::time::Duration;

/// PostgreSQL connection pool wrapper
#[derive(Clone)]
pub struct PgPool {
    pool: SqlxPgPool,
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
        // Migrations are embedded in the binary
        let migrator = Migrator::new(std::path::Path::new("./migrations"))
            .await
            .map_err(|e| {
                ServerError::DatabaseConnection(format!("Failed to load migrations: {}", e))
            })?;

        migrator.run(&self.pool).await.map_err(|e| {
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

    log::info!("Successfully connected to PostgreSQL");

    Ok(PgPool::new(pool))
}
