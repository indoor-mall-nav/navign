//! Dual-database API handlers
//!
//! These handlers provide a seamless transition from MongoDB to PostgreSQL
//! by checking which database is available and routing requests accordingly.
//!
//! Priority:
//! 1. PostgreSQL (if pg_pool is Some)
//! 2. MongoDB (fallback)

use crate::error::{Result, ServerError};
use crate::pg::adapters::*;
use crate::pg::repository::*;
use crate::schema::EntityServiceAddons;
use crate::schema::service::Service;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use bson::oid::ObjectId;
use futures::TryStreamExt;
use navign_shared::*;
use serde::Deserialize;
use sqlx::types::Uuid;
use std::str::FromStr;
use tracing::{debug, info};

/// Generic pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default)]
    pub offset: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    20
}

/// Entity search query parameters
#[derive(Debug, Deserialize)]
pub struct EntitySearchQuery {
    pub nation: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub name: Option<String>,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
}

// ============================================================================
// Entity Handlers
// ============================================================================

/// Get all entities with search filters
pub async fn get_entities(
    State(state): State<AppState>,
    Query(query): Query<EntitySearchQuery>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        debug!("Using PostgreSQL for entity search");
        let repo = EntityRepository::new(pg_pool.as_ref().clone());
        let pg_entities = repo
            .search_by_fields(
                query.nation.as_deref(),
                query.region.as_deref(),
                query.city.as_deref(),
                query.name.as_deref(),
                query.longitude,
                query.latitude,
            )
            .await?;

        // Convert PgEntity to Entity using adapters
        let entities: Vec<Entity> = pg_entities.into_iter().map(pg_entity_to_entity).collect();

        Ok(Json(entities))
    } else {
        debug!("Using MongoDB for entity search");
        let entities = Entity::search_entity_by_fields(
            &state.db,
            query.nation,
            query.region,
            query.city,
            query.name,
            query.longitude,
            query.latitude,
        )
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB query failed: {}", e)))?;

        Ok(Json(entities))
    }
}

/// Get entity by ID
pub async fn get_entity_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        debug!("Using PostgreSQL for entity lookup: {}", id);
        let repo = EntityRepository::new(pg_pool.as_ref().clone());

        match repo.get_by_id(&id).await? {
            Some(pg_entity) => {
                let entity = pg_entity_to_entity(pg_entity);
                Ok(Json(entity))
            }
            None => Err(ServerError::NotFound(format!("Entity {} not found", id))),
        }
    } else {
        debug!("Using MongoDB for entity lookup: {}", id);

        let entity = Entity::get_one_by_id(&state.db, &id)
            .await
            .ok_or_else(|| ServerError::NotFound(format!("Entity {} not found", id)))?;

        Ok(Json(entity))
    }
}

/// Create entity
pub async fn create_entity(
    State(state): State<AppState>,
    Json(entity): Json<Entity>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Creating entity in PostgreSQL: {}", entity.name);
        let repo = EntityRepository::new(pg_pool.as_ref().clone());

        let pg_entity = entity_to_pg_entity(entity.clone());
        let _id = repo.create(&pg_entity).await?;

        info!("Entity created in PostgreSQL");
        Ok((StatusCode::CREATED, Json(entity)))
    } else {
        info!("Creating entity in MongoDB: {}", entity.name);
        entity
            .create(&state.db)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB insert failed: {}", e)))?;

        Ok((StatusCode::CREATED, Json(entity)))
    }
}

/// Update entity
pub async fn update_entity(
    State(state): State<AppState>,
    Json(entity): Json<Entity>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Updating entity in PostgreSQL: {}", entity.name);
        let repo = EntityRepository::new(pg_pool.as_ref().clone());

        let pg_entity = entity_to_pg_entity(entity);
        repo.update(&pg_entity).await?;

        Ok(StatusCode::OK)
    } else {
        info!("Updating entity in MongoDB: {}", entity.name);
        entity
            .update(&state.db)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB update failed: {}", e)))?;

        Ok(StatusCode::OK)
    }
}

/// Delete entity
pub async fn delete_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Deleting entity from PostgreSQL: {}", id);
        let repo = EntityRepository::new(pg_pool.as_ref().clone());
        repo.delete(&id).await?;

        Ok(StatusCode::NO_CONTENT)
    } else {
        info!("Deleting entity from MongoDB: {}", id);

        Entity::delete_by_id(&state.db, &id)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB delete failed: {}", e)))?;

        Ok(StatusCode::NO_CONTENT)
    }
}

// ============================================================================
// Area Handlers
// ============================================================================

/// Get areas by entity
pub async fn get_areas_by_entity(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        debug!("Using PostgreSQL for areas lookup");
        let repo = AreaRepository::new(pg_pool.as_ref().clone());
        let uuid = Uuid::parse_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid UUID".to_string()))?;

        let pg_areas = repo
            .get_by_entity(uuid, pagination.offset, pagination.limit)
            .await?;

        let areas: Vec<Area> = pg_areas.into_iter().map(pg_area_to_area).collect();
        Ok(Json(areas))
    } else {
        debug!("Using MongoDB for areas lookup");
        let oid = ObjectId::from_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid ObjectId".to_string()))?;

        let collection = state.db.collection::<Area>("areas");
        let cursor = collection
            .find(bson::doc! { "entity": oid })
            .limit(pagination.limit)
            .skip(pagination.offset as u64)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB query failed: {}", e)))?;

        let areas: Vec<Area> = cursor
            .try_collect()
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB cursor failed: {}", e)))?;

        Ok(Json(areas))
    }
}

/// Get area by ID
pub async fn get_area_by_id(
    State(state): State<AppState>,
    Path((_entity_id, area_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        debug!("Using PostgreSQL for area lookup: {}", area_id);
        let repo = AreaRepository::new(pg_pool.as_ref().clone());

        match repo.get_by_id(&area_id).await? {
            Some(pg_area) => {
                let area = pg_area_to_area(pg_area);
                Ok(Json(area))
            }
            None => Err(ServerError::NotFound(format!("Area {} not found", area_id))),
        }
    } else {
        debug!("Using MongoDB for area lookup: {}", area_id);

        let area = Area::get_one_by_id(&state.db, &area_id)
            .await
            .ok_or_else(|| ServerError::NotFound(format!("Area {} not found", area_id)))?;

        Ok(Json(area))
    }
}

// ============================================================================
// Beacon Handlers
// ============================================================================

/// Get beacons by entity
pub async fn get_beacons_by_entity(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        debug!("Using PostgreSQL for beacons lookup");
        let repo = BeaconRepository::new(pg_pool.as_ref().clone());
        let uuid = Uuid::parse_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid UUID".to_string()))?;

        let pg_beacons = repo
            .get_by_entity(uuid, pagination.offset, pagination.limit)
            .await?;

        let beacons: Vec<Beacon> = pg_beacons.into_iter().map(pg_beacon_to_beacon).collect();
        Ok(Json(beacons))
    } else {
        debug!("Using MongoDB for beacons lookup");
        let oid = ObjectId::from_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid ObjectId".to_string()))?;

        let collection = state.db.collection::<Beacon>("beacons");
        let cursor = collection
            .find(bson::doc! { "entity": oid })
            .limit(pagination.limit)
            .skip(pagination.offset as u64)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB query failed: {}", e)))?;

        let beacons: Vec<Beacon> = cursor
            .try_collect()
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB cursor failed: {}", e)))?;

        Ok(Json(beacons))
    }
}
