use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use navign_shared::schema::{Beacon, BeaconDevice, BeaconType};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ServerError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateBeaconRequest {
    pub area_id: i32,
    pub merchant_id: Option<i32>,
    pub connection_id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub r#type: BeaconType,
    pub location: (f64, f64),
    pub device: BeaconDevice,
    pub mac: String,
}

/// List all beacons for an entity
pub async fn list_beacons(
    State(state): State<AppState>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<Beacon>>, ServerError> {
    let beacons = sqlx::query_as::<_, Beacon>(
        r#"
        SELECT id, entity_id, area_id, merchant_id, connection_id, name, description,
               type, location, device, mac, created_at, updated_at
        FROM beacons
        WHERE entity_id = $1
        ORDER BY name ASC
        "#,
    )
    .bind(entity_id)
    .fetch_all(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    Ok(Json(beacons))
}

/// Get beacon by ID
pub async fn get_beacon(
    State(state): State<AppState>,
    Path((entity_id, id)): Path<(Uuid, i32)>,
) -> Result<Json<Beacon>, ServerError> {
    let beacon = sqlx::query_as::<_, Beacon>(
        r#"
        SELECT id, entity_id, area_id, merchant_id, connection_id, name, description,
               type, location, device, mac, created_at, updated_at
        FROM beacons
        WHERE entity_id = $1 AND id = $2
        "#,
    )
    .bind(entity_id)
    .bind(id)
    .fetch_optional(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?
    .ok_or_else(|| ServerError::NotFound(format!("Beacon {} not found", id)))?;

    Ok(Json(beacon))
}

/// Create new beacon
pub async fn create_beacon(
    State(state): State<AppState>,
    Path(entity_id): Path<Uuid>,
    Json(req): Json<CreateBeaconRequest>,
) -> Result<(StatusCode, Json<Beacon>), ServerError> {
    // Verify entity exists
    let entity_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM entities WHERE id = $1)")
        .bind(entity_id)
        .fetch_one(&state.pg_pool.pool)
        .await
        .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    if !entity_exists {
        return Err(ServerError::EntityNotFound(format!("Entity {} not found", entity_id)));
    }

    // Verify area exists and belongs to entity
    let area_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM areas WHERE id = $1 AND entity_id = $2)"
    )
    .bind(req.area_id)
    .bind(entity_id)
    .fetch_one(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    if !area_exists {
        return Err(ServerError::AreaNotFound(format!("Area {} not found in entity {}", req.area_id, entity_id)));
    }

    // Verify merchant if provided
    if let Some(merchant_id) = req.merchant_id {
        let merchant_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM merchants WHERE id = $1 AND entity_id = $2)"
        )
        .bind(merchant_id)
        .bind(entity_id)
        .fetch_one(&state.pg_pool.pool)
        .await
        .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

        if !merchant_exists {
            return Err(ServerError::NotFound(format!("Merchant {} not found", merchant_id)));
        }
    }

    // Verify connection if provided
    if let Some(connection_id) = req.connection_id {
        let connection_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM connections WHERE id = $1 AND entity_id = $2)"
        )
        .bind(connection_id)
        .bind(entity_id)
        .fetch_one(&state.pg_pool.pool)
        .await
        .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

        if !connection_exists {
            return Err(ServerError::ConnectionNotFound(format!("Connection {} not found", connection_id)));
        }
    }

    // Convert BeaconType and BeaconDevice to strings for storage
    let type_str = match req.r#type {
        BeaconType::Navigation => "navigation",
        BeaconType::Marketing => "marketing",
        BeaconType::Tracking => "tracking",
        BeaconType::Environmental => "environmental",
        BeaconType::Security => "security",
        BeaconType::Other => "other",
    };

    let device_str = match req.device {
        BeaconDevice::Esp32 => "esp32",
        BeaconDevice::Esp32C3 => "esp32c3",
        BeaconDevice::Esp32C5 => "esp32c5",
        BeaconDevice::Esp32C6 => "esp32c6",
        BeaconDevice::Esp32S3 => "esp32s3",
    };

    let beacon = sqlx::query_as::<_, Beacon>(
        r#"
        INSERT INTO beacons (entity_id, area_id, merchant_id, connection_id, name, description,
                             type, location, device, mac)
        VALUES ($1, $2, $3, $4, $5, $6, $7, ST_SetSRID(ST_MakePoint($8, $9), 4326), $10, $11)
        RETURNING id, entity_id, area_id, merchant_id, connection_id, name, description,
                  type, location, device, mac, created_at, updated_at
        "#,
    )
    .bind(entity_id)
    .bind(req.area_id)
    .bind(req.merchant_id)
    .bind(req.connection_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(type_str)
    .bind(req.location.0)
    .bind(req.location.1)
    .bind(device_str)
    .bind(&req.mac)
    .fetch_one(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(beacon)))
}

