use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use navign_shared::schema::{postgis::PgPolygon, Area};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ServerError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateAreaRequest {
    pub name: String,
    pub description: Option<String>,
    pub floor_type: Option<String>,
    pub floor_name: Option<i32>,
    pub beacon_code: String,
    pub polygon: Vec<(f64, f64)>,
}

/// List all areas for an entity
pub async fn list_areas(
    State(state): State<AppState>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<Area>>, ServerError> {
    let areas = sqlx::query_as::<_, Area>(
        r#"
        SELECT id, entity_id, name, description, beacon_code, floor_type, floor_name,
               polygon, created_at, updated_at
        FROM areas
        WHERE entity_id = $1
        ORDER BY floor_name DESC, name ASC
        "#,
    )
    .bind(entity_id)
    .fetch_all(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    Ok(Json(areas))
}

/// Get area by ID
pub async fn get_area(
    State(state): State<AppState>,
    Path((entity_id, id)): Path<(Uuid, i32)>,
) -> Result<Json<Area>, ServerError> {
    let area = sqlx::query_as::<_, Area>(
        r#"
        SELECT id, entity_id, name, description, beacon_code, floor_type, floor_name,
               polygon, created_at, updated_at
        FROM areas
        WHERE entity_id = $1 AND id = $2
        "#,
    )
    .bind(entity_id)
    .bind(id)
    .fetch_optional(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?
    .ok_or_else(|| ServerError::NotFound(format!("Area {} not found", id)))?;

    Ok(Json(area))
}

/// Create new area
pub async fn create_area(
    State(state): State<AppState>,
    Path(entity_id): Path<Uuid>,
    Json(req): Json<CreateAreaRequest>,
) -> Result<(StatusCode, Json<Area>), ServerError> {
    // Verify entity exists
    let entity_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM entities WHERE id = $1)")
        .bind(entity_id)
        .fetch_one(&state.pg_pool.pool)
        .await
        .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    if !entity_exists {
        return Err(ServerError::EntityNotFound(format!("Entity {} not found", entity_id)));
    }

    // Validate polygon has at least 3 points
    if req.polygon.len() < 3 {
        return Err(ServerError::ValidationError(
            "Polygon must have at least 3 points".to_string(),
        ));
    }

    // Build WKT polygon string from coordinates
    let mut polygon_wkt = String::from("POLYGON((");
    for (i, (x, y)) in req.polygon.iter().enumerate() {
        if i > 0 {
            polygon_wkt.push(',');
        }
        polygon_wkt.push_str(&format!("{} {}", x, y));
    }
    // Close the polygon by repeating the first point
    if let Some((x, y)) = req.polygon.first() {
        polygon_wkt.push_str(&format!(",{} {}", x, y));
    }
    polygon_wkt.push_str("))");

    let area = sqlx::query_as::<_, Area>(
        r#"
        INSERT INTO areas (entity_id, name, description, floor_type, floor_name, beacon_code, polygon)
        VALUES ($1, $2, $3, $4, $5, $6, ST_SetSRID(ST_GeomFromText($7), 4326))
        RETURNING id, entity_id, name, description, beacon_code, floor_type, floor_name,
                  polygon, created_at, updated_at
        "#,
    )
    .bind(entity_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.floor_type)
    .bind(req.floor_name)
    .bind(&req.beacon_code)
    .bind(&polygon_wkt)
    .fetch_one(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(area)))
}

/// Update area
pub async fn update_area(
    State(state): State<AppState>,
    Path((entity_id, id)): Path<(Uuid, i32)>,
    Json(req): Json<CreateAreaRequest>,
) -> Result<Json<Area>, ServerError> {
    // Validate polygon has at least 3 points
    if req.polygon.len() < 3 {
        return Err(ServerError::ValidationError(
            "Polygon must have at least 3 points".to_string(),
        ));
    }

    // Build WKT polygon string from coordinates
    let mut polygon_wkt = String::from("POLYGON((");
    for (i, (x, y)) in req.polygon.iter().enumerate() {
        if i > 0 {
            polygon_wkt.push(',');
        }
        polygon_wkt.push_str(&format!("{} {}", x, y));
    }
    // Close the polygon
    if let Some((x, y)) = req.polygon.first() {
        polygon_wkt.push_str(&format!(",{} {}", x, y));
    }
    polygon_wkt.push_str("))");

    let area = sqlx::query_as::<_, Area>(
        r#"
        UPDATE areas
        SET name = $3, description = $4, floor_type = $5, floor_name = $6,
            beacon_code = $7, polygon = ST_SetSRID(ST_GeomFromText($8), 4326)
        WHERE entity_id = $1 AND id = $2
        RETURNING id, entity_id, name, description, beacon_code, floor_type, floor_name,
                  polygon, created_at, updated_at
        "#,
    )
    .bind(entity_id)
    .bind(id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.floor_type)
    .bind(req.floor_name)
    .bind(&req.beacon_code)
    .bind(&polygon_wkt)
    .fetch_optional(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?
    .ok_or_else(|| ServerError::NotFound(format!("Area {} not found", id)))?;

    Ok(Json(area))
}

/// Delete area
pub async fn delete_area(
    State(state): State<AppState>,
    Path((entity_id, id)): Path<(Uuid, i32)>,
) -> Result<StatusCode, ServerError> {
    let result = sqlx::query("DELETE FROM areas WHERE entity_id = $1 AND id = $2")
        .bind(entity_id)
        .bind(id)
        .execute(&state.pg_pool.pool)
        .await
        .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerError::NotFound(format!("Area {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
