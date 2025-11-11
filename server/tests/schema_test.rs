use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use uuid::Uuid;

/// Helper function to create a test database connection
async fn create_test_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/navign_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

/// Helper function to clean up test database
async fn cleanup_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("TRUNCATE TABLE unlock_instances, beacon_secrets, beacons, merchants, connections, areas, users, firmwares, entities CASCADE")
        .execute(pool)
        .await?;
    Ok(())
}

/// Helper to create a test entity
async fn create_test_entity(pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    let entity_id = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO entities (id, type, name, longitude_min, longitude_max,
         latitude_min, latitude_max, tags, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(entity_id)
    .bind("mall")
    .bind("Test Mall")
    .bind(-122.5)
    .bind(-122.4)
    .bind(37.7)
    .bind(37.8)
    .bind(serde_json::json!([]))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(entity_id)
}

/// Helper to create a test area
async fn create_test_area(pool: &PgPool, entity_id: Uuid) -> Result<i64, sqlx::Error> {
    let now = chrono::Utc::now().timestamp_millis();

    let area_id: i64 = sqlx::query_scalar(
        "INSERT INTO areas (entity, name, beacon_code, polygon, created_at, updated_at)
         VALUES ($1, $2, $3, ST_GeomFromText($4, 4326), $5, $6) RETURNING id",
    )
    .bind(entity_id)
    .bind("Test Area")
    .bind("AREA001")
    .bind("POLYGON((0 0, 0 10, 10 10, 10 0, 0 0))")
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(area_id)
}

#[cfg(test)]
mod merchant_tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_merchant_with_jsonb() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area");
        let now = chrono::Utc::now().timestamp_millis();

        // Create merchant with JSONB type field
        let merchant_type = serde_json::json!({
            "category": "restaurant",
            "cuisine": "italian"
        });

        let result = sqlx::query(
            "INSERT INTO merchants (entity, area, name, beacon_code, type, location, style, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, ST_GeomFromText($6, 4326), $7, $8, $9)"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Italian Restaurant")
        .bind("MERCHANT001")
        .bind(&merchant_type)
        .bind("POINT(5 5)")
        .bind("point")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Failed to insert merchant");
        assert_eq!(result.unwrap().rows_affected(), 1);
    }

    #[tokio::test]
    async fn test_merchant_with_polygon_style() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area");
        let now = chrono::Utc::now().timestamp_millis();

        // Create merchant with polygon style
        let result = sqlx::query(
            "INSERT INTO merchants (entity, area, name, beacon_code, type, location, style, polygon, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, ST_GeomFromText($6, 4326), $7, ST_GeomFromText($8, 4326), $9, $10)"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Large Store")
        .bind("MERCHANT002")
        .bind(serde_json::json!({"category": "retail"}))
        .bind("POINT(5 5)")
        .bind("polygon")
        .bind("POLYGON((3 3, 3 7, 7 7, 7 3, 3 3))")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Failed to insert merchant with polygon");
    }

    #[tokio::test]
    async fn test_merchant_with_tags() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area");
        let now = chrono::Utc::now().timestamp_millis();

        let tags = serde_json::json!(["food", "fast-food", "halal"]);

        sqlx::query(
            "INSERT INTO merchants (entity, area, name, beacon_code, type, location, style, tags, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, ST_GeomFromText($6, 4326), $7, $8, $9, $10)"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Food Court")
        .bind("MERCHANT003")
        .bind(serde_json::json!({"category": "food"}))
        .bind("POINT(5 5)")
        .bind("point")
        .bind(&tags)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert merchant");

        // Query using JSONB contains
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM merchants WHERE tags @> $1")
            .bind(serde_json::json!(["halal"]))
            .fetch_one(&pool)
            .await
            .expect("Failed to query");

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_merchant_with_social_media() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area");
        let now = chrono::Utc::now().timestamp_millis();

        let social_media = serde_json::json!({
            "instagram": "test_merchant",
            "facebook": "testmerchant",
            "twitter": "@testmerch"
        });

        sqlx::query(
            "INSERT INTO merchants (entity, area, name, beacon_code, type, location, style, social_media, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, ST_GeomFromText($6, 4326), $7, $8, $9, $10)"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Social Merchant")
        .bind("MERCHANT004")
        .bind(serde_json::json!({"category": "retail"}))
        .bind("POINT(5 5)")
        .bind("point")
        .bind(&social_media)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert merchant");

        // Retrieve and verify
        let row =
            sqlx::query("SELECT social_media FROM merchants WHERE beacon_code = 'MERCHANT004'")
                .fetch_one(&pool)
                .await
                .expect("Failed to select");

        let retrieved: serde_json::Value = row.get("social_media");
        assert_eq!(retrieved["instagram"], "test_merchant");
    }

    #[tokio::test]
    async fn test_merchant_unique_constraint() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area");
        let now = chrono::Utc::now().timestamp_millis();

        // Insert first merchant
        sqlx::query(
            "INSERT INTO merchants (entity, area, name, beacon_code, type, location, style, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, ST_GeomFromText($6, 4326), $7, $8, $9)"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Unique Merchant")
        .bind("MERCHANT005")
        .bind(serde_json::json!({"category": "retail"}))
        .bind("POINT(5 5)")
        .bind("point")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert first merchant");

        // Try to insert duplicate name in same entity (should fail)
        let result = sqlx::query(
            "INSERT INTO merchants (entity, area, name, beacon_code, type, location, style, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, ST_GeomFromText($6, 4326), $7, $8, $9)"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Unique Merchant")  // Same name
        .bind("MERCHANT006")
        .bind(serde_json::json!({"category": "retail"}))
        .bind("POINT(5 5)")
        .bind("point")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "Should fail due to unique constraint");
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_connection() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area1_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area 1");

        // Create second area
        let now = chrono::Utc::now().timestamp_millis();
        let area2_id: i64 = sqlx::query_scalar(
            "INSERT INTO areas (entity, name, beacon_code, polygon, created_at, updated_at)
             VALUES ($1, $2, $3, ST_GeomFromText($4, 4326), $5, $6) RETURNING id",
        )
        .bind(entity_id)
        .bind("Test Area 2")
        .bind("AREA002")
        .bind("POLYGON((10 10, 10 20, 20 20, 20 10, 10 10))")
        .bind(now)
        .bind(now)
        .fetch_one(&pool)
        .await
        .expect("Failed to create area 2");

        // Create connection with JSONB connected_areas
        let connected_areas = serde_json::json!([area1_id, area2_id]);

        let result = sqlx::query(
            "INSERT INTO connections (entity, name, type, connected_areas, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(entity_id)
        .bind("Test Elevator")
        .bind("elevator")
        .bind(&connected_areas)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Failed to insert connection");
        assert_eq!(result.unwrap().rows_affected(), 1);
    }

    #[tokio::test]
    async fn test_connection_with_available_period() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let now = chrono::Utc::now().timestamp_millis();

        // Available period as JSONB array of tuples
        let available_period = serde_json::json!([
            [0, 800],     // 00:00 to 08:00
            [1700, 2359]  // 17:00 to 23:59
        ]);

        let result = sqlx::query(
            "INSERT INTO connections (entity, name, type, connected_areas, available_period, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(entity_id)
        .bind("Night Elevator")
        .bind("elevator")
        .bind(serde_json::json!([1, 2]))
        .bind(&available_period)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(
            result.is_ok(),
            "Failed to insert connection with available period"
        );
    }

    #[tokio::test]
    async fn test_connection_types() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let now = chrono::Utc::now().timestamp_millis();

        let connection_types = vec!["gate", "escalator", "elevator", "stairs", "rail", "shuttle"];

        for conn_type in connection_types {
            let name = format!("Test {}", conn_type);
            let result = sqlx::query(
                "INSERT INTO connections (entity, name, type, connected_areas, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6)"
            )
            .bind(entity_id)
            .bind(&name)
            .bind(conn_type)
            .bind(serde_json::json!([1, 2]))
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await;

            assert!(result.is_ok(), "Failed to insert {} connection", conn_type);
        }

        // Verify count
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM connections WHERE entity = $1")
                .bind(entity_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to count");

        assert_eq!(count, 6);
    }

    #[tokio::test]
    async fn test_connection_with_ground_point() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let now = chrono::Utc::now().timestamp_millis();

        let result = sqlx::query(
            "INSERT INTO connections (entity, name, type, connected_areas, gnd, created_at, updated_at)
             VALUES ($1, $2, $3, $4, ST_GeomFromText($5, 4326), $6, $7)"
        )
        .bind(entity_id)
        .bind("Ground Entrance")
        .bind("gate")
        .bind(serde_json::json!([1, 2]))
        .bind("POINT(10 10)")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(
            result.is_ok(),
            "Failed to insert connection with ground point"
        );
    }

    #[tokio::test]
    async fn test_connection_with_tags() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let now = chrono::Utc::now().timestamp_millis();

        let tags = serde_json::json!(["accessible", "wheelchair", "express"]);

        sqlx::query(
            "INSERT INTO connections (entity, name, type, connected_areas, tags, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(entity_id)
        .bind("Accessible Elevator")
        .bind("elevator")
        .bind(serde_json::json!([1, 2]))
        .bind(&tags)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert");

        // Query connections with specific tag
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM connections WHERE tags @> $1")
                .bind(serde_json::json!(["wheelchair"]))
                .fetch_one(&pool)
                .await
                .expect("Failed to query");

        assert_eq!(count, 1);
    }
}

#[cfg(test)]
mod foreign_key_tests {
    use super::*;

    #[tokio::test]
    async fn test_cascade_delete_entity() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area");

        // Create beacon
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO beacons (entity, area, name, type, location, device, mac, created_at, updated_at)
             VALUES ($1, $2, $3, $4, ST_GeomFromText($5, 4326), $6, $7, $8, $9)"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Test Beacon")
        .bind("navigation")
        .bind("POINT(5 5)")
        .bind("esp32c3")
        .bind("AA:BB:CC:DD:EE:FF")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to create beacon");

        // Delete entity (should cascade to areas and beacons)
        sqlx::query("DELETE FROM entities WHERE id = $1")
            .bind(entity_id)
            .execute(&pool)
            .await
            .expect("Failed to delete entity");

        // Verify areas are deleted
        let area_count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM areas WHERE entity = $1")
                .bind(entity_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to count areas");

        assert_eq!(area_count, 0, "Areas should be cascade deleted");

        // Verify beacons are deleted
        let beacon_count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM beacons WHERE entity = $1")
                .bind(entity_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to count beacons");

        assert_eq!(beacon_count, 0, "Beacons should be cascade deleted");
    }

    #[tokio::test]
    async fn test_cascade_delete_area() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area");

        // Create beacon in area
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO beacons (entity, area, name, type, location, device, mac, created_at, updated_at)
             VALUES ($1, $2, $3, $4, ST_GeomFromText($5, 4326), $6, $7, $8, $9)"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Area Beacon")
        .bind("navigation")
        .bind("POINT(5 5)")
        .bind("esp32c3")
        .bind("AA:BB:CC:DD:EE:FF")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to create beacon");

        // Delete area
        sqlx::query("DELETE FROM areas WHERE id = $1")
            .bind(area_id)
            .execute(&pool)
            .await
            .expect("Failed to delete area");

        // Verify beacon is deleted
        let beacon_count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM beacons WHERE area = $1")
                .bind(area_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to count beacons");

        assert_eq!(beacon_count, 0, "Beacons should be cascade deleted");
    }

    #[tokio::test]
    async fn test_set_null_on_delete_merchant() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area");

        // Create merchant
        let now = chrono::Utc::now().timestamp_millis();
        let merchant_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO merchants (id, entity, area, name, beacon_code, type, location, style, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, ST_GeomFromText($7, 4326), $8, $9, $10)"
        )
        .bind(merchant_id)
        .bind(entity_id)
        .bind(area_id)
        .bind("Test Merchant")
        .bind("MERCH001")
        .bind(serde_json::json!({"category": "retail"}))
        .bind("POINT(5 5)")
        .bind("point")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to create merchant");

        // Create beacon linked to merchant
        let beacon_id: i64 = sqlx::query_scalar(
            "INSERT INTO beacons (entity, area, merchant, name, type, location, device, mac, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, ST_GeomFromText($6, 4326), $7, $8, $9, $10) RETURNING id"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind(merchant_id)
        .bind("Merchant Beacon")
        .bind("marketing")
        .bind("POINT(5 5)")
        .bind("esp32c3")
        .bind("AA:BB:CC:DD:EE:FF")
        .bind(now)
        .bind(now)
        .fetch_one(&pool)
        .await
        .expect("Failed to create beacon");

        // Delete merchant
        sqlx::query("DELETE FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .expect("Failed to delete merchant");

        // Verify beacon still exists but merchant field is NULL
        let merchant_ref: Option<Uuid> =
            sqlx::query_scalar("SELECT merchant FROM beacons WHERE id = $1")
                .bind(beacon_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to query beacon");

        assert!(merchant_ref.is_none(), "Merchant reference should be NULL");
    }

    #[tokio::test]
    async fn test_foreign_key_violation_beacon_invalid_area() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let now = chrono::Utc::now().timestamp_millis();

        // Try to create beacon with non-existent area (should fail)
        let result = sqlx::query(
            "INSERT INTO beacons (entity, area, name, type, location, device, mac, created_at, updated_at)
             VALUES ($1, $2, $3, $4, ST_GeomFromText($5, 4326), $6, $7, $8, $9)"
        )
        .bind(entity_id)
        .bind(999999i64)  // Non-existent area
        .bind("Invalid Beacon")
        .bind("navigation")
        .bind("POINT(5 5)")
        .bind("esp32c3")
        .bind("AA:BB:CC:DD:EE:FF")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "Should fail due to foreign key constraint");
    }

    #[tokio::test]
    async fn test_beacon_secrets_cascade_delete() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area");
        let now = chrono::Utc::now().timestamp_millis();

        // Create beacon
        let beacon_id: i64 = sqlx::query_scalar(
            "INSERT INTO beacons (entity, area, name, type, location, device, mac, created_at, updated_at)
             VALUES ($1, $2, $3, $4, ST_GeomFromText($5, 4326), $6, $7, $8, $9) RETURNING id"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Secure Beacon")
        .bind("security")
        .bind("POINT(5 5)")
        .bind("esp32c3")
        .bind("AA:BB:CC:DD:EE:FF")
        .bind(now)
        .bind(now)
        .fetch_one(&pool)
        .await
        .expect("Failed to create beacon");

        // Create beacon secret
        sqlx::query(
            "INSERT INTO beacon_secrets (beacon_id, private_key, public_key, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(beacon_id)
        .bind(vec![1u8; 32])
        .bind(vec![2u8; 64])
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to create beacon secret");

        // Delete beacon
        sqlx::query("DELETE FROM beacons WHERE id = $1")
            .bind(beacon_id)
            .execute(&pool)
            .await
            .expect("Failed to delete beacon");

        // Verify beacon secret is deleted
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM beacon_secrets WHERE beacon_id = $1",
        )
        .bind(beacon_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count secrets");

        assert_eq!(count, 0, "Beacon secrets should be cascade deleted");
    }
}

