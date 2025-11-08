// Library exports for testing and reusability

mod certification;
mod database;
mod kernel;
mod key_management;
mod schema;
mod shared;

use crate::kernel::auth::{login_handler, register_handler};
use crate::kernel::route::find_route;
use crate::kernel::unlocker::{
    create_unlock_instance, record_unlock_result, update_unlock_instance,
};
use crate::schema::firmware::{
    delete_firmware_handler, download_firmware_handler, get_firmware_by_id_handler,
    get_firmwares_handler, get_latest_firmware_handler, upload_firmware_handler,
};
use crate::schema::{Area, Beacon, Connection, Entity, Merchant, Service, EntityServiceAddons};
use crate::schema::service::OneInArea;
use axum::{
    Router,
    http::StatusCode,
    routing::{delete, get, post, put},
    extract::State,
    response::IntoResponse,
};
use bson::doc;
use log::info;
use mongodb::Database;
use p256::ecdsa::SigningKey;
use p256::pkcs8::EncodePublicKey;
use rsa::pkcs1::LineEnding;

async fn root() -> impl IntoResponse {
    (StatusCode::OK, "Hello, World!")
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.run_command(doc! { "ping": 1 }).await {
        Ok(_) => (StatusCode::OK, "Healthy"),
        Err(e) => {
            info!("Health check failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Unhealthy")
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub private_key: SigningKey,
}

async fn cert(State(state): State<AppState>) -> impl IntoResponse {
    match state
        .private_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
    {
        Ok(res) => (StatusCode::OK, res),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// Create the Axum router with all routes configured
/// This function is used by both the main server and tests
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/cert", get(cert))
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
            "/api/firmwares/latest/:device",
            get(get_latest_firmware_handler),
        )
        .route("/api/firmwares/:id", get(get_firmware_by_id_handler))
        .route(
            "/api/firmwares/:id/download",
            get(download_firmware_handler),
        )
        .route("/api/firmwares/:id", delete(delete_firmware_handler))
        .with_state(state)
}

// Re-export modules for main and tests
pub use key_management::load_or_generate_key;

// Database connection function
pub async fn connect_with_db() -> anyhow::Result<mongodb::Database> {
    database::connect_with_db().await
}
