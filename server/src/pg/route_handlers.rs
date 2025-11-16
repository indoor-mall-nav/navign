//! Route/Pathfinding handlers with dual-database support
//!
//! These handlers implement complex pathfinding logic with manual SQL queries
//! for PostgreSQL, maintaining compatibility with the MongoDB implementation.

use crate::error::{Result, ServerError};
use crate::kernel::route::{ConnectivityLimits, RouteQuery, route};
use crate::pg::adapters::*;
use crate::pg::repository::*;
use crate::schema::{Area, Connection, Entity, Merchant};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use bson::oid::ObjectId;
use futures::TryStreamExt;
use serde_json::json;
use sqlx::types::Uuid as SqlxUuid;
use std::str::FromStr;
use tokio::task::spawn_blocking;
use tracing::{info, trace};

/// Find route with dual-database support and manual SQL queries
///
/// This handler performs complex pathfinding by:
/// 1. Fetching entity metadata
/// 2. Querying all areas within the entity
/// 3. Querying all connections (elevators, stairs, escalators)
/// 4. Querying all merchants (stores, facilities)
/// 5. Running Dijkstra's pathfinding algorithm
/// 6. Generating turn-by-turn navigation instructions
#[axum::debug_handler]
pub async fn find_route_pg(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
    Query(query): Query<RouteQuery>,
) -> Result<impl IntoResponse> {
    let RouteQuery {
        from,
        to,
        disallow: block,
    } = query;

    let limits = ConnectivityLimits {
        elevator: block.as_ref().map(|x| !x.contains('e')).unwrap_or(true),
        stairs: block.as_ref().map(|x| !x.contains('s')).unwrap_or(true),
        escalator: block.as_ref().map(|x| !x.contains('c')).unwrap_or(true),
    };

    info!(
        "Route query: from={:?}, to={:?}, block={:?}, limits={:?}",
        from, to, block, limits
    );

    if from.is_none() || to.is_none() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Missing 'from' or 'to' parameter"})),
        ));
    }

    if let Some(pg_pool) = state.pg_pool.as_ref() {
        // PostgreSQL pathfinding with manual SQL queries
        info!("Using PostgreSQL for pathfinding");

        let entity_repo = EntityRepository::new(pg_pool.as_ref().clone());
        let area_repo = AreaRepository::new(pg_pool.as_ref().clone());
        let connection_repo = ConnectionRepository::new(pg_pool.as_ref().clone());
        let merchant_repo = MerchantRepository::new(pg_pool.as_ref().clone());

        // 1. Fetch entity
        let pg_entity = entity_repo
            .get_by_id(&entity_id)
            .await?
            .ok_or_else(|| ServerError::NotFound("Entity not found".to_string()))?;

        let entity = pg_entity_to_entity(pg_entity.clone());

        // 2. Fetch all areas for this entity using manual SQL
        info!("Fetching areas for entity {}", entity_id);
        let entity_uuid = SqlxUuid::parse_str(&entity_id)
            .map_err(|_| ServerError::InvalidInput("Invalid UUID".to_string()))?;

        let pg_areas = sqlx::query_as::<_, crate::pg::models::PgArea>(
            "SELECT * FROM areas WHERE entity_id = $1 ORDER BY name",
        )
        .bind(entity_uuid)
        .fetch_all(pg_pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch areas: {}", e)))?;

        if pg_areas.is_empty() {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "No areas found in entity"})),
            ));
        }

        let areas: Vec<Area> = pg_areas.into_iter().map(pg_area_to_area).collect();
        info!("Found {} areas", areas.len());

        // 3. Fetch all connections using manual SQL
        info!("Fetching connections for entity {}", entity_id);
        let pg_connections = sqlx::query_as::<_, crate::pg::models::PgConnection>(
            "SELECT * FROM connections WHERE entity_id = $1 ORDER BY name",
        )
        .bind(entity_uuid)
        .fetch_all(pg_pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch connections: {}", e)))?;

        if pg_connections.is_empty() {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "No connections found in entity"})),
            ));
        }

        let connections: Vec<Connection> = pg_connections
            .into_iter()
            .map(pg_connection_to_connection)
            .collect();
        info!("Found {} connections", connections.len());

        // 4. Fetch all merchants using manual SQL
        info!("Fetching merchants for entity {}", entity_id);
        let pg_merchants = sqlx::query_as::<_, crate::pg::models::PgMerchant>(
            "SELECT * FROM merchants WHERE entity_id = $1 ORDER BY name",
        )
        .bind(entity_uuid)
        .fetch_all(pg_pool.inner())
        .await
        .map_err(|e| ServerError::DatabaseQuery(format!("Failed to fetch merchants: {}", e)))?;

        if pg_merchants.is_empty() {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "No merchants found in entity"})),
            ));
        }

        let merchants: Vec<Merchant> = pg_merchants
            .into_iter()
            .map(pg_merchant_to_merchant)
            .collect();
        info!("Found {} merchants", merchants.len());

        // 5. Run pathfinding algorithm in blocking task
        let from_str = from.expect("from was already validated");
        let to_str = to.expect("to was already validated");

        spawn_blocking(move || {
            trace!(
                "Starting route computation from {} to {}, entity name: {}, {} areas, {} connections, {} merchants",
                from_str, to_str, entity.name, areas.len(), connections.len(), merchants.len()
            );

            match route(
                from_str.as_str(),
                to_str.as_str(),
                entity,
                areas,
                connections,
                merchants,
                limits,
            ) {
                Ok(instructions) => {
                    let json = json!({ "instructions": instructions });
                    (StatusCode::OK, Json(json))
                }
                Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e}))),
            }
        })
        .await
        .map_err(|e| {
            ServerError::InternalError(format!("Pathfinding task failed: {}", e))
        })
        .and_then(|result| Ok(result))
    } else {
        // MongoDB fallback
        info!("Using MongoDB for pathfinding");

        let entity = match Entity::get_one_by_id(&state.db, entity_id.as_str()).await {
            Some(e) => e,
            None => {
                return Ok((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Entity not found"})),
                ));
            }
        };

        let areas = match state
            .db
            .collection(Area::get_collection_name())
            .find(bson::doc! { "entity": entity.id })
            .await
        {
            Ok(cursor) => cursor.try_collect::<Vec<Area>>().await.unwrap_or_default(),
            Err(e) => {
                tracing::error!("Failed to find areas: {}", e);
                return Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to query areas"})),
                ));
            }
        };

        let connections = match state
            .db
            .collection(Connection::get_collection_name())
            .find(bson::doc! { "entity": entity.id })
            .await
        {
            Ok(cursor) => cursor
                .try_collect::<Vec<Connection>>()
                .await
                .unwrap_or_default(),
            Err(e) => {
                tracing::error!("Failed to find connections: {}", e);
                return Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to query connections"})),
                ));
            }
        };

        let merchants = match state
            .db
            .collection(Merchant::get_collection_name())
            .find(bson::doc! { "entity": entity.id })
            .await
        {
            Ok(cursor) => cursor
                .try_collect::<Vec<Merchant>>()
                .await
                .unwrap_or_default(),
            Err(e) => {
                tracing::error!("Failed to find merchants: {}", e);
                return Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to query merchants"})),
                ));
            }
        };

        if areas.is_empty() || connections.is_empty() || merchants.is_empty() {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Insufficient data in entity"})),
            ));
        }

        let from_str = from.expect("from was already validated");
        let to_str = to.expect("to was already validated");

        spawn_blocking(move || {
            trace!(
                "Starting route computation from {} to {}, entity name: {}, {} areas, {} connections, {} merchants",
                from_str, to_str, entity.name, areas.len(), connections.len(), merchants.len()
            );

            match route(
                from_str.as_str(),
                to_str.as_str(),
                entity,
                areas,
                connections,
                merchants,
                limits,
            ) {
                Ok(instructions) => {
                    let json = json!({ "instructions": instructions });
                    (StatusCode::OK, Json(json))
                }
                Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e}))),
            }
        })
        .await
        .map_err(|e| {
            ServerError::InternalError(format!("Pathfinding task failed: {}", e))
        })
        .and_then(|result| Ok(result))
    }
}
