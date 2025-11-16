use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use navign_shared::schema::{Entity, EntityType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ServerError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    offset: i64,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize)]
pub struct CreateEntityRequest {
    pub r#type: EntityType,
    pub name: String,
    pub description: Option<String>,
    pub point_min: (f64, f64),
    pub point_max: (f64, f64),
    pub altitude_min: Option<f64>,
    pub altitude_max: Option<f64>,
    pub nation: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub tags: Vec<String>,
}

/// List all entities with pagination
pub async fn list_entities(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Entity>>, ServerError> {
    let entities = sqlx::query_as::<_, Entity>(
        r#"
        SELECT id, type, name, description, point_min, point_max,
               altitude_min, altitude_max, nation, region, city, tags,
               created_at, updated_at
        FROM entities
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(query.limit)
    .bind(query.offset)
    .fetch_all(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    Ok(Json(entities))
}

/// Get entity by ID
pub async fn get_entity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Entity>, ServerError> {
    let entity = sqlx::query_as::<_, Entity>(
        r#"
        SELECT id, type, name, description, point_min, point_max,
               altitude_min, altitude_max, nation, region, city, tags,
               created_at, updated_at
        FROM entities
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?
    .ok_or_else(|| ServerError::NotFound(format!("Entity {} not found", id)))?;

    Ok(Json(entity))
}

/// Create new entity
pub async fn create_entity(
    State(state): State<AppState>,
    Json(req): Json<CreateEntityRequest>,
) -> Result<(StatusCode, Json<Entity>), ServerError> {
    let entity = sqlx::query_as::<_, Entity>(
        r#"
        INSERT INTO entities (type, name, description, point_min, point_max,
                              altitude_min, altitude_max, nation, region, city, tags)
        VALUES ($1, $2, $3, ST_SetSRID(ST_MakePoint($4, $5), 4326),
                ST_SetSRID(ST_MakePoint($6, $7), 4326), $8, $9, $10, $11, $12, $13)
        RETURNING id, type, name, description, point_min, point_max,
                  altitude_min, altitude_max, nation, region, city, tags,
                  created_at, updated_at
        "#,
    )
    .bind(req.r#type.to_string())
    .bind(&req.name)
    .bind(&req.description)
    .bind(req.point_min.0)
    .bind(req.point_min.1)
    .bind(req.point_max.0)
    .bind(req.point_max.1)
    .bind(req.altitude_min)
    .bind(req.altitude_max)
    .bind(&req.nation)
    .bind(&req.region)
    .bind(&req.city)
    .bind(&req.tags)
    .fetch_one(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(entity)))
}

/// Update entity
pub async fn update_entity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateEntityRequest>,
) -> Result<Json<Entity>, ServerError> {
    let entity = sqlx::query_as::<_, Entity>(
        r#"
        UPDATE entities
        SET type = $2, name = $3, description = $4,
            point_min = ST_SetSRID(ST_MakePoint($5, $6), 4326),
            point_max = ST_SetSRID(ST_MakePoint($7, $8), 4326),
            altitude_min = $9, altitude_max = $10,
            nation = $11, region = $12, city = $13, tags = $14
        WHERE id = $1
        RETURNING id, type, name, description, point_min, point_max,
                  altitude_min, altitude_max, nation, region, city, tags,
                  created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(req.r#type.to_string())
    .bind(&req.name)
    .bind(&req.description)
    .bind(req.point_min.0)
    .bind(req.point_min.1)
    .bind(req.point_max.0)
    .bind(req.point_max.1)
    .bind(req.altitude_min)
    .bind(req.altitude_max)
    .bind(&req.nation)
    .bind(&req.region)
    .bind(&req.city)
    .bind(&req.tags)
    .fetch_optional(&state.pg_pool.pool)
    .await
    .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?
    .ok_or_else(|| ServerError::NotFound(format!("Entity {} not found", id)))?;

    Ok(Json(entity))
}

/// Delete entity
pub async fn delete_entity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ServerError> {
    let result = sqlx::query("DELETE FROM entities WHERE id = $1")
        .bind(id)
        .execute(&state.pg_pool.pool)
        .await
        .map_err(|e| ServerError::DatabaseQuery(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerError::NotFound(format!("Entity {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
