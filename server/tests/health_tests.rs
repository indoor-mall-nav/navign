/// Basic health and connectivity tests for the Navign server
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use mongodb::Client;
use navign_server::{create_app, connect_with_db, AppState};
use p256::ecdsa::{SigningKey, signature::rand_core::OsRng};
use tower::ServiceExt;

/// Create a test app with a test database
async fn setup_test_app() -> (AppState, axum::Router) {
    // Connect to MongoDB
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    let client = Client::with_uri_str(&database_url)
        .await
        .expect("Failed to connect to MongoDB for tests");

    // Create a unique test database
    let db_name = format!("navign_test_{}", uuid::Uuid::new_v4());
    let db = client.database(&db_name);

    // Generate test private key
    let private_key = SigningKey::random(&mut OsRng);

    let state = AppState {
        db: db.clone(),
        private_key,
    };

    let app = create_app(state.clone());

    (state, app)
}

/// Clean up test database
async fn cleanup_test_db(state: &AppState) {
    let db_name = state.db.name().to_string();
    state
        .db
        .drop()
        .await
        .expect("Failed to drop test database");
    eprintln!("Dropped test database: {}", db_name);
}

#[tokio::test]
async fn test_root_endpoint() {
    let (_state, app) = setup_test_app().await;

    let request = Request::builder()
        .uri("/")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(body_str, "Hello, World!");
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let (state, app) = setup_test_app().await;

    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(body_str, "Healthy");

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_cert_endpoint_returns_pem() {
    let (state, app) = setup_test_app().await;

    let request = Request::builder()
        .uri("/cert")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let pem_str = String::from_utf8(body.to_vec()).unwrap();

    // Check that it's a valid PEM format
    assert!(pem_str.starts_with("-----BEGIN PUBLIC KEY-----"));
    assert!(pem_str.ends_with("-----END PUBLIC KEY-----\n"));

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_nonexistent_route_returns_404() {
    let (state, app) = setup_test_app().await;

    let request = Request::builder()
        .uri("/nonexistent")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_test_db(&state).await;
}
