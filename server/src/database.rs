use crate::error::{Result, ServerError};
use log::info;
use mongodb::options::{ClientOptions, ServerAddress};
use mongodb::{Client, Database};
use std::str::FromStr;
use std::time::Duration;

pub(crate) async fn connect_with_db() -> Result<Database> {
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

    let mongodb_host =
        std::env::var("MONGODB_HOST").unwrap_or_else(|_| "localhost:27017".to_string());
    let db_name =
        std::env::var("MONGODB_DB_NAME").unwrap_or_else(|_| "indoor-mall-nav".to_string());

    info!("Connecting to MongoDB at: {}", mongodb_host);
    info!("Using database: {}", db_name);

    let mut options = ClientOptions::default();
    options.max_pool_size = Some(8);
    options.min_pool_size = Some(2);
    options.max_idle_time = Some(Duration::from_secs(30));
    options.max_connecting = Some(10);
    options.connect_timeout = Some(Duration::from_secs(10));
    options.server_selection_timeout = Some(Duration::from_secs(10));
    options.app_name = Some("indoor-mall-nav".to_string());

    let host = ServerAddress::from_str(&mongodb_host).map_err(|e| {
        ServerError::DatabaseConnection(format!("Invalid MongoDB host '{}': {}", mongodb_host, e))
    })?;
    options.hosts = vec![host];

    let client = Client::with_options(options).map_err(|e| {
        ServerError::DatabaseConnection(format!("Failed to create MongoDB client: {}", e))
    })?;

    // Test the connection
    let db = client.database(&db_name);
    db.run_command(bson::doc! { "ping": 1 })
        .await
        .map_err(|e| {
            ServerError::DatabaseConnection(format!(
                "Failed to connect to MongoDB at '{}': {}. Please ensure MongoDB is running and accessible.",
                mongodb_host, e
            ))
        })?;

    info!("Successfully connected to MongoDB server");
    Ok(db)
}
