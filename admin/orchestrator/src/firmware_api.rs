use crate::error::{OrchestratorError, Result};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use navign_shared::{Firmware, FirmwareDevice, FirmwareQuery};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct FirmwareClient {
    server_url: String,
    client: reqwest::Client,
}

impl FirmwareClient {
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch latest firmware metadata for a specific device
    pub async fn get_latest_firmware(&self, device: FirmwareDevice) -> Result<Firmware> {
        let url = format!(
            "{}/api/firmwares/latest/{}",
            self.server_url,
            device.as_str()
        );

        tracing::info!("Fetching latest firmware from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            tracing::error!("Failed to fetch firmware: {}", response.status());
            return Err(OrchestratorError::FirmwareServerUnavailable);
        }

        response.json::<Firmware>().await.map_err(Into::into)
    }

    /// Download firmware binary
    pub async fn download_firmware(&self, firmware_id: &str) -> Result<Vec<u8>> {
        let url = format!("{}/api/firmwares/{}/download", self.server_url, firmware_id);

        tracing::info!("Downloading firmware from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            tracing::error!("Failed to download firmware: {}", response.status());
            return Err(OrchestratorError::FirmwareDownloadFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(Into::into)
    }

    /// List all firmwares with optional filtering
    pub async fn list_firmwares(&self, query: FirmwareQuery) -> Result<Vec<Firmware>> {
        let mut url = format!("{}/api/firmwares", self.server_url);

        let mut params = vec![];
        if let Some(device) = &query.device {
            params.push(format!("device={}", device.as_str()));
        }
        if let Some(version) = &query.version {
            params.push(format!("version={}", version));
        }
        if let Some(latest_only) = query.latest_only {
            params.push(format!("latest_only={}", latest_only));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::info!("Listing firmwares from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            tracing::error!("Failed to list firmwares: {}", response.status());
            return Err(OrchestratorError::FirmwareServerUnavailable);
        }

        response.json::<Vec<Firmware>>().await.map_err(Into::into)
    }

    /// Get firmware metadata by ID
    pub async fn get_firmware_by_id(&self, id: &str) -> Result<Firmware> {
        let url = format!("{}/api/firmwares/{}", self.server_url, id);

        tracing::info!("Fetching firmware metadata from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            tracing::error!("Failed to fetch firmware metadata: {}", response.status());
            return Err(if response.status() == reqwest::StatusCode::NOT_FOUND {
                OrchestratorError::FirmwareNotFound(id.to_string())
            } else {
                OrchestratorError::FirmwareServerUnavailable
            });
        }

        response.json::<Firmware>().await.map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct AppState {
    pub firmware_client: Arc<FirmwareClient>,
}

/// Handler: GET /firmwares/latest/:device
/// Returns the latest firmware metadata for a specific device
pub async fn get_latest_firmware_handler(
    State(state): State<AppState>,
    Path(device_str): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("GET /firmwares/latest/{}", device_str);

    let device = device_str.parse::<FirmwareDevice>().map_err(|_| {
        OrchestratorError::ValidationError(format!("Invalid device type: {}", device_str))
    })?;

    let firmware = state.firmware_client.get_latest_firmware(device).await?;
    Ok((StatusCode::OK, Json(serde_json::json!(firmware))))
}

/// Handler: GET /firmwares?device=esp32c3&latest_only=true
/// Lists firmwares with optional filtering
pub async fn list_firmwares_handler(
    State(state): State<AppState>,
    Query(query): Query<FirmwareQuery>,
) -> Result<impl IntoResponse> {
    tracing::info!("GET /firmwares - query: {:?}", query);

    let firmwares = state.firmware_client.list_firmwares(query).await?;
    Ok((StatusCode::OK, Json(serde_json::json!(firmwares))))
}

/// Handler: GET /firmwares/:id
/// Gets firmware metadata by ID
pub async fn get_firmware_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("GET /firmwares/{}", id);

    let firmware = state.firmware_client.get_firmware_by_id(&id).await?;
    Ok((StatusCode::OK, Json(serde_json::json!(firmware))))
}

/// Handler: GET /firmwares/:id/download
/// Downloads firmware binary and returns it
pub async fn download_firmware_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("GET /firmwares/{}/download", id);

    // First get firmware metadata
    let firmware = state.firmware_client.get_firmware_by_id(&id).await?;

    // Then download the binary
    let data = state.firmware_client.download_firmware(&id).await?;

    use axum::http::header::{self, HeaderName};
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", firmware.file_path)
            .parse()
            .unwrap(),
    );
    headers.insert(
        HeaderName::from_static("x-firmware-version"),
        firmware.version.parse().unwrap(),
    );
    headers.insert(
        HeaderName::from_static("x-firmware-checksum"),
        firmware.checksum.parse().unwrap(),
    );
    headers.insert(
        HeaderName::from_static("x-firmware-device"),
        firmware.device.as_str().parse().unwrap(),
    );

    Ok((headers, data))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrchestratorInfo {
    pub version: String,
    pub server_url: String,
    pub status: String,
}

/// Handler: GET /health
/// Returns orchestrator health status
pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("GET /health");

    let info = OrchestratorInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        server_url: state.firmware_client.server_url.clone(),
        status: "healthy".to_string(),
    };

    (StatusCode::OK, Json(serde_json::json!(info)))
}
