mod database;
mod schema;
mod shared;

use crate::schema::{Area, Beacon, Connection, Entity, Merchant, Service};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{
    Router,
    http::{Method, StatusCode},
    routing::{delete, get, post, put},
};
use bson::doc;
use log::{LevelFilter, info};
use mongodb::Database;
use simple_logger::SimpleLogger;
use tower_http::cors::CorsLayer;

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::set_boxed_logger(Box::new(SimpleLogger::new()))
        .map(|()| log::set_max_level(LevelFilter::Info))?;
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);
    info!("Cors layer configured.");
    let db = database::connect_with_db().await?;
    let state = AppState { db };
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/api/beacon", get(Beacon::get_handler))
        .route("/api/beacon/{id}", get(Beacon::get_one_handler))
        .route("/api/beacon", post(Beacon::create_handler))
        .route("/api/beacon", put(Beacon::update_handler))
        .route("/api/beacon/{id}", delete(Beacon::delete_handler))
        .route("/api/area", get(Area::get_handler))
        .route("/api/area/{id}", get(Area::get_one_handler))
        .route("/api/area", post(Area::create_handler))
        .route("/api/area", put(Area::update_handler))
        .route("/api/area/{id}", delete(Area::delete_handler))
        .route("/api/entity", get(Entity::get_handler))
        .route("/api/entity/{id}", get(Entity::get_one_handler))
        .route("/api/entity", post(Entity::create_handler))
        .route("/api/entity", put(Entity::update_handler))
        .route("/api/entity/{id}", delete(Entity::delete_handler))
        .route("/api/merchant", get(Merchant::get_handler))
        .route("/api/merchant/{id}", get(Merchant::get_one_handler))
        .route("/api/merchant", post(Merchant::create_handler))
        .route("/api/merchant", put(Merchant::update_handler))
        .route("/api/merchant/{id}", delete(Merchant::delete_handler))
        .route("/api/connection", get(Connection::get_handler))
        .route("/api/connection/{id}", get(Connection::get_one_handler))
        .route("/api/connection", post(Connection::create_handler))
        .route("/api/connection", put(Connection::update_handler))
        .route("/api/connection/{id}", delete(Connection::delete_handler))
        .layer(cors)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
