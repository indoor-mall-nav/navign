mod database;
mod schema;

use axum::response::IntoResponse;
use axum::{
    http::{Method, StatusCode},
    routing::{get, post},
    Router,
};
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use std::sync::{Arc, Mutex};
use axum::extract::State;
use bson::doc;
use mongodb::Database;
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
    db: Database
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
        .layer(cors)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