#[cfg(test)]
mod user_tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_user_password() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let now = chrono::Utc::now().timestamp_millis();

        let result = sqlx::query(
            "INSERT INTO users (username, email, password_hash, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind("testuser")
        .bind("test@example.com")
        .bind("$2b$12$hashed_password")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Failed to insert user");
    }

    #[tokio::test]
    async fn test_insert_user_oauth_github() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let now = chrono::Utc::now().timestamp_millis();

        let result = sqlx::query(
            "INSERT INTO users (username, email, github_id, avatar_url, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind("githubuser")
        .bind("github@example.com")
        .bind("12345")
        .bind("https://github.com/avatar.png")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Failed to insert GitHub user");
    }

    #[tokio::test]
    async fn test_user_unique_constraints() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let now = chrono::Utc::now().timestamp_millis();

        // Insert first user
        sqlx::query(
            "INSERT INTO users (username, email, created_at, updated_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind("unique_user")
        .bind("unique@example.com")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert first user");

        // Try duplicate username (should fail)
        let result = sqlx::query(
            "INSERT INTO users (username, email, created_at, updated_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind("unique_user")
        .bind("another@example.com")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "Should fail due to duplicate username");

        // Try duplicate email (should fail)
        let result = sqlx::query(
            "INSERT INTO users (username, email, created_at, updated_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind("another_user")
        .bind("unique@example.com")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "Should fail due to duplicate email");
    }

    #[tokio::test]
    async fn test_user_with_public_key() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let now = chrono::Utc::now().timestamp_millis();
        let public_key = vec![1u8; 65]; // P-256 public key

        sqlx::query(
            "INSERT INTO users (username, public_key, created_at, updated_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind("crypto_user")
        .bind(&public_key)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert user with public key");

        // Retrieve and verify
        let retrieved_key: Vec<u8> =
            sqlx::query_scalar("SELECT public_key FROM users WHERE username = $1")
                .bind("crypto_user")
                .fetch_one(&pool)
                .await
                .expect("Failed to retrieve public key");

        assert_eq!(retrieved_key, public_key);
    }

    #[tokio::test]
    async fn test_unlock_instances() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = create_test_entity(&pool)
            .await
            .expect("Failed to create entity");
        let area_id = create_test_area(&pool, entity_id)
            .await
            .expect("Failed to create area");
        let now = chrono::Utc::now().timestamp_millis();

        // Create user
        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, username, created_at, updated_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(user_id)
        .bind("unlock_user")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to create user");

        // Create beacon
        let beacon_id: i64 = sqlx::query_scalar(
            "INSERT INTO beacons (entity, area, name, type, location, device, mac, created_at, updated_at)
             VALUES ($1, $2, $3, $4, ST_GeomFromText($5, 4326), $6, $7, $8, $9) RETURNING id"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Door Beacon")
        .bind("security")
        .bind("POINT(5 5)")
        .bind("esp32c3")
        .bind("AA:BB:CC:DD:EE:FF")
        .bind(now)
        .bind(now)
        .fetch_one(&pool)
        .await
        .expect("Failed to create beacon");

        // Create unlock instance
        let result = sqlx::query(
            "INSERT INTO unlock_instances (beacon_id, user_id, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(beacon_id)
        .bind(user_id)
        .bind("success")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Failed to create unlock instance");

        // Query unlock history
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM unlock_instances WHERE user_id = $1 AND status = 'success'",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to query unlock history");

        assert_eq!(count, 1);
    }
}

