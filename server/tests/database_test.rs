use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

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

#[cfg(test)]
mod database_connection_tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let pool = create_test_pool().await;
        assert!(pool.is_ok(), "Failed to connect to PostgreSQL");
    }

    #[tokio::test]
    async fn test_database_ping() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        let result = sqlx::query("SELECT 1 as ping").fetch_one(&pool).await;
        assert!(result.is_ok(), "Failed to ping database");
    }

    #[tokio::test]
    async fn test_extensions_installed() {
        let pool = create_test_pool().await.expect("Failed to create pool");

        // Check uuid-ossp extension
        let uuid_ext = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'uuid-ossp')",
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check uuid-ossp extension");
        assert!(uuid_ext, "uuid-ossp extension not installed");

        // Check PostGIS extension
        let postgis_ext = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'postgis')",
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check postgis extension");
        assert!(postgis_ext, "PostGIS extension not installed");
    }

    #[tokio::test]
    async fn test_all_tables_exist() {
        let pool = create_test_pool().await.expect("Failed to create pool");

        let tables = vec![
            "entities",
            "areas",
            "beacons",
            "merchants",
            "connections",
            "users",
            "beacon_secrets",
            "firmwares",
            "unlock_instances",
        ];

        for table in tables {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = $1)",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect(&format!("Failed to check table existence for {}", table));

            assert!(exists, "Table {} does not exist", table);
        }
    }

    #[tokio::test]
    async fn test_connection_pool_settings() {
        let pool = create_test_pool().await.expect("Failed to create pool");

        // Test that we can acquire multiple connections
        let conn1 = pool.acquire().await;
        let conn2 = pool.acquire().await;

        assert!(conn1.is_ok(), "Failed to acquire first connection");
        assert!(conn2.is_ok(), "Failed to acquire second connection");
    }
}

