// Use library functions for creating the app
use navign_server::{create_app, load_or_generate_key, connect_with_db, AppState};

use log::{LevelFilter, info};
use simple_logger::SimpleLogger;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_http::cors::CorsLayer;
use axum::http::Method;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logger
    log::set_boxed_logger(Box::new(SimpleLogger::new()))
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| anyhow::anyhow!("Failed to initialize logger: {}", e))?;

    info!("Starting Navign Server v{}", env!("CARGO_PKG_VERSION"));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);
    info!("CORS layer configured (permissive mode for development)");

    // Load or generate persistent private key
    info!("Loading server private key...");
    let private_key = load_or_generate_key()
        .map_err(|e| anyhow::anyhow!("Failed to load or generate private key: {}", e))?;
    let public_key = private_key.verifying_key();
    info!(
        "Server public key loaded: {:?}",
        public_key.to_encoded_point(false).as_bytes()
    );

    // Configure rate limiting with environment variable support
    let requests_per_second = std::env::var("RATE_LIMIT_PER_SECOND")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let burst_size = std::env::var("RATE_LIMIT_BURST_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(requests_per_second)
            .burst_size(burst_size)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .ok_or_else(|| anyhow::anyhow!("Failed to build rate limiter configuration"))?,
    );

    info!(
        "Rate limiting configured: {} requests/second with burst size {}",
        requests_per_second, burst_size
    );

    // Start background task for rate limiter cleanup
    let governor_limiter = governor_conf.limiter().clone();
    let interval = Duration::from_secs(60);
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(interval);
            info!("Rate limiter storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    // Connect to database
    info!("Connecting to database...");
    let db = connect_with_db().await.map_err(|e| {
        anyhow::anyhow!(
            "Database connection failed: {}. Please check your MongoDB configuration.",
            e
        )
    })?;

    // Get server bind address from environment
    let bind_addr =
        std::env::var("SERVER_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    info!("Server will bind to: {}", bind_addr);

    let state = AppState { db, private_key };

    // Create the app using the library function
    let app = create_app(state)
        .layer(GovernorLayer::new(governor_conf))
        .layer(cors);

    info!("Starting HTTP server...");
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to bind to address '{}': {}. Please check if the port is already in use.",
                bind_addr,
                e
            )
        })?;

    info!("Server listening on {}", bind_addr);
    info!("Health check endpoint: http://{}/health", bind_addr);
    info!("API documentation: http://{}/", bind_addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}