#[cfg(test)]
mod firmware_tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_firmware() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let now = chrono::Utc::now().timestamp_millis();

        let result = sqlx::query(
            "INSERT INTO firmwares (version, device, description, file_path, file_size, checksum, is_stable, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind("1.0.0")
        .bind("esp32c3")
        .bind("Initial firmware release")
        .bind("/firmwares/esp32c3-1.0.0.bin")
        .bind(1024000i64)
        .bind("abcdef1234567890")
        .bind(true)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Failed to insert firmware");
    }

    #[tokio::test]
    async fn test_firmware_unique_constraint() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let now = chrono::Utc::now().timestamp_millis();

        // Insert first firmware
        sqlx::query(
            "INSERT INTO firmwares (version, device, file_path, file_size, checksum, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind("1.0.0")
        .bind("esp32c3")
        .bind("/firmwares/v1.bin")
        .bind(1024i64)
        .bind("hash1")
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert first firmware");

        // Try duplicate version+device (should fail)
        let result = sqlx::query(
            "INSERT INTO firmwares (version, device, file_path, file_size, checksum, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind("1.0.0")
        .bind("esp32c3")
        .bind("/firmwares/v1-dup.bin")
        .bind(1024i64)
        .bind("hash2")
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "Should fail due to unique constraint");
    }

    #[tokio::test]
    async fn test_query_latest_stable_firmware() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let now = chrono::Utc::now().timestamp_millis();

        // Insert multiple firmwares
        for (version, is_stable) in [("1.0.0", false), ("1.1.0", true), ("1.2.0-beta", false)] {
            sqlx::query(
                "INSERT INTO firmwares (version, device, file_path, file_size, checksum, is_stable, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind(version)
            .bind("esp32c3")
            .bind(format!("/firmwares/{}.bin", version))
            .bind(1024i64)
            .bind(format!("hash_{}", version))
            .bind(is_stable)
            .bind(now)
            .execute(&pool)
            .await
            .expect("Failed to insert firmware");
        }

        // Query latest stable firmware
        let latest_version: String = sqlx::query_scalar(
            "SELECT version FROM firmwares WHERE device = $1 AND is_stable = true ORDER BY created_at DESC LIMIT 1"
        )
        .bind("esp32c3")
        .fetch_one(&pool)
        .await
        .expect("Failed to query latest stable firmware");

        assert_eq!(latest_version, "1.1.0");
    }
}
