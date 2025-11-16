//! Dual-database API handlers
//!
//! These handlers provide a seamless transition from MongoDB to PostgreSQL
//! by checking which database is available and routing requests accordingly.
//!
//! Priority:
//! 1. PostgreSQL (if pg_pool is Some)
//! 2. MongoDB (fallback)

#![allow(dead_code)] // Handlers will be integrated into main router

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

/// Get beacon by ID
pub async fn get_beacon_by_id(
    State(state): State<AppState>,
    Path((_entity_id, beacon_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        debug!("Using PostgreSQL for beacon lookup: {}", beacon_id);
        let repo = BeaconRepository::new(pg_pool.as_ref().clone());

        match repo.get_by_id(&beacon_id).await? {
            Some(pg_beacon) => {
                let beacon = pg_beacon_to_beacon(pg_beacon);
                Ok(Json(beacon))
            }
            None => Err(ServerError::NotFound(format!("Beacon {} not found", beacon_id))),
        }
    } else {
        debug!("Using MongoDB for beacon lookup: {}", beacon_id);

        let beacon = Beacon::get_one_by_id(&state.db, &beacon_id)
            .await
            .ok_or_else(|| ServerError::NotFound(format!("Beacon {} not found", beacon_id)))?;

        Ok(Json(beacon))
    }
}

/// Create area
pub async fn create_area(
    State(state): State<AppState>,
    Json(area): Json<Area>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Creating area in PostgreSQL: {}", area.name);
        let repo = AreaRepository::new(pg_pool.as_ref().clone());

        let pg_area = area_to_pg_area(area.clone());
        let _id = repo.create(&pg_area).await?;

        info!("Area created in PostgreSQL");
        Ok((StatusCode::CREATED, Json(area)))
    } else {
        info!("Creating area in MongoDB: {}", area.name);
        area.create(&state.db)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB insert failed: {}", e)))?;

        Ok((StatusCode::CREATED, Json(area)))
    }
}

/// Update area
pub async fn update_area(
    State(state): State<AppState>,
    Json(area): Json<Area>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Updating area in PostgreSQL: {}", area.name);
        let repo = AreaRepository::new(pg_pool.as_ref().clone());

        let pg_area = area_to_pg_area(area);
        repo.update(&pg_area).await?;

        Ok(StatusCode::OK)
    } else {
        info!("Updating area in MongoDB: {}", area.name);
        area.update(&state.db)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB update failed: {}", e)))?;

        Ok(StatusCode::OK)
    }
}

/// Delete area
pub async fn delete_area(
    State(state): State<AppState>,
    Path((_entity_id, area_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Deleting area from PostgreSQL: {}", area_id);
        let repo = AreaRepository::new(pg_pool.as_ref().clone());
        repo.delete(&area_id).await?;

        Ok(StatusCode::NO_CONTENT)
    } else {
        info!("Deleting area from MongoDB: {}", area_id);

        Area::delete_by_id(&state.db, &area_id)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB delete failed: {}", e)))?;

        Ok(StatusCode::NO_CONTENT)
    }
}

/// Create beacon
pub async fn create_beacon(
    State(state): State<AppState>,
    Json(beacon): Json<Beacon>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Creating beacon in PostgreSQL: {}", beacon.name);
        let repo = BeaconRepository::new(pg_pool.as_ref().clone());

        let pg_beacon = beacon_to_pg_beacon(beacon.clone());
        let _id = repo.create(&pg_beacon).await?;

        info!("Beacon created in PostgreSQL");
        Ok((StatusCode::CREATED, Json(beacon)))
    } else {
        info!("Creating beacon in MongoDB: {}", beacon.name);
        beacon
            .create(&state.db)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB insert failed: {}", e)))?;

        Ok((StatusCode::CREATED, Json(beacon)))
    }
}

/// Update beacon
pub async fn update_beacon(
    State(state): State<AppState>,
    Json(beacon): Json<Beacon>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Updating beacon in PostgreSQL: {}", beacon.name);
        let repo = BeaconRepository::new(pg_pool.as_ref().clone());

        let pg_beacon = beacon_to_pg_beacon(beacon);
        repo.update(&pg_beacon).await?;

        Ok(StatusCode::OK)
    } else {
        info!("Updating beacon in MongoDB: {}", beacon.name);
        beacon
            .update(&state.db)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB update failed: {}", e)))?;

        Ok(StatusCode::OK)
    }
}

