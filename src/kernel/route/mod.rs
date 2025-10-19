use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bson::doc;
use bumpalo::Bump;
use futures::TryStreamExt;
use log::{info, trace};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::task::spawn_blocking;

pub mod implementations;
pub mod instructions;
pub mod types;

use crate::schema::Service;
pub use implementations::*;
pub use instructions::*;
pub use types::*;

pub fn route(
    departure: &str,
    arrival: &str,
    entity_id: crate::schema::Entity,
    areas: Vec<crate::schema::Area>,
    connections: Vec<crate::schema::Connection>,
    merchants: Vec<crate::schema::Merchant>,
    limits: ConnectivityLimits,
) -> Result<Vec<InstructionType>, NavigationError> {
    let departure = if departure.contains(",") {
        let parts: Vec<&str> = departure.split(',').collect();
        if parts.len() != 3 {
            return Err(NavigationError::InvalidDeparture);
        }
        let lon = parts[0].parse::<f64>().map_err(|_| NavigationError::InvalidDeparture)?;
        let lat = parts[1].parse::<f64>().map_err(|_| NavigationError::InvalidDeparture)?;
        let area = parts[2].to_string();
        (lon, lat, area)
    } else {
        let departure = match merchants
            .iter()
            .find(|m| m.id.to_hex() == departure)
        {
            Some(m) => m,
            None => return Err(NavigationError::InvalidDeparture),
        }
            .clone();
        (departure.location.0, departure.location.1, departure.area.to_hex())
    };
    let arrival = if arrival.contains(",") {
        let parts: Vec<&str> = arrival.split(',').collect();
        if parts.len() != 3 {
            return Err(NavigationError::InvalidArrival);
        }
        let lon = parts[0].parse::<f64>().map_err(|_| NavigationError::InvalidArrival)?;
        let lat = parts[1].parse::<f64>().map_err(|_| NavigationError::InvalidArrival)?;
        let area = parts[2].to_string();
        (lon, lat, area)
    } else {
        let arrival = match merchants.iter().find(|m| m.id.to_hex() == arrival) {
            Some(m) => m,
            None => return Err(NavigationError::InvalidArrival),
        }
            .clone();
        (arrival.location.0, arrival.location.1, arrival.area.to_hex())
    };
    route_point(
        departure,
        arrival,
        entity_id,
        areas,
        connections,
        merchants,
        limits,
    )
}

pub fn route_point(
    departure: (f64, f64, String),
    arrival: (f64, f64, String),
    entity_id: crate::schema::Entity,
    areas: Vec<crate::schema::Area>,
    connections: Vec<crate::schema::Connection>,
    merchants: Vec<crate::schema::Merchant>,
    limits: ConnectivityLimits,
) -> Result<Vec<InstructionType>, NavigationError> {
    let alloc = Bump::default();
    let entity = match Entity::convert_entity_in(
        &alloc,
        entity_id,
        areas,
        connections,
        merchants,
    ) {
        Some(e) => e,
        None => return Err(NavigationError::InvalidArrival),
    };
    trace!("Entity: {}", entity);
    trace!("Routing within entity: {}", entity.name);
    let dep_area = departure.2;
    let arr_area = arrival.2;
    let src = (
        departure.0,
        departure.1,
        Atom::from_in(dep_area, &alloc),
    );
    trace!("Source location: {:?}", src);
    let dest = (
        arrival.0,
        arrival.1,
        Atom::from_in(arr_area, &alloc),
    );
    trace!("Destination location: {:?}", dest);
    entity.navigate(src, dest, limits, &alloc)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteQuery {
    from: Option<String>,
    to: Option<String>,
    disallow: Option<String>,
}

#[axum::debug_handler]
pub async fn find_route(
    State(state): State<AppState>,
    Path(entity): Path<String>,
    Query(query): Query<RouteQuery>,
) -> impl IntoResponse {
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
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({"error": "Missing 'from' or 'to' parameter"})),
        );
    }
    let entity = match crate::schema::Entity::get_one_by_id(&state.db, entity.as_str()).await {
        Some(e) => e,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({"error": "Entity not found"})),
            );
        }
    };
    let areas = state
        .db
        .collection(crate::schema::Area::get_collection_name())
        .find(doc! { "entity": entity.id })
        .await
        .unwrap()
        .try_collect::<Vec<crate::schema::Area>>()
        .await
        .unwrap_or_default();
    let connections = state
        .db
        .collection(crate::schema::Connection::get_collection_name())
        .find(doc! { "entity": entity.id })
        .await
        .unwrap()
        .try_collect::<Vec<crate::schema::Connection>>()
        .await
        .unwrap_or_default();
    let merchants = state
        .db
        .collection(crate::schema::Merchant::get_collection_name())
        .find(doc! { "entity": entity.id })
        .await
        .unwrap()
        .try_collect::<Vec<crate::schema::Merchant>>()
        .await
        .unwrap_or_default();
    if areas.is_empty() || connections.is_empty() || merchants.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({"error": "Insufficient data in entity"})),
        );
    }
    spawn_blocking(move || {
        trace!("Starting route computation from {} to {}, entity name: {}, {} areas, {} connections, {} merchants", from.as_ref().unwrap(), to.as_ref().unwrap(), entity.name, areas.len(), connections.len(), merchants.len());
        match route(
            from.unwrap().as_str(),
            to.unwrap().as_str(),
            entity,
            areas,
            connections,
            merchants,
            limits,
        ) {
            Ok(instructions) => {
                let instructions = instructions.clone();
                let json = json!({ "instructions": instructions });
                (StatusCode::OK, axum::Json(json))
            }
            Err(e) => (StatusCode::BAD_REQUEST, axum::Json(json!({"error": e}))),
        }
    })
    .await
    .unwrap_or_else(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({"error": format!("Internal error: {}", e)})),
        )
    })
}
