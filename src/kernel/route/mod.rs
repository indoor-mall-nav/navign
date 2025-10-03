use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bson::doc;
use bumpalo::Bump;
use futures::TryStreamExt;
use mongodb::Database;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::__rt::spawn_blocking;
use tokio::spawn;
use crate::AppState;
use crate::kernel::route::instructions::InstructionType;
use crate::kernel::route::types::{Atom, FromIn};
use crate::kernel::route::utils::{NavigationError, Navigate};
use crate::kernel::route::utils::connectivity::ConnectivityLimits;
use crate::schema::{Area, Connection, Entity, Merchant, Service};

pub mod types;
pub mod instructions;
pub mod utils;

pub fn route_merchant(departure_merchant: &str, arrival_merchant: &str, entity_id: Entity, areas: Vec<Area>, connections: Vec<Connection>, merchants: Vec<Merchant>, limits: ConnectivityLimits) -> Result<Vec<InstructionType>, NavigationError> {
    let alloc = Bump::default();
    let departure = match merchants.iter().find(|m| m.id.to_hex() == departure_merchant) {
        Some(m) => m,
        None => return Err(NavigationError::InvalidDeparture),
    };
    let arrival = match merchants.iter().find(|m| m.id.to_hex() == arrival_merchant) {
        Some(m) => m,
        None => return Err(NavigationError::InvalidArrival),
    };
    let entity = match types::entity::Entity::convert_area_in(&alloc, entity_id, areas, connections, merchants).await {
        Some(e) => e,
        None => return Err(NavigationError::InvalidArrival),
    };
    println!("Routing within entity: {}", entity.name);
    let dep_area = departure.area.to_hex();
    let arr_area = arrival.area.to_hex();
    println!("Routing from merchant {} in area {} to merchant {} in area {} within entity {}", departure_merchant, dep_area, arrival_merchant, arr_area, entity_id);
    let src = (departure.location.0, departure.location.1, Atom::from_in(dep_area, &alloc));
    println!("Source location: {:?}", src);
    let dest = (arrival.location.0, arrival.location.1, Atom::from_in(arr_area, &alloc));
    println!("Destination location: {:?}", dest);
    entity.navigate(src, dest, limits, &alloc)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteQuery {
    from: Option<String>,
    to: Option<String>,
    block: Option<String>
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
        block
    } = query;
    let limits = ConnectivityLimits {
        elevator: block.as_ref().map(|x| !x.contains('e')).unwrap_or(true),
        stairs: block.as_ref().map(|x| !x.contains('e')).unwrap_or(true),
        escalator: block.as_ref().map(|x| !x.contains('e')).unwrap_or(true),
    };
    if from.is_none() || to.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({"error": "Missing 'from' or 'to' parameter"})),
        );
    }
    let local = tokio::task::LocalSet::new();
    let entity = match Entity::get_one_by_id(&state.db, entity.as_str()).await {
        Some(e) => e,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({"error": "Entity not found"})),
            );
        }
    };
    let areas = state.db.collection(Area::get_collection_name()).find(doc! { "entity": entity.id }).await.unwrap().try_collect::<Vec<Area>>().await.unwrap_or_default();
    let connections = state.db.collection(Connection::get_collection_name()).find(doc! { "entity": entity.id }).await.unwrap().try_collect::<Vec<Connection>>().await.unwrap_or_default();
    let merchants = state.db.collection(Merchant::get_collection_name()).find(doc! { "entity": entity.id }).await.unwrap().try_collect::<Vec<Merchant>>().await.unwrap_or_default();
    if areas.is_empty() || connections.is_empty() || merchants.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({"error": "Insufficient data in entity"})),
        );
    }
    spawn_blocking(move || {
        match route_merchant(from.unwrap().as_str(), to.unwrap().as_str(), entity, areas, connections, merchants, limits) {
            Ok(instructions) => {
                let instructions = instructions.clone();
                let json = json!({ "instructions": instructions });
                (StatusCode::OK, axum::Json(json))
            }
            Err(e) => (StatusCode::BAD_REQUEST, axum::Json(json!({"error": e}))),
        }
    }).await;
}
