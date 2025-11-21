//! Unlocker module - handles unlock instance creation and verification
//! 
//! This module provides API endpoints for the BLE-based access control system.
//! Currently simplified without user authentication - to be integrated later.

use crate::error::ServerError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UnlockData {
    pub payload: String,
}

/// Placeholder handler for creating unlock instances
/// TODO: Implement full unlock logic with user authentication
#[axum::debug_handler]
pub async fn create_unlock_instance(
    State(_state): State<AppState>,
    Path((entity_id, beacon_id)): Path<(String, String)>,
    Json(_data): Json<UnlockData>,
) -> Result<impl IntoResponse, ServerError> {
    info!(
        "Creating unlock instance for beacon {} in {} (placeholder)",
        beacon_id, entity_id
    );

    Ok(Json(json!({
        "status": "not_implemented",
        "message": "Unlock functionality to be implemented with user authentication"
    })))
}

/// Placeholder handler for updating unlock instances
/// TODO: Implement full verification logic
#[axum::debug_handler]
pub async fn update_unlock_instance(
    State(_state): State<AppState>,
    Path((entity_id, beacon_id, instance_id)): Path<(String, String, String)>,
    Json(_data): Json<UnlockData>,
) -> Result<impl IntoResponse, ServerError> {
    info!(
        "Updating unlock instance {} for beacon {} in {} (placeholder)",
        instance_id, beacon_id, entity_id
    );

    Ok(Json(json!({
        "status": "not_implemented",
        "message": "Unlock update functionality to be implemented"
    })))
}

/// Placeholder handler for recording unlock results
/// TODO: Implement result recording
#[axum::debug_handler]
pub async fn record_unlock_result(
    State(_state): State<AppState>,
    Path((entity_id, beacon_id, instance_id)): Path<(String, String, String)>,
    Json(_data): Json<serde_json::Value>,
) -> Result<impl IntoResponse, ServerError> {
    info!(
        "Recording unlock result for instance {} of beacon {} in {} (placeholder)",
        instance_id, beacon_id, entity_id
    );

    Ok(Json(json!({
        "status": "not_implemented",
        "message": "Result recording functionality to be implemented"
    })))
}
