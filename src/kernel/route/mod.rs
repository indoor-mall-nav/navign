use crate::AppState;
use crate::kernel::route::instructions::InstructionType;
use crate::kernel::route::types::{Atom, FromIn};
use crate::kernel::route::utils::connectivity::ConnectivityLimits;
use crate::kernel::route::utils::{Navigate, NavigationError};
use crate::schema::{Area, Connection, Entity, Merchant, Service};
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

pub mod instructions;
pub mod types;
pub mod utils;

pub fn route_merchant(
    departure_merchant: &str,
    arrival_merchant: &str,
    entity_id: Entity,
    areas: Vec<Area>,
    connections: Vec<Connection>,
    merchants: Vec<Merchant>,
    limits: ConnectivityLimits,
) -> Result<Vec<InstructionType>, NavigationError> {
    let alloc = Bump::default();
    let departure = match merchants
        .iter()
        .find(|m| m.id.to_hex() == departure_merchant)
    {
        Some(m) => m,
        None => return Err(NavigationError::InvalidDeparture),
    }
    .clone();
    let arrival = match merchants.iter().find(|m| m.id.to_hex() == arrival_merchant) {
        Some(m) => m,
        None => return Err(NavigationError::InvalidArrival),
    }
    .clone();
    let entity = match types::entity::Entity::convert_area_in(
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
    let dep_area = departure.area.to_hex();
    let arr_area = arrival.area.to_hex();
    let src = (
        departure.location.0,
        departure.location.1,
        Atom::from_in(dep_area, &alloc),
    );
    trace!("Source location: {:?}", src);
    let dest = (
        arrival.location.0,
        arrival.location.1,
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
    let entity = match Entity::get_one_by_id(&state.db, entity.as_str()).await {
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
        .collection(Area::get_collection_name())
        .find(doc! { "entity": entity.id })
        .await
        .unwrap()
        .try_collect::<Vec<Area>>()
        .await
        .unwrap_or_default();
    let connections = state
        .db
        .collection(Connection::get_collection_name())
        .find(doc! { "entity": entity.id })
        .await
        .unwrap()
        .try_collect::<Vec<Connection>>()
        .await
        .unwrap_or_default();
    let merchants = state
        .db
        .collection(Merchant::get_collection_name())
        .find(doc! { "entity": entity.id })
        .await
        .unwrap()
        .try_collect::<Vec<Merchant>>()
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
        match route_merchant(
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
