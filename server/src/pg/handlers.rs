//! Dual-database API handlers
//!
//! These handlers provide a seamless transition from MongoDB to PostgreSQL
//! by checking which database is available and routing requests accordingly.
//!
//! Priority:
//! 1. PostgreSQL (if pg_pool is Some)
//! 2. MongoDB (fallback)

use crate::AppState;
use crate::error::{Result, ServerError};
use crate::pg::models::*;
use crate::pg::repository::*;
use crate::schema::service::Service;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use bson::oid::ObjectId;
use futures::TryStreamExt;
use navign_shared::*;
use serde::{Deserialize, Serialize};
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
        let entities = repo
            .search_by_fields(
                query.nation.as_deref(),
                query.region.as_deref(),
                query.city.as_deref(),
                query.name.as_deref(),
                query.longitude,
                query.latitude,
            )
            .await?;

        // Convert PgEntity to Entity for response
        let response: Vec<Entity> = entities
            .into_iter()
            .map(|pg_entity| pg_entity.into())
            .collect();

        Ok(Json(response))
    } else {
        debug!("Using MongoDB for entity search");
        // Fallback to MongoDB
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
                let entity: Entity = pg_entity.into();
                Ok(Json(entity))
            }
            None => Err(ServerError::NotFound(format!("Entity {} not found", id))),
        }
    } else {
        debug!("Using MongoDB for entity lookup: {}", id);
        let oid = ObjectId::from_str(&id)
            .map_err(|_| ServerError::InvalidInput("Invalid ObjectId".to_string()))?;

        let entity = Entity::get_by_id(&state.db, &oid)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB query failed: {}", e)))?
            .ok_or_else(|| ServerError::NotFound(format!("Entity {} not found", id)))?;

        Ok(Json(entity))
    }
}

/// Create entity
pub async fn create_entity(
    State(state): State<AppState>,
    Json(mut entity): Json<Entity>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Creating entity in PostgreSQL: {}", entity.name);
        let repo = EntityRepository::new(pg_pool.as_ref().clone());

        let pg_entity: PgEntity = entity.clone().into();
        let id = repo.create(&pg_entity).await?;

        info!("Entity created with UUID: {}", id);
        entity.id = ObjectId::new(); // Placeholder, will be replaced by frontend
        Ok((StatusCode::CREATED, Json(entity)))
    } else {
        info!("Creating entity in MongoDB: {}", entity.name);
        Entity::create(&state.db, &mut entity)
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

        let pg_entity: PgEntity = entity.into();
        repo.update(&pg_entity).await?;

        Ok(StatusCode::OK)
    } else {
        info!("Updating entity in MongoDB: {}", entity.name);
        Entity::update(&state.db, &entity)
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
        let oid = ObjectId::from_str(&id)
            .map_err(|_| ServerError::InvalidInput("Invalid ObjectId".to_string()))?;

        Entity::delete(&state.db, &oid)
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

        let areas = repo
            .get_by_entity(uuid, pagination.offset, pagination.limit)
            .await?;

        let response: Vec<Area> = areas.into_iter().map(|pg_area| pg_area.into()).collect();
        Ok(Json(response))
    } else {
        debug!("Using MongoDB for areas lookup");
        let oid = ObjectId::from_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid ObjectId".to_string()))?;

        let collection = state.db.collection::<Area>("areas");
        let cursor = collection
            .find(bson::doc! { "entity_id": oid })
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
                let area: Area = pg_area.into();
                Ok(Json(area))
            }
            None => Err(ServerError::NotFound(format!("Area {} not found", area_id))),
        }
    } else {
        debug!("Using MongoDB for area lookup: {}", area_id);
        let oid = ObjectId::from_str(&area_id)
            .map_err(|_| ServerError::InvalidInput("Invalid ObjectId".to_string()))?;

        let area = Area::get_by_id(&state.db, &oid)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB query failed: {}", e)))?
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

        let beacons = repo
            .get_by_entity(uuid, pagination.offset, pagination.limit)
            .await?;

        let response: Vec<Beacon> = beacons
            .into_iter()
            .map(|pg_beacon| pg_beacon.into())
            .collect();
        Ok(Json(response))
    } else {
        debug!("Using MongoDB for beacons lookup");
        let oid = ObjectId::from_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid ObjectId".to_string()))?;

        let collection = state.db.collection::<Beacon>("beacons");
        let cursor = collection
            .find(bson::doc! { "entity_id": oid })
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

// ============================================================================
// Conversion utilities (PgEntity <-> Entity, etc.)
// ============================================================================

// These conversions should ideally be in the shared crate,
// but we'll implement them here for now

impl From<PgEntity> for Entity {
    fn from(pg: PgEntity) -> Self {
        Entity {
            id: ObjectId::new(), // Placeholder - frontend should use UUID
            r#type: pg.r#type.parse().unwrap_or(EntityType::Mall),
            name: pg.name,
            description: pg.description,
            nation: pg.nation,
            region: pg.region,
            city: pg.city,
            address: pg.address,
            longitude_range: (pg.longitude_min, pg.longitude_max),
            latitude_range: (pg.latitude_min, pg.latitude_max),
            floors: serde_json::from_value(pg.floors.0).unwrap_or_default(),
        }
    }
}

impl From<Entity> for PgEntity {
    fn from(entity: Entity) -> Self {
        PgEntity {
            id: Uuid::new_v4(), // Will be generated by DB
            r#type: entity.r#type.to_string(),
            name: entity.name,
            description: entity.description,
            nation: entity.nation,
            region: entity.region,
            city: entity.city,
            address: entity.address,
            longitude_min: entity.longitude_range.0,
            longitude_max: entity.longitude_range.1,
            latitude_min: entity.latitude_range.0,
            latitude_max: entity.latitude_range.1,
            floors: sqlx::types::Json(serde_json::to_value(&entity.floors).unwrap()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl From<PgArea> for Area {
    fn from(pg: PgArea) -> Self {
        Area {
            id: ObjectId::new(),        // Placeholder
            entity_id: ObjectId::new(), // Placeholder
            name: pg.name,
            description: pg.description,
            floor: Floor {
                name: pg.floor,
                r#type: FloorType::Ground, // Default, should be stored in DB
            },
            beacon_code: pg.beacon_code,
            polygon: serde_json::from_value(pg.polygon.0).unwrap_or_default(),
        }
    }
}

impl From<PgBeacon> for Beacon {
    fn from(pg: PgBeacon) -> Self {
        Beacon {
            id: ObjectId::new(),        // Placeholder
            entity_id: ObjectId::new(), // Placeholder
            area_id: ObjectId::new(),   // Placeholder
            merchant_id: None,
            connection_id: None,
            name: pg.name,
            description: pg.description,
            r#type: pg.r#type.parse().unwrap_or(BeaconType::Merchant),
            device: BeaconDevice {
                device_id: pg.device_id,
                public_key: pg.public_key,
                capabilities: serde_json::from_value(pg.capabilities.0).unwrap_or_default(),
                unlock_method: pg.unlock_method.and_then(|s| s.parse().ok()),
            },
            floor: pg.floor,
            location: (0.0, 0.0), // Need to parse from PostGIS POINT
        }
    }
}
