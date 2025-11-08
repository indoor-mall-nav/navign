#![allow(dead_code)]

mod firmware_api;
mod grpc_service;
mod robot_registry;
mod task_queue;
mod types;

use axum::{Router, routing::get};
use firmware_api::{
    AppState, FirmwareClient, download_firmware_handler, get_firmware_by_id_handler,
    get_latest_firmware_handler, health_handler, list_firmwares_handler,
};
use grpc_service::OrchestratorServiceImpl;
use robot_registry::RobotRegistry;
use std::sync::Arc;
use tonic::transport::Server;
use tower_http::cors::CorsLayer;
use types::task::orchestrator_service_server::OrchestratorServiceServer;

// Example function to demonstrate task creation (in real app, this would be called based on business logic)
#[allow(dead_code)]
async fn create_example_task(registry: &RobotRegistry, entity_id: &str) {
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};
    use types::{Priority, Task, TaskType, task};

    let task = Task {
        id: uuid::Uuid::new_v4().to_string(),
        r#type: TaskType::Delivery as i32,
        sources: vec![task::Location {
            x: 100.0,
            y: 200.0,
            z: 0.0,
            floor: "1F".to_string(),
        }],
        terminals: vec![task::Location {
            x: 500.0,
            y: 600.0,
            z: 0.0,
            floor: "2F".to_string(),
        }],
        priority: Priority::Normal as i32,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        entity_id: entity_id.to_string(),
        metadata: HashMap::new(),
    };

    match registry.assign_task(task).await {
        Ok(robot_id) => log::info!("Task assigned to robot: {}", robot_id),
        Err(e) => log::warn!("Failed to assign task: {}", e),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Get configuration from environment variables
    let grpc_addr = std::env::var("ORCHESTRATOR_GRPC_ADDR")
        .unwrap_or_else(|_| "[::1]:50051".to_string())
        .parse()?;

    let http_addr =
        std::env::var("ORCHESTRATOR_HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".to_string());

    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    log::info!("Orchestrator starting...");
    log::info!("  gRPC server: {}", grpc_addr);
    log::info!("  HTTP server: {}", http_addr);
    log::info!("  Backend server: {}", server_url);

    // Create orchestrator service for gRPC
    let orchestrator = OrchestratorServiceImpl::new();

    // Create firmware client for HTTP API
    let firmware_client = Arc::new(FirmwareClient::new(server_url));
    let app_state = AppState { firmware_client };

    // Configure CORS for HTTP server
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers(tower_http::cors::Any);

    // Create HTTP router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/firmwares", get(list_firmwares_handler))
        .route(
            "/firmwares/latest/:device",
            get(get_latest_firmware_handler),
        )
        .route("/firmwares/:id", get(get_firmware_by_id_handler))
        .route("/firmwares/:id/download", get(download_firmware_handler))
        .layer(cors)
        .with_state(app_state);

    // Create gRPC server future
    let grpc_server = async move {
        log::info!("gRPC server listening on {}", grpc_addr);
        Server::builder()
            .add_service(OrchestratorServiceServer::new(orchestrator))
            .serve(grpc_addr)
            .await
    };

    // Create HTTP server future
    let http_server = async move {
        let listener = tokio::net::TcpListener::bind(&http_addr).await.unwrap();
        log::info!("HTTP server listening on {}", http_addr);
        axum::serve(listener, app).await
    };

    // Run both servers concurrently
    log::info!("Both servers started successfully");

    tokio::select! {
        result = grpc_server => {
            if let Err(e) = result {
                log::error!("gRPC server error: {}", e);
                return Err(e.into());
            }
        }
        result = http_server => {
            if let Err(e) = result {
                log::error!("HTTP server error: {}", e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}
