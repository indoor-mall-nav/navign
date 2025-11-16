use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use navign_shared::schema::{Merchant, MerchantStyle, MerchantType, SocialMedia};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ServerError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateMerchantRequest {
    pub name: String,
    pub description: Option<String>,
    pub chain: Option<String>,
    pub beacon_code: String,
    pub area_id: i32,
    pub r#type: MerchantType,
    pub color: Option<String>,
    pub tags: Vec<String>,
    pub location: (f64, f64),
    pub style: MerchantStyle,
    pub polygon: Vec<(f64, f64)>,
    pub available_period: Option<Vec<(i64, i64)>>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub social_media: Option<Vec<SocialMedia>>,
}

/// List all merchants for an entity
pub async fn list_merchants(
    State(state): State<AppState>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<Merchant>>, ServerError> {
    let merchants = sqlx::query_as::<_, Merchant>(
        r#"
        SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
               color, tags, location, style, polygon, available_period, opening_hours,
               email, phone, website, social_media, created_at, updated_at
        FROM merchants
        WHERE entity_id = $1
        ORDER BY name ASC
        "#,
    )
    .bind(entity_id)
    .fetch_all(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    Ok(Json(merchants))
}

/// Get merchant by ID
pub async fn get_merchant(
    State(state): State<AppState>,
    Path((entity_id, id)): Path<(Uuid, i32)>,
) -> Result<Json<Merchant>, ServerError> {
    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
               color, tags, location, style, polygon, available_period, opening_hours,
               email, phone, website, social_media, created_at, updated_at
        FROM merchants
        WHERE entity_id = $1 AND id = $2
        "#,
    )
    .bind(entity_id)
    .bind(id)
    .fetch_optional(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?
    .ok_or_else(|| ServerError::NotFound(format!("Merchant {} not found", id)))?;

    Ok(Json(merchant))
}

/// Create new merchant
pub async fn create_merchant(
    State(state): State<AppState>,
    Path(entity_id): Path<Uuid>,
    Json(req): Json<CreateMerchantRequest>,
) -> Result<(StatusCode, Json<Merchant>), ServerError> {
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

    // Serialize MerchantType to JSON
    let type_json = serde_json::to_value(&req.r#type)
        .map_err(|e| ServerError::SerializationError(e.to_string()))?;

    // Serialize MerchantStyle to string
    let style_str = match req.style {
        MerchantStyle::Store => "store",
        MerchantStyle::Kiosk => "kiosk",
        MerchantStyle::PopUp => "pop-up",
        MerchantStyle::FoodTruck => "food-truck",
        MerchantStyle::Room => "room",
    };

    // Serialize social media to JSON if provided
    let social_media_json = req.social_media.as_ref().map(|sm| {
        serde_json::to_value(sm)
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }).transpose()?;

    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        INSERT INTO merchants (entity_id, area_id, name, description, chain, beacon_code,
                               type, color, tags, location, style, polygon, available_period,
                               email, phone, website, social_media)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                ST_SetSRID(ST_MakePoint($10, $11), 4326), $12,
                ST_SetSRID(ST_GeomFromText($13), 4326), $14, $15, $16, $17, $18)
        RETURNING id, entity_id, area_id, name, description, chain, beacon_code, type,
                  color, tags, location, style, polygon, available_period, opening_hours,
                  email, phone, website, social_media, created_at, updated_at
        "#,
    )
    .bind(entity_id)
    .bind(req.area_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.chain)
    .bind(&req.beacon_code)
    .bind(type_json)
    .bind(&req.color)
    .bind(&req.tags)
    .bind(req.location.0)
    .bind(req.location.1)
    .bind(style_str)
    .bind(&polygon_wkt)
    .bind(req.available_period.as_ref().map(|v| serde_json::to_value(v).ok()).flatten())
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.website)
    .bind(social_media_json)
    .fetch_one(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(merchant)))
}

/// Update merchant
pub async fn update_merchant(
    State(state): State<AppState>,
    Path((entity_id, id)): Path<(Uuid, i32)>,
    Json(req): Json<CreateMerchantRequest>,
) -> Result<Json<Merchant>, ServerError> {
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

    // Serialize MerchantType to JSON
    let type_json = serde_json::to_value(&req.r#type)
        .map_err(|e| ServerError::SerializationError(e.to_string()))?;

    // Serialize MerchantStyle to string
    let style_str = match req.style {
        MerchantStyle::Store => "store",
        MerchantStyle::Kiosk => "kiosk",
        MerchantStyle::PopUp => "pop-up",
        MerchantStyle::FoodTruck => "food-truck",
        MerchantStyle::Room => "room",
    };

    // Serialize social media to JSON if provided
    let social_media_json = req.social_media.as_ref().map(|sm| {
        serde_json::to_value(sm)
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }).transpose()?;

    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        UPDATE merchants
        SET area_id = $3, name = $4, description = $5, chain = $6, beacon_code = $7,
            type = $8, color = $9, tags = $10,
            location = ST_SetSRID(ST_MakePoint($11, $12), 4326),
            style = $13, polygon = ST_SetSRID(ST_GeomFromText($14), 4326),
            available_period = $15, email = $16, phone = $17, website = $18,
            social_media = $19
        WHERE entity_id = $1 AND id = $2
        RETURNING id, entity_id, area_id, name, description, chain, beacon_code, type,
                  color, tags, location, style, polygon, available_period, opening_hours,
                  email, phone, website, social_media, created_at, updated_at
        "#,
    )
    .bind(entity_id)
    .bind(id)
    .bind(req.area_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.chain)
    .bind(&req.beacon_code)
    .bind(type_json)
    .bind(&req.color)
    .bind(&req.tags)
    .bind(req.location.0)
    .bind(req.location.1)
    .bind(style_str)
    .bind(&polygon_wkt)
    .bind(req.available_period.as_ref().map(|v| serde_json::to_value(v).ok()).flatten())
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.website)
    .bind(social_media_json)
    .fetch_optional(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?
    .ok_or_else(|| ServerError::NotFound(format!("Merchant {} not found", id)))?;

    Ok(Json(merchant))
}

/// Delete merchant
pub async fn delete_merchant(
    State(state): State<AppState>,
    Path((entity_id, id)): Path<(Uuid, i32)>,
) -> Result<StatusCode, ServerError> {
    let result = sqlx::query("DELETE FROM merchants WHERE entity_id = $1 AND id = $2")
        .bind(entity_id)
        .bind(id)
        .execute(&state.pg_pool.pool)
        .await
        .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerError::NotFound(format!("Merchant {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
