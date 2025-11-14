use mongodb::{Client, Database};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize logging for tests (only once)
pub fn init_logging() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}

/// Setup test database connection
pub async fn setup_test_db() -> Database {
    let mongodb_uri =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    let client = Client::with_uri_str(&mongodb_uri)
        .await
        .expect("Failed to connect to MongoDB for tests");

    let db_name = format!("navign_test_{}", chrono::Utc::now().timestamp());
    client.database(&db_name)
}

/// Cleanup test database
pub async fn cleanup_test_db(db: &Database) {
    db.drop().await.expect("Failed to drop test database");
}

/// Create a test entity with some default data
pub fn create_test_entity_data() -> serde_json::Value {
    serde_json::json!({
        "name": "Test Mall",
        "description": "A test shopping mall",
        "address": {
            "street": "123 Test St",
            "city": "Test City",
            "state": "TS",
            "zip": "12345",
            "country": "Test Country"
        },
        "floors": [
            {"type": "floor", "name": 1},
            {"type": "floor", "name": 2},
            {"type": "floor", "name": 3}
        ],
        "opening_hours": {
            "monday": {"open": "09:00", "close": "21:00"},
            "tuesday": {"open": "09:00", "close": "21:00"},
            "wednesday": {"open": "09:00", "close": "21:00"},
            "thursday": {"open": "09:00", "close": "21:00"},
            "friday": {"open": "09:00", "close": "21:00"},
            "saturday": {"open": "10:00", "close": "22:00"},
            "sunday": {"open": "10:00", "close": "20:00"}
        }
    })
}

/// Create a test area with polygon
pub fn create_test_area_data(entity_id: &str, floor: i32) -> serde_json::Value {
    serde_json::json!({
        "entity": entity_id,
        "name": format!("Test Area {}", floor),
        "description": "A test area for unit testing",
        "beacon_code": format!("AREA{:03}", floor),
        "floor": {"type": "floor", "name": floor},
        "polygon": "POLYGON((0 0, 100 0, 100 100, 0 100, 0 0))"
    })
}

/// Create a test beacon
pub fn create_test_beacon_data(entity_id: &str, area_id: &str) -> serde_json::Value {
    serde_json::json!({
        "entity": entity_id,
        "area": area_id,
        "name": "Test Beacon",
        "device_id": "000102030405060708090a0b0c0d0e0f10111213",
        "beacon_type": "pathway",
        "capabilities": ["battery_status"],
        "floor": {"type": "floor", "name": 1},
        "location": [50.0, 50.0]
    })
}

/// Create a test merchant
pub fn create_test_merchant_data(entity_id: &str, area_id: &str) -> serde_json::Value {
    serde_json::json!({
        "entity": entity_id,
        "area": area_id,
        "name": "Test Coffee Shop",
        "description": "A cozy coffee shop",
        "beacon_code": "MERCH001",
        "location": [25.0, 25.0],
        "floor": {"type": "floor", "name": 1},
        "tags": ["food", "coffee", "cafe"],
        "type": {"food": {"cuisine": "american", "type": "cafe"}},
        "style": "store"
    })
}

/// Create a test connection (elevator, stairs, etc)
pub fn create_test_connection_data(
    entity_id: &str,
    from_area: &str,
    to_area: &str,
) -> serde_json::Value {
    serde_json::json!({
        "entity": entity_id,
        "from_area": from_area,
        "to_area": to_area,
        "connection_type": "elevator",
        "name": "Main Elevator",
        "bidirectional": true
    })
}
