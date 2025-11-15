mod certification;
mod database;
mod error;
mod kernel;
mod key_management;
mod metrics;
mod pg;
mod schema;
mod shared;

use crate::error::{Result as ServerResult, ServerError};
use crate::kernel::auth::{login_handler, register_handler};
use crate::kernel::route::find_route;
use crate::kernel::unlocker::{
    create_unlock_instance, record_unlock_result, update_unlock_instance,
};
use crate::key_management::load_or_generate_key;
use crate::schema::firmware::{
    delete_firmware_handler, download_firmware_handler, get_firmware_by_id_handler,
    get_firmwares_handler, get_latest_firmware_handler, upload_firmware_handler,
};
use crate::schema::service::OneInArea;
use crate::schema::{Area, Beacon, Connection, Entity, EntityServiceAddons, Merchant, Service};
use axum::extract::State;
use axum::middleware;
use axum::response::IntoResponse;
use axum::{
    Router,
    http::{Method, StatusCode},
    routing::{delete, get, post, put},
};
use bson::doc;
use mongodb::Database;
use p256::ecdsa::SigningKey;
use p256::pkcs8::EncodePublicKey;
use rsa::pkcs1::LineEnding;
use std::sync::Arc;
#[cfg(not(debug_assertions))]
use std::time::Duration;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
#[cfg(debug_assertions)]
use tower_governor::key_extractor::GlobalKeyExtractor;
#[cfg(not(debug_assertions))]
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_http::cors::CorsLayer;
use tracing::info;

async fn root() -> impl IntoResponse {
    (StatusCode::OK, "Hello, World!")
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Here you can add logic to check the health of your application, e.g., database connection
    match state.db.run_command(doc! { "ping": 1 }).await {
        Ok(_) => (StatusCode::OK, "Healthy"),
        Err(e) => {
            info!("Health check failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Unhealthy")
        }
    }
}

#[derive(Clone)]
pub(crate) struct AppState {
    db: Database,
    #[allow(dead_code)] // PostgreSQL layer not yet integrated
    pg_pool: Option<Arc<pg::PgPool>>,
    private_key: SigningKey,
    prometheus_handle: metrics_exporter_prometheus::PrometheusHandle,
}

async fn cert(State(state): State<AppState>) -> Result<String, ServerError> {
    state
        .private_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| ServerError::CryptographyError(format!("Failed to encode public key: {}", e)))
}

