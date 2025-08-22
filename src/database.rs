use anyhow::Result;
use log::info;
use mongodb::options::{ClientOptions, ServerAddress};
use mongodb::{Client, Database};
use std::str::FromStr;
use std::time::Duration;

pub(crate) async fn connect_with_db() -> Result<Database> {
    dotenv::dotenv()?;
    info!(
        "Connecting to database with url {}",
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string())
    );
    let mut options = ClientOptions::default();
    options.max_pool_size = Some(8);
    options.min_pool_size = Some(2);
    options.max_idle_time = Some(Duration::from_secs(30));
    options.max_connecting = Some(10);
    options.connect_timeout = Some(Duration::from_secs(10));
    options.server_selection_timeout = Some(Duration::from_secs(10));
    options.app_name = Some("indoor-mall-nav".to_string());
    let host = std::env::var("MONGODB_HOST").unwrap_or_else(|_| "localhost:27017".to_string());
    let host = ServerAddress::from_str(&host)?;
    options.hosts = vec![host];
    let client = Client::with_options(options)?;
    info!("Connected to MongoDB server successfully.");
    let db_name = std::env::var("MONGODB_DB_NAME").unwrap_or("indoor-mall-nav".to_string());
    info!("Using database: {}", db_name);
    let db = client.database(&db_name);
    Ok(db)
}
