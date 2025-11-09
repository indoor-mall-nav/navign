use crate::error::{Result, ServerError};
use log::info;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub(crate) async fn connect_with_db() -> Result<PgPool> {
    // Load .env file if it exists, but don't fail if it's missing
    match dotenv::dotenv() {
        Ok(path) => info!("Loaded environment variables from: {:?}", path),
        Err(dotenv::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("No .env file found, using system environment variables");
        }
        Err(e) => {
            log::warn!(
                "Error loading .env file: {}. Continuing with system environment variables.",
                e
            );
        }
    }

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/navign".to_string());

    info!("Connecting to PostgreSQL at: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(8)
        .min_connections(2)
        .max_lifetime(Some(Duration::from_secs(30 * 60))) // 30 minutes
        .idle_timeout(Some(Duration::from_secs(10 * 60))) // 10 minutes
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .map_err(|e| {
            ServerError::DatabaseConnection(format!(
                "Failed to connect to PostgreSQL at '{}': {}. Please ensure PostgreSQL is running and accessible.",
                database_url, e
            ))
        })?;

    // Test the connection
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            ServerError::DatabaseConnection(format!(
                "Failed to ping PostgreSQL at '{}': {}. Please ensure PostgreSQL is running and accessible.",
                database_url, e
            ))
        })?;

    info!("Successfully connected to PostgreSQL server");

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| {
            ServerError::DatabaseConnection(format!(
                "Failed to run database migrations: {}",
                e
            ))
        })?;

    info!("Database migrations completed successfully");

    Ok(pool)
}
