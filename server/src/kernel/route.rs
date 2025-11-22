use crate::error::ServerError;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use navign_shared::IntRepository;
use navign_shared::pathfinding::{
    AreaData, ConnectionData, ConnectivityLimits, RouteInstruction, find_path_between_areas,
};
use navign_shared::schema::{Area, Connection, Merchant};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteQuery {
    /// Starting location: either "x,y,area_id" or merchant_id
    pub from: String,
    /// Destination: either "x,y,area_id" or merchant_id  
    pub to: String,
    /// Disallowed connection types: 'e' for elevator, 's' for stairs, 'c' for escalator
    #[serde(default)]
    pub disallow: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    pub instructions: Vec<RouteInstruction>,
}

/// Parse location from either "x,y,area_id" format or merchant_id
async fn parse_location(
    location_str: &str,
    merchants: &[Merchant],
) -> Result<(f64, f64, i32), ServerError> {
    if location_str.contains(',') {
        // Parse as "x,y,area_id"
        let parts: Vec<&str> = location_str.split(',').collect();
        if parts.len() != 3 {
            return Err(ServerError::InvalidInput(
                "Location format should be 'x,y,area_id'".to_string(),
            ));
        }
        let x = parts[0]
            .parse::<f64>()
            .map_err(|_| ServerError::InvalidInput("Invalid x coordinate".to_string()))?;
        let y = parts[1]
            .parse::<f64>()
            .map_err(|_| ServerError::InvalidInput("Invalid y coordinate".to_string()))?;
        let area_id = parts[2]
            .parse::<i32>()
            .map_err(|_| ServerError::InvalidInput("Invalid area_id".to_string()))?;
        Ok((x, y, area_id))
    } else {
        // Parse as merchant_id
        let merchant_id = location_str
            .parse::<i32>()
            .map_err(|_| ServerError::InvalidInput("Invalid merchant ID".to_string()))?;

        let merchant = merchants
            .iter()
            .find(|m| m.id == merchant_id)
            .ok_or_else(|| ServerError::NotFound("Merchant not found".to_string()))?;

        let location = merchant.location();
        Ok((location.0, location.1, merchant.area_id))
    }
}

/// Convert Area to AreaData for pathfinding
fn area_to_area_data(area: &Area, connections: &[Connection]) -> Result<AreaData, ServerError> {
    // Extract polygon coordinates
    let polygon_coords: Vec<(f64, f64)> = area
        .polygon()
        .map_err(|e| ServerError::InternalError(format!("Failed to parse polygon: {}", e)))?;

    let polygon = navign_shared::pathfinding::Polygon::new(polygon_coords);

    // Find connections for this area
    let area_connections: Vec<ConnectionData> = connections
        .iter()
        .filter(|conn| conn.connected_areas.iter().any(|ca| ca.0 == area.id))
        .map(|conn| {
            let connected_areas = conn
                .connected_areas
                .iter()
                .filter(|ca| ca.0 != area.id)
                .map(|ca| (ca.0, ca.1, ca.2, ca.3))
                .collect();

            ConnectionData {
                id: conn.id,
                conn_type: conn.r#type,
                connected_areas,
            }
        })
        .collect();

    Ok(AreaData {
        id: area.id,
        polygon,
        connections: area_connections,
    })
}

/// Main route finding handler
#[axum::debug_handler]
pub async fn find_route(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
    Query(query): Query<RouteQuery>,
) -> Result<impl IntoResponse, ServerError> {
    info!(
        "Route query for entity {}: from={}, to={}, disallow={:?}",
        entity_id, query.from, query.to, query.disallow
    );

    // Parse entity UUID
    let entity_uuid = uuid::Uuid::parse_str(&entity_id)
        .map_err(|_| ServerError::InvalidInput("Invalid entity UUID".to_string()))?;

    // Fetch areas, connections, and merchants from PostgreSQL
    let areas = Area::list(state.pg_pool.inner(), 0, 10000, entity_uuid)
        .await
        .map_err(ServerError::Database)?;

    let connections = Connection::list(state.pg_pool.inner(), 0, 10000, entity_uuid)
        .await
        .map_err(ServerError::Database)?;

    let merchants = Merchant::list(state.pg_pool.inner(), 0, 10000, entity_uuid)
        .await
        .map_err(ServerError::Database)?;

    if areas.is_empty() {
        return Err(ServerError::NotFound(
            "No areas found for entity".to_string(),
        ));
    }

    // Parse locations
    let (start_x, start_y, start_area_id) = parse_location(&query.from, &merchants).await?;
    let (end_x, end_y, end_area_id) = parse_location(&query.to, &merchants).await?;

    info!(
        "Parsed locations: start=({}, {}) in area {}, end=({}, {}) in area {}",
        start_x, start_y, start_area_id, end_x, end_y, end_area_id
    );

    // Parse connectivity limits
    let limits = if let Some(ref disallow) = query.disallow {
        ConnectivityLimits {
            elevator: !disallow.contains('e'),
            stairs: !disallow.contains('s'),
            escalator: !disallow.contains('c'),
        }
    } else {
        ConnectivityLimits::default()
    };

    // Convert areas to AreaData
    let area_data: Vec<AreaData> = areas
        .iter()
        .map(|area| area_to_area_data(area, &connections))
        .collect::<Result<Vec<_>, _>>()?;

    // Find path using shared pathfinding module
    let instructions = find_path_between_areas(
        &area_data,
        start_area_id,
        (start_x, start_y),
        end_area_id,
        (end_x, end_y),
        limits,
        1.0, // block_size for inner-area pathfinding
    )
    .map_err(|e| {
        error!("Pathfinding error: {}", e);
        ServerError::InternalError(format!("Pathfinding failed: {}", e))
    })?;

    info!("Found path with {} instructions", instructions.len());

    Ok(Json(RouteResponse { instructions }))
}
