use crate::AppState;
use crate::kernel::auth::UserData;
use crate::kernel::unlocker::instance::UnlockInstance;
use crate::schema::{Beacon, Service};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use base64::Engine;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UnlockData {
    pub payload: String,
}

pub mod instance;
#[axum::debug_handler]
pub async fn create_unlock_instance(
    State(state): State<AppState>,
    Path((entity_id, beacon_id)): Path<(String, String)>,
    user: UserData,
    Json(data): Json<UnlockData>,
) -> impl IntoResponse {
    info!(
        "Creating unlock instance for beacon {} in {}",
        beacon_id, entity_id
    );

    let beacon = match Beacon::get_one_by_id(&state.db, beacon_id.as_str()).await {
        Some(beacon) => beacon,
        None => return (StatusCode::NOT_FOUND, "Beacon not found".to_string()),
    };

    if beacon.entity.to_hex() != entity_id {
        return (
            StatusCode::BAD_REQUEST,
            "Beacon does not belong to the entity".to_string(),
        );
    }
    match UnlockInstance::create_instance(
        &state.db,
        beacon_id,
        data.payload,
        user.sub.clone(),
        user.device.clone(),
    )
    .await
    {
        Ok(instance) => match state
            .db
            .collection::<UnlockInstance>(UnlockInstance::get_collection_name())
            .insert_one(&instance)
            .await
        {
            Ok(_) => {
                let nonce_encoded = hex::encode(instance.challenge_nonce.as_bytes());
                (
                    StatusCode::CREATED,
                    json!({ "instance_id": instance.get_id(), "nonce": nonce_encoded }).to_string(),
                )
            }
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        },
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()),
    }
}

#[axum::debug_handler]
pub async fn update_unlock_instance(
    State(state): State<AppState>,
    Path((entity_id, beacon_id, instance_id)): Path<(String, String, String)>,
    user: UserData,
    Json(data): Json<UnlockData>,
) -> impl IntoResponse {
    info!(
        "Updating unlock instance {} for beacon {} in {}",
        instance_id, beacon_id, entity_id
    );

    let beacon = match Beacon::get_one_by_id(&state.db, beacon_id.as_str()).await {
        Some(beacon) => beacon,
        None => return (StatusCode::NOT_FOUND, "Beacon not found".to_string()),
    };
    if beacon.entity.to_hex() != entity_id {
        return (
            StatusCode::BAD_REQUEST,
            "Beacon does not belong to the entity".to_string(),
        );
    }

    let instance = match UnlockInstance::get_one_by_id(&state.db, instance_id.as_str()).await {
        Some(instance) => instance,
        None => return (StatusCode::NOT_FOUND, "Instance not found".to_string()),
    };
    if instance.user != user.sub.as_str() {
        return (
            StatusCode::FORBIDDEN,
            "You are not allowed to update this instance".to_string(),
        );
    }
    let payload = match base64::engine::general_purpose::STANDARD.decode(data.payload) {
        Ok(payload) => payload,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()),
    };
    if payload.len() != 8 + 64 {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid payload length".to_string(),
        );
    }
    let mut signature = [0u8; 64];
    signature.copy_from_slice(&payload[0..64]);
    let timestamp = u64::from_be_bytes(payload[64..72].try_into().unwrap());
    match instance
        .update_instance(
            &state.db,
            &state.private_key,
            base64::engine::general_purpose::STANDARD.encode(signature),
            timestamp,
        )
        .await
    {
        Ok(proof) => (StatusCode::OK, proof),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockResult {
    success: bool,
    outcome: String,
}

pub async fn record_unlock_result(
    State(state): State<AppState>,
    Path((entity_id, beacon_id, instance_id)): Path<(String, String, String)>,
    user: UserData,
    Json(data): Json<UnlockResult>,
) -> impl IntoResponse {
    info!(
        "Recording unlock result for instance {} of beacon {} in {}",
        instance_id, beacon_id, entity_id
    );

    let beacon = match Beacon::get_one_by_id(&state.db, beacon_id.as_str()).await {
        Some(beacon) => beacon,
        None => return (StatusCode::NOT_FOUND, "Beacon not found".to_string()),
    };
    if beacon.entity.to_hex() != entity_id {
        return (
            StatusCode::BAD_REQUEST,
            "Beacon does not belong to the entity".to_string(),
        );
    }

    let instance = match UnlockInstance::get_one_by_id(&state.db, instance_id.as_str()).await {
        Some(instance) => instance,
        None => return (StatusCode::NOT_FOUND, "Instance not found".to_string()),
    };
    if instance.user != user.sub.as_str() {
        return (
            StatusCode::FORBIDDEN,
            "You are not allowed to update this instance".to_string(),
        );
    }
    match instance
        .record_results(&state.db, data.success, data.outcome)
        .await
    {
        Ok(_) => (StatusCode::OK, "Unlock result recorded".to_string()),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()),
    }
}