/// Delete beacon
pub async fn delete_beacon(
    State(state): State<AppState>,
    Path((_entity_id, beacon_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Deleting beacon from PostgreSQL: {}", beacon_id);
        let repo = BeaconRepository::new(pg_pool.as_ref().clone());
        repo.delete(&beacon_id).await?;

        Ok(StatusCode::NO_CONTENT)
    } else {
        info!("Deleting beacon from MongoDB: {}", beacon_id);

        Beacon::delete_by_id(&state.db, &beacon_id)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB delete failed: {}", e)))?;

        Ok(StatusCode::NO_CONTENT)
    }
}

// ============================================================================
// Merchant Handlers
// ============================================================================

/// Get merchants by entity
pub async fn get_merchants_by_entity(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        debug!("Using PostgreSQL for merchants lookup");
        let repo = MerchantRepository::new(pg_pool.as_ref().clone());
        let uuid = Uuid::parse_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid UUID".to_string()))?;

        let pg_merchants = repo
            .get_by_entity(uuid, pagination.offset, pagination.limit)
            .await?;

        let merchants: Vec<Merchant> = pg_merchants
            .into_iter()
            .map(pg_merchant_to_merchant)
            .collect();
        Ok(Json(merchants))
    } else {
        debug!("Using MongoDB for merchants lookup");
        let oid = ObjectId::from_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid ObjectId".to_string()))?;

        let collection = state.db.collection::<Merchant>("merchants");
        let cursor = collection
            .find(bson::doc! { "entity": oid })
            .limit(pagination.limit)
            .skip(pagination.offset as u64)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB query failed: {}", e)))?;

        let merchants: Vec<Merchant> = cursor
            .try_collect()
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB cursor failed: {}", e)))?;

        Ok(Json(merchants))
    }
}

/// Get merchant by ID
pub async fn get_merchant_by_id(
    State(state): State<AppState>,
    Path((_entity_id, merchant_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        debug!("Using PostgreSQL for merchant lookup: {}", merchant_id);
        let repo = MerchantRepository::new(pg_pool.as_ref().clone());

        match repo.get_by_id(&merchant_id).await? {
            Some(pg_merchant) => {
                let merchant = pg_merchant_to_merchant(pg_merchant);
                Ok(Json(merchant))
            }
            None => Err(ServerError::NotFound(format!(
                "Merchant {} not found",
                merchant_id
            ))),
        }
    } else {
        debug!("Using MongoDB for merchant lookup: {}", merchant_id);

        let merchant = Merchant::get_one_by_id(&state.db, &merchant_id)
            .await
            .ok_or_else(|| {
                ServerError::NotFound(format!("Merchant {} not found", merchant_id))
            })?;

        Ok(Json(merchant))
    }
}

/// Create merchant
pub async fn create_merchant(
    State(state): State<AppState>,
    Json(merchant): Json<Merchant>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Creating merchant in PostgreSQL: {}", merchant.name);
        let repo = MerchantRepository::new(pg_pool.as_ref().clone());

        let pg_merchant = merchant_to_pg_merchant(merchant.clone());
        let _id = repo.create(&pg_merchant).await?;

        info!("Merchant created in PostgreSQL");
        Ok((StatusCode::CREATED, Json(merchant)))
    } else {
        info!("Creating merchant in MongoDB: {}", merchant.name);
        merchant
            .create(&state.db)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB insert failed: {}", e)))?;

        Ok((StatusCode::CREATED, Json(merchant)))
    }
}

/// Update merchant
pub async fn update_merchant(
    State(state): State<AppState>,
    Json(merchant): Json<Merchant>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Updating merchant in PostgreSQL: {}", merchant.name);
        let repo = MerchantRepository::new(pg_pool.as_ref().clone());

        let pg_merchant = merchant_to_pg_merchant(merchant);
        repo.update(&pg_merchant).await?;

        Ok(StatusCode::OK)
    } else {
        info!("Updating merchant in MongoDB: {}", merchant.name);
        merchant
            .update(&state.db)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB update failed: {}", e)))?;

        Ok(StatusCode::OK)
    }
}

