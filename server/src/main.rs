mod certification;
mod error;
mod kernel;
mod key_management;
mod metrics;
mod pg;
mod schema;
mod state;

use crate::error::{Result as ServerResult, ServerError};
use crate::key_management::load_or_generate_key;
use crate::pg::{IntCrudRepository, IntCrudRepositoryInArea, UuidCrudRepository};
use crate::state::AppState;
use axum::extract::State;
use axum::middleware;
use axum::response::IntoResponse;
use axum::{
    Router,
    http::{Method, StatusCode},
    routing::{delete, get, post, put},
};
use navign_shared::schema::{Area, Beacon, Connection, Entity, Merchant};
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
    match state
        .pg_pool
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::DatabaseConnection(format!("PostgreSQL connection error: {}", e)))
    {
        Ok(_) => (StatusCode::OK, "Healthy").into_response(),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "Unhealthy").into_response(),
    }
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

                Some(pool)
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

    let Some(pg_pool) = pg_pool else {
        return Err(ServerError::ConfigurationError(
            "PostgreSQL connection is required but could not be established.".to_string(),
        ));
    };

    // Get server bind address from environment
    let bind_addr =
        std::env::var("SERVER_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    info!("Server will bind to: {}", bind_addr);

    let state = AppState {
        pg_pool,
        private_key,
        prometheus_handle,
    };
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/cert", get(cert))
        .route("/metrics", get(metrics::metrics_handler))
        // Entity endpoints (UUID-based)
        .route("/api/entities", get(Entity::crud_search))
        .route("/api/entities", post(Entity::crud_create))
        .route("/api/entities", put(Entity::crud_update))
        .route("/api/entities/{id}", get(Entity::crud_read_one))
        .route("/api/entities/{id}", delete(Entity::crud_delete))
        // Area endpoints (Int-based, entity-scoped)
        .route("/api/entities/{entity}/areas", get(Area::crud_search))
        .route("/api/entities/{entity}/areas", post(Area::crud_create))
        .route("/api/entities/{entity}/areas", put(Area::crud_update))
        .route(
            "/api/entities/{entity}/areas/{id}",
            get(Area::crud_read_one),
        )
        .route(
            "/api/entities/{entity}/areas/{id}",
            delete(Area::crud_delete),
        )
        // Beacon endpoints (Int-based, entity-scoped)
        .route("/api/entities/{entity}/beacons", get(Beacon::crud_search))
        .route("/api/entities/{entity}/beacons", post(Beacon::crud_create))
        .route("/api/entities/{entity}/beacons", put(Beacon::crud_update))
        .route(
            "/api/entities/{entity}/beacons/{id}",
            get(Beacon::crud_read_one),
        )
        .route(
            "/api/entities/{entity}/beacons/{id}",
            delete(Beacon::crud_delete),
        )
        // Merchant endpoints (Int-based, entity-scoped)
        .route(
            "/api/entities/{entity}/merchants",
            get(Merchant::crud_search),
        )
        .route(
            "/api/entities/{entity}/merchants",
            post(Merchant::crud_create),
        )
        .route(
            "/api/entities/{entity}/merchants",
            put(Merchant::crud_update),
        )
        .route(
            "/api/entities/{entity}/merchants/{id}",
            get(Merchant::crud_read_one),
        )
        .route(
            "/api/entities/{entity}/merchants/{id}",
            delete(Merchant::crud_delete),
        )
        // Merchant area-scoped search
        .route(
            "/api/entities/{entity}/areas/{area}/merchants",
            get(Merchant::crud_search_in_area),
        )
        // Beacon area-scoped search
        .route(
            "/api/entities/{entity}/areas/{area}/beacons",
            get(Beacon::crud_search_in_area),
        )
        // Connection endpoints (Int-based, entity-scoped)
        .route(
            "/api/entities/{entity}/connections",
            get(Connection::crud_search),
        )
        .route(
            "/api/entities/{entity}/connections",
            post(Connection::crud_create),
        )
        .route(
            "/api/entities/{entity}/connections",
            put(Connection::crud_update),
        )
        .route(
            "/api/entities/{entity}/connections/{id}",
            get(Connection::crud_read_one),
        )
        .route(
            "/api/entities/{entity}/connections/{id}",
            delete(Connection::crud_delete),
        )
        // Route finding endpoint
        .route(
            "/api/entities/{entity}/route",
            get(kernel::route::find_route),
        )
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