/// Update beacon
pub async fn update_beacon(
    State(state): State<AppState>,
    Path((entity_id, id)): Path<(Uuid, i32)>,
    Json(req): Json<CreateBeaconRequest>,
) -> Result<Json<Beacon>, ServerError> {
    // Verify area exists and belongs to entity
    let area_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM areas WHERE id = $1 AND entity_id = $2)"
    )
    .bind(req.area_id)
    .bind(entity_id)
    .fetch_one(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    if !area_exists {
        return Err(ServerError::AreaNotFound(format!("Area {} not found", req.area_id)));
    }

    // Verify merchant if provided
    if let Some(merchant_id) = req.merchant_id {
        let merchant_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM merchants WHERE id = $1 AND entity_id = $2)"
        )
        .bind(merchant_id)
        .bind(entity_id)
        .fetch_one(&state.pg_pool.pool)
        .await
        .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

        if !merchant_exists {
            return Err(ServerError::NotFound(format!("Merchant {} not found", merchant_id)));
        }
    }

    // Verify connection if provided
    if let Some(connection_id) = req.connection_id {
        let connection_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM connections WHERE id = $1 AND entity_id = $2)"
        )
        .bind(connection_id)
        .bind(entity_id)
        .fetch_one(&state.pg_pool.pool)
        .await
        .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

        if !connection_exists {
            return Err(ServerError::ConnectionNotFound(format!("Connection {} not found", connection_id)));
        }
    }

    // Convert enums to strings
    let type_str = match req.r#type {
        BeaconType::Navigation => "navigation",
        BeaconType::Marketing => "marketing",
        BeaconType::Tracking => "tracking",
        BeaconType::Environmental => "environmental",
        BeaconType::Security => "security",
        BeaconType::Other => "other",
    };

    let device_str = match req.device {
        BeaconDevice::Esp32 => "esp32",
        BeaconDevice::Esp32C3 => "esp32c3",
        BeaconDevice::Esp32C5 => "esp32c5",
        BeaconDevice::Esp32C6 => "esp32c6",
        BeaconDevice::Esp32S3 => "esp32s3",
    };

    let beacon = sqlx::query_as::<_, Beacon>(
        r#"
        UPDATE beacons
        SET area_id = $3, merchant_id = $4, connection_id = $5, name = $6,
            description = $7, type = $8, location = ST_SetSRID(ST_MakePoint($9, $10), 4326),
            device = $11, mac = $12
        WHERE entity_id = $1 AND id = $2
        RETURNING id, entity_id, area_id, merchant_id, connection_id, name, description,
                  type, location, device, mac, created_at, updated_at
        "#,
    )
    .bind(entity_id)
    .bind(id)
    .bind(req.area_id)
    .bind(req.merchant_id)
    .bind(req.connection_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(type_str)
    .bind(req.location.0)
    .bind(req.location.1)
    .bind(device_str)
    .bind(&req.mac)
    .fetch_optional(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?
    .ok_or_else(|| ServerError::NotFound(format!("Beacon {} not found", id)))?;

    Ok(Json(beacon))
}

/// Delete beacon
pub async fn delete_beacon(
    State(state): State<AppState>,
    Path((entity_id, id)): Path<(Uuid, i32)>,
) -> Result<StatusCode, ServerError> {
    let result = sqlx::query("DELETE FROM beacons WHERE entity_id = $1 AND id = $2")
        .bind(entity_id)
        .bind(id)
        .execute(&state.pg_pool.pool)
        .await
        .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerError::NotFound(format!("Beacon {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