/// Delete merchant
pub async fn delete_merchant(
    State(state): State<AppState>,
    Path((_entity_id, merchant_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Deleting merchant from PostgreSQL: {}", merchant_id);
        let repo = MerchantRepository::new(pg_pool.as_ref().clone());
        repo.delete(&merchant_id).await?;

        Ok(StatusCode::NO_CONTENT)
    } else {
        info!("Deleting merchant from MongoDB: {}", merchant_id);

        Merchant::delete_by_id(&state.db, &merchant_id)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB delete failed: {}", e)))?;

        Ok(StatusCode::NO_CONTENT)
    }
}

// ============================================================================
// Connection Handlers
// ============================================================================

/// Get connections by entity
pub async fn get_connections_by_entity(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        debug!("Using PostgreSQL for connections lookup");
        let repo = ConnectionRepository::new(pg_pool.as_ref().clone());
        let uuid = Uuid::parse_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid UUID".to_string()))?;

        let pg_connections = repo
            .get_by_entity(uuid, pagination.offset, pagination.limit)
            .await?;

        let connections: Vec<Connection> = pg_connections
            .into_iter()
            .map(pg_connection_to_connection)
            .collect();
        Ok(Json(connections))
    } else {
        debug!("Using MongoDB for connections lookup");
        let oid = ObjectId::from_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid ObjectId".to_string()))?;

        let collection = state.db.collection::<Connection>("connections");
        let cursor = collection
            .find(bson::doc! { "entity": oid })
            .limit(pagination.limit)
            .skip(pagination.offset as u64)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB query failed: {}", e)))?;

        let connections: Vec<Connection> = cursor
            .try_collect()
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB cursor failed: {}", e)))?;

        Ok(Json(connections))
    }
}

/// Get connection by ID
pub async fn get_connection_by_id(
    State(state): State<AppState>,
    Path((_entity_id, connection_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        debug!("Using PostgreSQL for connection lookup: {}", connection_id);
        let repo = ConnectionRepository::new(pg_pool.as_ref().clone());

        match repo.get_by_id(&connection_id).await? {
            Some(pg_connection) => {
                let connection = pg_connection_to_connection(pg_connection);
                Ok(Json(connection))
            }
            None => Err(ServerError::NotFound(format!(
                "Connection {} not found",
                connection_id
            ))),
        }
    } else {
        debug!("Using MongoDB for connection lookup: {}", connection_id);

        let connection = Connection::get_one_by_id(&state.db, &connection_id)
            .await
            .ok_or_else(|| {
                ServerError::NotFound(format!("Connection {} not found", connection_id))
            })?;

        Ok(Json(connection))
    }
}

/// Create connection
pub async fn create_connection(
    State(state): State<AppState>,
    Json(connection): Json<Connection>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Creating connection in PostgreSQL: {}", connection.name);
        let repo = ConnectionRepository::new(pg_pool.as_ref().clone());

        let pg_connection = connection_to_pg_connection(connection.clone());
        let _id = repo.create(&pg_connection).await?;

        info!("Connection created in PostgreSQL");
        Ok((StatusCode::CREATED, Json(connection)))
    } else {
        info!("Creating connection in MongoDB: {}", connection.name);
        connection
            .create(&state.db)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB insert failed: {}", e)))?;

        Ok((StatusCode::CREATED, Json(connection)))
    }
}

/// Update connection
pub async fn update_connection(
    State(state): State<AppState>,
    Json(connection): Json<Connection>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Updating connection in PostgreSQL: {}", connection.name);
        let repo = ConnectionRepository::new(pg_pool.as_ref().clone());

        let pg_connection = connection_to_pg_connection(connection);
        repo.update(&pg_connection).await?;

        Ok(StatusCode::OK)
    } else {
        info!("Updating connection in MongoDB: {}", connection.name);
        connection
            .update(&state.db)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB update failed: {}", e)))?;

        Ok(StatusCode::OK)
    }
}

/// Delete connection
pub async fn delete_connection(
    State(state): State<AppState>,
    Path((_entity_id, connection_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        info!("Deleting connection from PostgreSQL: {}", connection_id);
        let repo = ConnectionRepository::new(pg_pool.as_ref().clone());
        repo.delete(&connection_id).await?;

        Ok(StatusCode::NO_CONTENT)
    } else {
        info!("Deleting connection from MongoDB: {}", connection_id);

        Connection::delete_by_id(&state.db, &connection_id)
            .await
            .map_err(|e| ServerError::DatabaseQuery(format!("MongoDB delete failed: {}", e)))?;

        Ok(StatusCode::NO_CONTENT)
    }
}