#[cfg(test)]
mod entity_crud_tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_insert_entity() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp_millis();

        let result = sqlx::query(
            "INSERT INTO entities (id, type, name, description, longitude_min, longitude_max,
             latitude_min, latitude_max, tags, created_at, updated_at, city)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(entity_id)
        .bind("mall")
        .bind("Test Mall")
        .bind("A test shopping mall")
        .bind(-122.5)
        .bind(-122.4)
        .bind(37.7)
        .bind(37.8)
        .bind(serde_json::json!(["shopping", "retail"]))
        .bind(now)
        .bind(now)
        .bind("San Francisco")
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Failed to insert entity");
        assert_eq!(result.unwrap().rows_affected(), 1);
    }

    #[tokio::test]
    async fn test_select_entity() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        // First insert an entity
        let entity_id = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp_millis();

        sqlx::query(
            "INSERT INTO entities (id, type, name, longitude_min, longitude_max,
             latitude_min, latitude_max, tags, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(entity_id)
        .bind("hospital")
        .bind("Test Hospital")
        .bind(-122.5)
        .bind(-122.4)
        .bind(37.7)
        .bind(37.8)
        .bind(serde_json::json!([]))
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert entity");

        // Now select it
        let row = sqlx::query("SELECT id, type, name FROM entities WHERE id = $1")
            .bind(entity_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to select entity");

        let retrieved_id: Uuid = row.get("id");
        let entity_type: String = row.get("type");
        let name: String = row.get("name");

        assert_eq!(retrieved_id, entity_id);
        assert_eq!(entity_type, "hospital");
        assert_eq!(name, "Test Hospital");
    }

    #[tokio::test]
    async fn test_update_entity() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp_millis();

        // Insert
        sqlx::query(
            "INSERT INTO entities (id, type, name, longitude_min, longitude_max,
             latitude_min, latitude_max, tags, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(entity_id)
        .bind("school")
        .bind("Old Name")
        .bind(-122.5)
        .bind(-122.4)
        .bind(37.7)
        .bind(37.8)
        .bind(serde_json::json!([]))
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert");

        // Update
        let result = sqlx::query("UPDATE entities SET name = $1, updated_at = $2 WHERE id = $3")
            .bind("New Name")
            .bind(chrono::Utc::now().timestamp_millis())
            .bind(entity_id)
            .execute(&pool)
            .await
            .expect("Failed to update");

        assert_eq!(result.rows_affected(), 1);

        // Verify update
        let row = sqlx::query("SELECT name FROM entities WHERE id = $1")
            .bind(entity_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to select");

        let name: String = row.get("name");
        assert_eq!(name, "New Name");
    }

    #[tokio::test]
    async fn test_delete_entity() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp_millis();

        // Insert
        sqlx::query(
            "INSERT INTO entities (id, type, name, longitude_min, longitude_max,
             latitude_min, latitude_max, tags, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(entity_id)
        .bind("transportation")
        .bind("Test Airport")
        .bind(-122.5)
        .bind(-122.4)
        .bind(37.7)
        .bind(37.8)
        .bind(serde_json::json!([]))
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert");

        // Delete
        let result = sqlx::query("DELETE FROM entities WHERE id = $1")
            .bind(entity_id)
            .execute(&pool)
            .await
            .expect("Failed to delete");

        assert_eq!(result.rows_affected(), 1);

        // Verify deletion
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM entities WHERE id = $1")
            .bind(entity_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count");

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_entity_constraints() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let now = chrono::Utc::now().timestamp_millis();

        // Test invalid entity type (should fail CHECK constraint)
        let result = sqlx::query(
            "INSERT INTO entities (type, name, longitude_min, longitude_max,
             latitude_min, latitude_max, tags, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind("invalid_type")
        .bind("Test")
        .bind(-122.5)
        .bind(-122.4)
        .bind(37.7)
        .bind(37.8)
        .bind(serde_json::json!([]))
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "Should fail with invalid entity type");

        // Test invalid longitude range (should fail CHECK constraint)
        let result = sqlx::query(
            "INSERT INTO entities (type, name, longitude_min, longitude_max,
             latitude_min, latitude_max, tags, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind("mall")
        .bind("Test")
        .bind(-122.4)
        .bind(-122.5) // min > max
        .bind(37.7)
        .bind(37.8)
        .bind(serde_json::json!([]))
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "Should fail with invalid longitude range");
    }

    #[tokio::test]
    async fn test_entity_tags_jsonb() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        let entity_id = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp_millis();
        let tags = serde_json::json!(["shopping", "food", "entertainment"]);

        // Insert with tags
        sqlx::query(
            "INSERT INTO entities (id, type, name, longitude_min, longitude_max,
             latitude_min, latitude_max, tags, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(entity_id)
        .bind("mall")
        .bind("Tagged Mall")
        .bind(-122.5)
        .bind(-122.4)
        .bind(37.7)
        .bind(37.8)
        .bind(&tags)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert");

        // Query tags
        let row = sqlx::query("SELECT tags FROM entities WHERE id = $1")
            .bind(entity_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to select");

        let retrieved_tags: serde_json::Value = row.get("tags");
        assert_eq!(retrieved_tags, tags);

        // Test JSONB query (contains operator)
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM entities WHERE tags @> $1")
            .bind(serde_json::json!(["shopping"]))
            .fetch_one(&pool)
            .await
            .expect("Failed to query jsonb");

        assert_eq!(count, 1);
    }
}

#[cfg(test)]
mod postgis_geometry_tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_insert_area_with_polygon() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        // First create an entity
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
        .execute(&pool)
        .await
        .expect("Failed to insert entity");

        // Insert area with polygon using WKT
        let result = sqlx::query(
            "INSERT INTO areas (entity, name, beacon_code, polygon, created_at, updated_at)
             VALUES ($1, $2, $3, ST_GeomFromText($4, 4326), $5, $6)",
        )
        .bind(entity_id)
        .bind("Test Area")
        .bind("AREA001")
        .bind("POLYGON((0 0, 0 10, 10 10, 10 0, 0 0))")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Failed to insert area with polygon");
        assert_eq!(result.unwrap().rows_affected(), 1);
    }

    #[tokio::test]
    async fn test_insert_beacon_with_point() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        // Create entity
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
        .execute(&pool)
        .await
        .expect("Failed to insert entity");

        // Create area
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
        .fetch_one(&pool)
        .await
        .expect("Failed to insert area");

        // Insert beacon with point location
        let result = sqlx::query(
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
        .await;

        assert!(result.is_ok(), "Failed to insert beacon with point");
        assert_eq!(result.unwrap().rows_affected(), 1);
    }

    #[tokio::test]
    async fn test_postgis_spatial_query() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        // Create entity and area
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
        .execute(&pool)
        .await
        .expect("Failed to insert entity");

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
        .fetch_one(&pool)
        .await
        .expect("Failed to insert area");

        // Test point-in-polygon query
        let is_inside: bool = sqlx::query_scalar(
            "SELECT ST_Contains(polygon, ST_GeomFromText($1, 4326))
             FROM areas WHERE id = $2",
        )
        .bind("POINT(5 5)") // Inside the polygon
        .bind(area_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to query point-in-polygon");

        assert!(is_inside, "Point should be inside polygon");

        // Test point outside
        let is_outside: bool = sqlx::query_scalar(
            "SELECT ST_Contains(polygon, ST_GeomFromText($1, 4326))
             FROM areas WHERE id = $2",
        )
        .bind("POINT(15 15)") // Outside the polygon
        .bind(area_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to query point-in-polygon");

        assert!(!is_outside, "Point should be outside polygon");
    }

    #[tokio::test]
    async fn test_postgis_distance_query() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

        // Create entity and area
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
        .execute(&pool)
        .await
        .expect("Failed to insert entity");

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
        .fetch_one(&pool)
        .await
        .expect("Failed to insert area");

        // Insert two beacons
        sqlx::query(
            "INSERT INTO beacons (entity, area, name, type, location, device, mac, created_at, updated_at)
             VALUES ($1, $2, $3, $4, ST_GeomFromText($5, 4326), $6, $7, $8, $9)"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Beacon 1")
        .bind("navigation")
        .bind("POINT(0 0)")
        .bind("esp32c3")
        .bind("AA:BB:CC:DD:EE:F1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert beacon 1");

        sqlx::query(
            "INSERT INTO beacons (entity, area, name, type, location, device, mac, created_at, updated_at)
             VALUES ($1, $2, $3, $4, ST_GeomFromText($5, 4326), $6, $7, $8, $9)"
        )
        .bind(entity_id)
        .bind(area_id)
        .bind("Beacon 2")
        .bind("navigation")
        .bind("POINT(3 4)")  // Distance = 5 from origin
        .bind("esp32c3")
        .bind("AA:BB:CC:DD:EE:F2")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert beacon 2");

        // Find nearest beacon to a point
        let nearest_name: String = sqlx::query_scalar(
            "SELECT name FROM beacons
             WHERE entity = $1
             ORDER BY ST_Distance(location, ST_GeomFromText($2, 4326))
             LIMIT 1",
        )
        .bind(entity_id)
        .bind("POINT(1 1)")
        .fetch_one(&pool)
        .await
        .expect("Failed to find nearest beacon");

        assert_eq!(nearest_name, "Beacon 1");
    }

    #[tokio::test]
    async fn test_postgis_area_calculation() {
        let pool = create_test_pool().await.expect("Failed to create pool");
        cleanup_database(&pool).await.expect("Failed to cleanup");

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
        .execute(&pool)
        .await
        .expect("Failed to insert entity");

        let area_id: i64 = sqlx::query_scalar(
            "INSERT INTO areas (entity, name, beacon_code, polygon, created_at, updated_at)
             VALUES ($1, $2, $3, ST_GeomFromText($4, 4326), $5, $6) RETURNING id",
        )
        .bind(entity_id)
        .bind("Square Area")
        .bind("AREA001")
        .bind("POLYGON((0 0, 0 10, 10 10, 10 0, 0 0))") // 10x10 square
        .bind(now)
        .bind(now)
        .fetch_one(&pool)
        .await
        .expect("Failed to insert area");

        // Calculate area (in degrees, not real-world units)
        let area: f64 = sqlx::query_scalar("SELECT ST_Area(polygon) FROM areas WHERE id = $1")
            .bind(area_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to calculate area");

        assert!(
            (area - 100.0).abs() < 0.01,
            "Area should be approximately 100"
        );
    }
}
