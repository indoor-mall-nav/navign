mod common;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use mongodb::Database;
use serde_json::{Value, json};
use tower::ServiceExt; // for `oneshot`

/// Helper to create an app instance with test database
async fn create_test_app(db: Database) -> Router {
    // TODO: Import the actual app creation logic from main.rs
    // For now, this is a placeholder
    unimplemented!("Need to extract app creation logic from main.rs into a reusable function");
}

/// Helper to make JSON POST request
async fn post_json(
    app: &mut Router,
    uri: &str,
    body: Value,
    auth_token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .uri(uri)
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(token) = auth_token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
    }

    let request = builder
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));

    (status, body_json)
}

/// Helper to make GET request
async fn get(app: &mut Router, uri: &str, auth_token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri).method("GET");

    if let Some(token) = auth_token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
    }

    let request = builder.body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));

    (status, body_json)
}

#[tokio::test]
async fn test_health_check() {
    common::init_logging();
    let db = common::setup_test_db().await;
    let mut app = create_test_app(db.clone()).await;

    let (status, _body) = get(&mut app, "/health", None).await;
    assert_eq!(status, StatusCode::OK);

    common::cleanup_test_db(&db).await;
}

#[tokio::test]
async fn test_user_registration_and_login() {
    common::init_logging();
    let db = common::setup_test_db().await;
    let mut app = create_test_app(db.clone()).await;

    // Register new user
    let register_payload = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "SecurePassword123!"
    });

    let (status, response) =
        post_json(&mut app, "/api/auth/register", register_payload, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        response.get("token").is_some(),
        "Expected token in response"
    );

    // Login with registered user
    let login_payload = json!({
        "username": "testuser",
        "password": "SecurePassword123!"
    });

    let (status, response) = post_json(&mut app, "/api/auth/login", login_payload, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        response.get("token").is_some(),
        "Expected token in response"
    );

    // Login with wrong password
    let wrong_login = json!({
        "username": "testuser",
        "password": "WrongPassword"
    });

    let (status, _) = post_json(&mut app, "/api/auth/login", wrong_login, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    common::cleanup_test_db(&db).await;
}