#[tokio::main]
async fn main() -> ServerResult<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting Navign Server v{}", env!("CARGO_PKG_VERSION"));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);
    info!("CORS layer configured (permissive mode for development)");

    // Initialize metrics
    info!("Initializing metrics exporter...");
    let prometheus_handle = metrics::init_metrics().map_err(|e| {
        ServerError::ConfigurationError(format!("Failed to initialize metrics: {}", e))
    })?;
    info!("Metrics exporter initialized");

    // Load or generate persistent private key
    info!("Loading server private key...");
    let private_key = load_or_generate_key()?;
    let public_key = private_key.verifying_key();
    info!(
        "Server public key loaded: {:?}",
        public_key.to_encoded_point(false).as_bytes()
    );

    // Configure rate limiting with environment variable support
    #[cfg(not(debug_assertions))]
    let requests_per_second = std::env::var("RATE_LIMIT_PER_SECOND")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    #[cfg(not(debug_assertions))]
    let burst_size = std::env::var("RATE_LIMIT_BURST_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);

    #[cfg(not(debug_assertions))]
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(requests_per_second)
            .burst_size(burst_size)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .ok_or_else(|| {
                ServerError::ConfigurationError(
                    "Failed to build rate limiter configuration".to_string(),
                )
            })?,
    );
    #[cfg(debug_assertions)]
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(u32::MAX)
            .key_extractor(GlobalKeyExtractor)
            .finish()
            .ok_or_else(|| {
                ServerError::ConfigurationError(
                    "Failed to build rate limiter configuration".to_string(),
                )
            })?,
    );

    #[cfg(not(debug_assertions))]
    info!(
        "Rate limiting configured: {} requests/second with burst size {}",
        requests_per_second, burst_size
    );

    // Start background task for rate limiter cleanup
    #[cfg(not(debug_assertions))]
    let governor_limiter = governor_conf.limiter().clone();
    #[cfg(not(debug_assertions))]
    let interval = Duration::from_secs(60);
    #[cfg(not(debug_assertions))]
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(interval);
            info!("Rate limiter storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    // Connect to MongoDB database
    info!("Connecting to MongoDB...");
    let db = database::connect_with_db().await?;

    // Optionally connect to PostgreSQL for migration
    let pg_pool = if let Ok(pg_url) = std::env::var("POSTGRES_URL") {
        info!("PostgreSQL URL found, connecting to PostgreSQL...");
        match pg::create_pool(&pg_url).await {
            Ok(pool) => {
                info!("Successfully connected to PostgreSQL");

                // Run migrations if POSTGRES_RUN_MIGRATIONS=true
                if std::env::var("POSTGRES_RUN_MIGRATIONS").unwrap_or_default() == "true" {
                    info!("Running PostgreSQL migrations...");
                    if let Err(e) = pool.run_migrations().await {
                        tracing::warn!("Failed to run PostgreSQL migrations: {}", e);
                    } else {
                        info!("PostgreSQL migrations completed successfully");
                    }
                }

                Some(Arc::new(pool))
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to connect to PostgreSQL: {}. Continuing with MongoDB only.",
                    e
                );
                None
            }
        }
    } else {
        info!("No PostgreSQL URL configured, using MongoDB only");
        None
    };

    // Get server bind address from environment
    let bind_addr =
        std::env::var("SERVER_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    info!("Server will bind to: {}", bind_addr);

    let state = AppState {
        db,
        pg_pool,
        private_key,
        prometheus_handle,
    };
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/cert", get(cert))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/api/auth/register", post(register_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/api/entities/{eid}/beacons/", get(Beacon::get_handler))
        .route("/api/entities/{eid}/beacons", get(Beacon::get_handler))
        .route(
            "/api/entities/{eid}/beacons/{id}",
            get(Beacon::get_one_handler),
        )
        .route("/api/entities/{eid}/beacons", post(Beacon::create_handler))
        .route(
            "/api/entities/{eid}/beacons/{id}/unlocker",
            post(create_unlock_instance),
        )
        .route(
            "/api/entities/{eid}/beacons/{id}/unlocker/{instance}/status",
            put(update_unlock_instance),
        )
        .route(
            "/api/entities/{eid}/beacons/{id}/unlocker/{instance}/outcome",
            put(record_unlock_result),
        )
        .route("/api/entities/{eid}/beacons", put(Beacon::update_handler))
        .route("/api/entities/{eid}/beacons/", post(Beacon::create_handler))
        .route("/api/entities/{eid}/beacons/", put(Beacon::update_handler))
        .route(
            "/api/entities/{eid}/beacons/{id}",
            delete(Beacon::delete_handler),
        )
        .route("/api/entities/{eid}/areas", get(Area::get_handler))
        .route("/api/entities/{eid}/areas/", get(Area::get_handler))
        .route("/api/entities/{eid}/areas/{id}", get(Area::get_one_handler))
        .route("/api/entities/{eid}/areas", post(Area::create_handler))
        .route("/api/entities/{eid}/areas", put(Area::update_handler))
        .route("/api/entities/{eid}/areas/", post(Area::create_handler))
        .route("/api/entities/{eid}/areas/", put(Area::update_handler))
        .route(
            "/api/entities/{eid}/areas/{id}",
            delete(Area::delete_handler),
        )
        .route(
            "/api/entities/{eid}/areas/{aid}/beacons",
            get(Beacon::get_all_in_area_handler),
        )
        .route(
            "/api/entities/{eid}/areas/{aid}/merchants",
            get(Merchant::get_all_in_area_handler),
        )
        .route("/api/entities", get(Entity::search_entity_handler))
        .route("/api/entities/", get(Entity::search_entity_handler))
        .route("/api/entities/{id}", get(Entity::get_one_handler))
        .route("/api/entities/{id}/route", get(find_route))
        .route("/api/entities/{id}/route/", get(find_route))
        .route("/api/entities/{id}/route/point", get(find_route))
        .route("/api/entities/{id}/route/point/", get(find_route))
        .route("/api/entities", post(Entity::create_handler))
        .route("/api/entities", put(Entity::update_handler))
        .route("/api/entities/", post(Entity::create_handler))
        .route("/api/entities/", put(Entity::update_handler))
        .route("/api/entities/{id}", delete(Entity::delete_handler))
        .route("/api/entities/{eid}/merchants", get(Merchant::get_handler))
        .route("/api/entities/{eid}/merchants/", get(Merchant::get_handler))
        .route(
            "/api/entities/{eid}/merchants/{id}",
            get(Merchant::get_one_handler),
        )
        .route(
            "/api/entities/{eid}/merchants",
            post(Merchant::create_handler),
        )
        .route(
            "/api/entities/{eid}/merchants",
            put(Merchant::update_handler),
        )
        .route(
            "/api/entities/{eid}/merchants/",
            post(Merchant::create_handler),
        )
        .route(
            "/api/entities/{eid}/merchants/",
            put(Merchant::update_handler),
        )
        .route(
            "/api/entities/{eid}/merchants/{id}",
            delete(Merchant::delete_handler),
        )
        .route(
            "/api/entities/{eid}/connections",
            get(Connection::get_handler),
        )
        .route(
            "/api/entities/{eid}/connections/",
            get(Connection::get_handler),
        )
        .route(
            "/api/entities/{eid}/connections/{id}",
            get(Connection::get_one_handler),
        )
        .route(
            "/api/entities/{eid}/connections",
            post(Connection::create_handler),
        )
        .route(
            "/api/entities/{eid}/connections",
            put(Connection::update_handler),
        )
        .route(
            "/api/entities/{eid}/connections/",
            post(Connection::create_handler),
        )
        .route(
            "/api/entities/{eid}/connections/",
            put(Connection::update_handler),
        )
        .route(
            "/api/entities/{eid}/connections/{id}",
            delete(Connection::delete_handler),
        )
        // Firmware management routes
        .route("/api/firmwares", get(get_firmwares_handler))
        .route("/api/firmwares/upload", post(upload_firmware_handler))
        .route(
            "/api/firmwares/latest/{device}",
            get(get_latest_firmware_handler),
        )
        .route("/api/firmwares/{id}", get(get_firmware_by_id_handler))
        .route(
            "/api/firmwares/{id}/download",
            get(download_firmware_handler),
        )
        .route("/api/firmwares/{id}", delete(delete_firmware_handler))
        .layer(middleware::from_fn(metrics::track_metrics))
        .layer(GovernorLayer::new(governor_conf))
        .layer(cors)
        .with_state(state);

    info!("Starting HTTP server...");
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| {
            ServerError::ConfigurationError(format!(
                "Failed to bind to address '{}': {}. Please check if the port is already in use.",
                bind_addr, e
            ))
        })?;

    info!("Server listening on {}", bind_addr);
    info!("Health check endpoint: http://{}/health", bind_addr);
    info!("Metrics endpoint: http://{}/metrics", bind_addr);
    info!("API documentation: http://{}/", bind_addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| ServerError::InternalError(format!("Server error: {}", e)))?;

    Ok(())
}
