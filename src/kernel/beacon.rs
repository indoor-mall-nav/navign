use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::kernel::unlocker::{unlocker_instance, Unlocker};
use crate::schema::{Beacon, Service, User};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthenticationInstance {
    /// The ObjectId of the beacon that initiate the authentication
    pub beacon: ObjectId,
    /// The ObjectId of the account that is being authenticated
    pub account: ObjectId,
    /// The timestamp when the authentication is initiated
    pub initiated_at: i64,
    pub status: AuthenticationStatus,
    pub completed_at: Option<i64>,
    pub r#type: AuthenticationType,
    pub failed_reason: Option<FailedReason>,
    pub nonce: [u8; 16],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthenticationStatus {
    Initiated,
    InProgress,
    Completed,
    Refused,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthenticationType {
    Rfid,
    Nfc,
    Biometric,
    Totp,
    Password,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FailedReason {
    Timeout,
    InvalidCode,
    PermissionDenied,
    CredentialMismatch,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthenticationHistory {
    /// Instance of the authentication
    pub instance: ObjectId,
}

async fn initiate_authentication(
    State(state): State<AppState>,
    axum::Json(instance): axum::Json<AuthenticationInstance>,
) -> impl IntoResponse {
    // Save the instance to the database
    let collection = state
        .db
        .collection::<AuthenticationInstance>("authentication_instances");
    let user = User::get_one_by_id(&state.db, instance.account.to_hex().as_str()).await;
    if user.is_none() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Account does not exist".to_string(),
        )
            .into_response();
    }
    let user = user.unwrap();
    if !user.activated || !user.is_privileged() {
        return (
            axum::http::StatusCode::FORBIDDEN,
            "Account is not activated or not privileged".to_string(),
        )
            .into_response();
    }
    let beacon = Beacon::get_one_by_id(&state.db, instance.beacon.to_hex().as_str()).await;
    if beacon.is_none() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Beacon does not exist".to_string(),
        )
            .into_response();
    }

    let result = collection.insert_one(instance).await;

    match result {
        Ok(insert_result) => {
            let id = insert_result.inserted_id.as_object_id().map(|oid| oid.to_hex());
            match id {
                Some(id) => (axum::http::StatusCode::OK, id).into_response(),
                None => (
                    axum::http::StatusCode::BAD_REQUEST,
                    "Failed to get inserted ID".to_string(),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Failed to initiate authentication: {}", e),
        )
            .into_response(),
    }
}

pub async fn initiate_unlocker(State(state): State<AppState>, axum::Json(unlocker): axum::Json<Unlocker>) -> impl IntoResponse {
    match unlocker_instance(unlocker, &state.private_key, |_| true, &state.db).await {
        Ok(challenge) => {
            match serde_json::to_string(&challenge) {
                Ok(challenge) => {
                    (StatusCode::CREATED, challenge)
                }
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string())
    }
}