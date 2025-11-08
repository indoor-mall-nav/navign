use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
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
    pub async fn get_latest_firmware(
        &self,
        device: FirmwareDevice,
    ) -> Result<Firmware, reqwest::Error> {
        let url = format!(
            "{}/api/firmwares/latest/{}",
            self.server_url,
            device.as_str()
        );

        log::info!("Fetching latest firmware from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            log::error!("Failed to fetch firmware: {}", response.status());
            return Err(response.error_for_status().unwrap_err());
        }

        response.json::<Firmware>().await
    }

    /// Download firmware binary
    pub async fn download_firmware(&self, firmware_id: &str) -> Result<Vec<u8>, reqwest::Error> {
        let url = format!("{}/api/firmwares/{}/download", self.server_url, firmware_id);

        log::info!("Downloading firmware from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            log::error!("Failed to download firmware: {}", response.status());
            return Err(response.error_for_status().unwrap_err());
        }

        response.bytes().await.map(|b| b.to_vec())
    }

    /// List all firmwares with optional filtering
    pub async fn list_firmwares(
        &self,
        query: FirmwareQuery,
    ) -> Result<Vec<Firmware>, reqwest::Error> {
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

        log::info!("Listing firmwares from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            log::error!("Failed to list firmwares: {}", response.status());
            return Err(response.error_for_status().unwrap_err());
        }

        response.json::<Vec<Firmware>>().await
    }

    /// Get firmware metadata by ID
    pub async fn get_firmware_by_id(&self, id: &str) -> Result<Firmware, reqwest::Error> {
        let url = format!("{}/api/firmwares/{}", self.server_url, id);

        log::info!("Fetching firmware metadata from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            log::error!("Failed to fetch firmware metadata: {}", response.status());
            return Err(response.error_for_status().unwrap_err());
        }

        response.json::<Firmware>().await
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
) -> impl IntoResponse {
    log::info!("GET /firmwares/latest/{}", device_str);

    let device = match FirmwareDevice::from_str(&device_str) {
        Some(d) => d,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid device type",
                    "valid_devices": ["esp32", "esp32c3", "esp32c5", "esp32c6", "esp32s3"]
                })),
            )
        }
    };

    match state.firmware_client.get_latest_firmware(device).await {
        Ok(firmware) => (StatusCode::OK, Json(serde_json::json!(firmware))),
        Err(e) => {
            log::error!("Failed to fetch latest firmware: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch latest firmware",
                    "details": e.to_string()
                })),
            )
        }
    }
}

/// Handler: GET /firmwares?device=esp32c3&latest_only=true
/// Lists firmwares with optional filtering
pub async fn list_firmwares_handler(
    State(state): State<AppState>,
    Query(query): Query<FirmwareQuery>,
) -> impl IntoResponse {
    log::info!("GET /firmwares - query: {:?}", query);

    match state.firmware_client.list_firmwares(query).await {
        Ok(firmwares) => (StatusCode::OK, Json(serde_json::json!(firmwares))),
        Err(e) => {
            log::error!("Failed to list firmwares: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to list firmwares",
                    "details": e.to_string()
                })),
            )
        }
    }
}

/// Handler: GET /firmwares/:id
/// Gets firmware metadata by ID
pub async fn get_firmware_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    log::info!("GET /firmwares/{}", id);

    match state.firmware_client.get_firmware_by_id(&id).await {
        Ok(firmware) => (StatusCode::OK, Json(serde_json::json!(firmware))),
        Err(e) => {
            log::error!("Failed to get firmware: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get firmware",
                    "details": e.to_string()
                })),
            )
        }
    }
}

/// Handler: GET /firmwares/:id/download
/// Downloads firmware binary and returns it
pub async fn download_firmware_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    log::info!("GET /firmwares/{}/download", id);

    // First get firmware metadata
    let firmware = match state.firmware_client.get_firmware_by_id(&id).await {
        Ok(fw) => fw,
        Err(e) => {
            log::error!("Failed to get firmware metadata: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get firmware metadata",
                    "details": e.to_string()
                })),
            ));
        }
    };

    // Then download the binary
    match state.firmware_client.download_firmware(&id).await {
        Ok(data) => {
            let headers = [
                ("Content-Type", "application/octet-stream"),
                (
                    "Content-Disposition",
                    &format!("attachment; filename=\"{}\"", firmware.file_path),
                ),
                ("X-Firmware-Version", &firmware.version),
                ("X-Firmware-Checksum", &firmware.checksum),
                ("X-Firmware-Device", firmware.device.as_str()),
            ];

            Ok((headers, data))
        }
        Err(e) => {
            log::error!("Failed to download firmware: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to download firmware",
                    "details": e.to_string()
                })),
            ))
        }
    }
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
    log::debug!("GET /health");

    let info = OrchestratorInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        server_url: state.firmware_client.server_url.clone(),
        status: "healthy".to_string(),
    };

    (StatusCode::OK, Json(serde_json::json!(info)))
}