#[tokio::test]
async fn test_create_and_get_entity() {
    common::init_logging();
    let db = common::setup_test_db().await;
    let mut app = create_test_app(db.clone()).await;

    // First register and get auth token
    let register_payload = json!({
        "username": "entityuser",
        "email": "entity@example.com",
        "password": "Password123!"
    });

    let (_, response) = post_json(&mut app, "/api/auth/register", register_payload, None).await;
    let token = response["token"].as_str().unwrap();

    // Create entity
    let entity_data = common::create_test_entity_data();
    let (status, response) = post_json(&mut app, "/api/entities", entity_data, Some(token)).await;
    assert_eq!(status, StatusCode::CREATED);

    let entity_id = response["_id"].as_str().unwrap();

    // Get entity
    let uri = format!("/api/entities/{}", entity_id);
    let (status, response) = get(&mut app, &uri, Some(token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["name"], "Test Mall");
    assert_eq!(response["floors"].as_array().unwrap().len(), 3);

    common::cleanup_test_db(&db).await;
}

#[tokio::test]
async fn test_create_area_with_polygon() {
    common::init_logging();
    let db = common::setup_test_db().await;
    let mut app = create_test_app(db.clone()).await;

    // Setup: Register user and create entity
    let register_payload = json!({
        "username": "areauser",
        "email": "area@example.com",
        "password": "Password123!"
    });
    let (_, response) = post_json(&mut app, "/api/auth/register", register_payload, None).await;
    let token = response["token"].as_str().unwrap();

    let entity_data = common::create_test_entity_data();
    let (_, response) = post_json(&mut app, "/api/entities", entity_data, Some(token)).await;
    let entity_id = response["_id"].as_str().unwrap().to_string();

    // Create area
    let area_data = common::create_test_area_data(&entity_id, 1);
    let uri = format!("/api/entities/{}/areas", entity_id);
    let (status, response) = post_json(&mut app, &uri, area_data, Some(token)).await;
    assert_eq!(status, StatusCode::CREATED);

    let area_id = response["_id"].as_str().unwrap();

    // Get area
    let uri = format!("/api/entities/{}/areas/{}", entity_id, area_id);
    let (status, response) = get(&mut app, &uri, Some(token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["name"], "Test Area 1");
    assert!(response["polygon"].as_str().unwrap().starts_with("POLYGON"));

    common::cleanup_test_db(&db).await;
}

#[tokio::test]
async fn test_create_beacon_and_merchant() {
    common::init_logging();
    let db = common::setup_test_db().await;
    let mut app = create_test_app(db.clone()).await;

    // Setup: Register, create entity and area
    let register_payload = json!({
        "username": "beaconuser",
        "email": "beacon@example.com",
        "password": "Password123!"
    });
    let (_, response) = post_json(&mut app, "/api/auth/register", register_payload, None).await;
    let token = response["token"].as_str().unwrap();

    let entity_data = common::create_test_entity_data();
    let (_, response) = post_json(&mut app, "/api/entities", entity_data, Some(token)).await;
    let entity_id = response["_id"].as_str().unwrap().to_string();

    let area_data = common::create_test_area_data(&entity_id, 1);
    let uri = format!("/api/entities/{}/areas", entity_id);
    let (_, response) = post_json(&mut app, &uri, area_data, Some(token)).await;
    let area_id = response["_id"].as_str().unwrap().to_string();

    // Create beacon
    let beacon_data = common::create_test_beacon_data(&entity_id, &area_id);
    let uri = format!("/api/entities/{}/beacons", entity_id);
    let (status, response) = post_json(&mut app, &uri, beacon_data, Some(token)).await;
    assert_eq!(status, StatusCode::CREATED);

    let beacon_id = response["_id"].as_str().unwrap();

    // Get beacon
    let uri = format!("/api/entities/{}/beacons/{}", entity_id, beacon_id);
    let (status, response) = get(&mut app, &uri, Some(token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["name"], "Test Beacon");
    assert_eq!(response["beacon_type"], "pathway");

    // Create merchant
    let merchant_data = common::create_test_merchant_data(&entity_id, &area_id);
    let uri = format!("/api/entities/{}/merchants", entity_id);
    let (status, response) = post_json(&mut app, &uri, merchant_data, Some(token)).await;
    assert_eq!(status, StatusCode::CREATED);

    let merchant_id = response["_id"].as_str().unwrap();

    // Get merchant
    let uri = format!("/api/entities/{}/merchants/{}", entity_id, merchant_id);
    let (status, response) = get(&mut app, &uri, Some(token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["name"], "Test Coffee Shop");
    assert_eq!(response["tags"].as_array().unwrap().len(), 3);

    common::cleanup_test_db(&db).await;
}

#[tokio::test]
async fn test_pathfinding_route() {
    common::init_logging();
    let db = common::setup_test_db().await;
    let mut app = create_test_app(db.clone()).await;

    // Setup: Create entity with multiple areas, merchants, and connections
    let register_payload = json!({
        "username": "routeuser",
        "email": "route@example.com",
        "password": "Password123!"
    });
    let (_, response) = post_json(&mut app, "/api/auth/register", register_payload, None).await;
    let token = response["token"].as_str().unwrap();

    let entity_data = common::create_test_entity_data();
    let (_, response) = post_json(&mut app, "/api/entities", entity_data, Some(token)).await;
    let entity_id = response["_id"].as_str().unwrap().to_string();

    // Create two areas on different floors
    let area1_data = common::create_test_area_data(&entity_id, 1);
    let uri = format!("/api/entities/{}/areas", entity_id);
    let (_, response) = post_json(&mut app, &uri, area1_data, Some(token)).await;
    let area1_id = response["_id"].as_str().unwrap().to_string();

    let area2_data = common::create_test_area_data(&entity_id, 2);
    let (_, response) = post_json(&mut app, &uri, area2_data, Some(token)).await;
    let area2_id = response["_id"].as_str().unwrap().to_string();

    // Create connection between areas (elevator)
    let connection_data = common::create_test_connection_data(&entity_id, &area1_id, &area2_id);
    let uri = format!("/api/entities/{}/connections", entity_id);
    let (status, _) = post_json(&mut app, &uri, connection_data, Some(token)).await;
    assert_eq!(status, StatusCode::CREATED);

    // Create merchants in each area
    let merchant1_data = common::create_test_merchant_data(&entity_id, &area1_id);
    let uri = format!("/api/entities/{}/merchants", entity_id);
    let (_, response) = post_json(&mut app, &uri, merchant1_data, Some(token)).await;
    let merchant1_id = response["_id"].as_str().unwrap().to_string();

    let mut merchant2_data = common::create_test_merchant_data(&entity_id, &area2_id);
    merchant2_data["name"] = json!("Test Restaurant");
    merchant2_data["beacon_code"] = json!("MERCH002");
    let (_, response) = post_json(&mut app, &uri, merchant2_data, Some(token)).await;
    let merchant2_id = response["_id"].as_str().unwrap().to_string();

    // Request route from merchant1 to merchant2
    let uri = format!(
        "/api/entities/{}/route?from={}&to={}",
        entity_id, merchant1_id, merchant2_id
    );
    let (status, response) = get(&mut app, &uri, Some(token)).await;
    assert_eq!(status, StatusCode::OK);

    // Verify route contains instructions
    assert!(response.get("instructions").is_some());
    let instructions = response["instructions"].as_array().unwrap();
    assert!(instructions.len() > 0, "Expected navigation instructions");

    // Should include a transport instruction for the elevator
    let has_transport = instructions.iter().any(|i| i.get("transport").is_some());
    assert!(
        has_transport,
        "Expected transport instruction for floor change"
    );

    common::cleanup_test_db(&db).await;
}

#[tokio::test]
async fn test_unauthorized_access() {
    common::init_logging();
    let db = common::setup_test_db().await;
    let mut app = create_test_app(db.clone()).await;

    // Try to create entity without token
    let entity_data = common::create_test_entity_data();
    let (status, _) = post_json(&mut app, "/api/entities", entity_data, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Try with invalid token
    let entity_data = common::create_test_entity_data();
    let (status, _) = post_json(
        &mut app,
        "/api/entities",
        entity_data,
        Some("invalid_token"),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    common::cleanup_test_db(&db).await;
}

#[tokio::test]
async fn test_rate_limiting() {
    common::init_logging();
    let db = common::setup_test_db().await;
    let mut app = create_test_app(db.clone()).await;

    // Make many rapid requests
    let mut rate_limited = false;
    for _ in 0..300 {
        let (status, _) = get(&mut app, "/health", None).await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            rate_limited = true;
            break;
        }
    }

    assert!(
        rate_limited,
        "Expected rate limiting to kick in after 300 requests"
    );

    common::cleanup_test_db(&db).await;
}
