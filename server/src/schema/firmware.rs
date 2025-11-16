use crate::AppState;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bson::doc;
use bson::oid::ObjectId;
use futures::stream::TryStreamExt;
use mongodb::{Collection, Database};
use navign_shared::{Firmware, FirmwareDevice, FirmwareQuery, FirmwareUploadResponse};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::info;

/// Get the firmware storage directory
fn get_firmware_storage_dir() -> PathBuf {
    std::env::var("FIRMWARE_STORAGE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./firmware_storage"))
}

/// Ensure firmware storage directory exists
async fn ensure_storage_dir() -> Result<PathBuf, std::io::Error> {
    let dir = get_firmware_storage_dir();
    fs::create_dir_all(&dir).await?;
    Ok(dir)
}

/// Calculate SHA256 checksum of a file
async fn calculate_checksum(file_path: &PathBuf) -> Result<String, std::io::Error> {
    let contents = fs::read(file_path).await?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Get the latest firmware for a specific device
pub async fn get_latest_firmware(
    db: &Database,
    device: FirmwareDevice,
) -> Result<Option<Firmware>, mongodb::error::Error> {
    let collection: Collection<Firmware> = db.collection("firmwares");

    let filter = doc! {
        "device": device.as_str(),
        "is_latest": true,
    };

    collection.find_one(filter).await
}

/// Get firmware by ID
pub async fn get_firmware_by_id(
    db: &Database,
    id: &str,
) -> Result<Option<Firmware>, mongodb::error::Error> {
    let collection: Collection<Firmware> = db.collection("firmwares");
    let oid = ObjectId::parse_str(id)
        .map_err(|e| mongodb::error::Error::custom(format!("Invalid ObjectId: {}", e)))?;

    collection.find_one(doc! { "_id": oid }).await
}

/// Get firmware by version and device
pub async fn get_firmware_by_version(
    db: &Database,
    version: &str,
    device: FirmwareDevice,
) -> Result<Option<Firmware>, mongodb::error::Error> {
    let collection: Collection<Firmware> = db.collection("firmwares");

    let filter = doc! {
        "version": version,
        "device": device.as_str(),
    };

    collection.find_one(filter).await
}

/// List all firmwares with optional filtering
pub async fn list_firmwares(
    db: &Database,
    query: FirmwareQuery,
) -> Result<Vec<Firmware>, mongodb::error::Error> {
    let collection: Collection<Firmware> = db.collection("firmwares");

    let mut filter = doc! {};

    if let Some(device) = &query.device {
        filter.insert("device", device.as_str());
    }

    if let Some(version) = &query.version {
        filter.insert("version", version);
    }

    if let Some(true) = query.latest_only {
        filter.insert("is_latest", true);
    }

    let options = mongodb::options::FindOptions::builder()
        .sort(doc! { "created_at": -1 })
        .build();

    let cursor = collection.find(filter).with_options(options).await?;
    cursor.try_collect::<Vec<Firmware>>().await
}

/// Mark a firmware as latest and unmark previous ones
async fn mark_as_latest(
    db: &Database,
    firmware_id: ObjectId,
    device: FirmwareDevice,
) -> Result<(), mongodb::error::Error> {
    let collection: Collection<Firmware> = db.collection("firmwares");

    // Unmark all previous firmwares for this device
    collection
        .update_many(
            doc! {
                "device": device.as_str(),
                "is_latest": true,
            },
            doc! {
                "$set": { "is_latest": false }
            },
        )
        .await?;

    // Mark the new firmware as latest
    collection
        .update_one(
            doc! { "_id": firmware_id },
            doc! {
                "$set": { "is_latest": true }
            },
        )
        .await?;

    Ok(())
}

/// Handler: GET /api/firmwares?device=esp32c3&latest_only=true
pub async fn get_firmwares_handler(
    State(state): State<AppState>,
    Query(query): Query<FirmwareQuery>,
) -> impl IntoResponse {
    info!("GET /api/firmwares - query: {:?}", query);

    match list_firmwares(&state.db, query).await {
        Ok(firmwares) => (StatusCode::OK, axum::Json(json!(firmwares))),
        Err(e) => {
            tracing::error!("Failed to list firmwares: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "error": "Failed to list firmwares",
                    "details": e.to_string()
                })),
            )
        }
    }
}

/// Handler: GET /api/firmwares/:id
pub async fn get_firmware_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("GET /api/firmwares/{}", id);

    match get_firmware_by_id(&state.db, &id).await {
        Ok(Some(firmware)) => (StatusCode::OK, axum::Json(json!(firmware))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            axum::Json(json!({
                "error": "Firmware not found"
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to get firmware: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "error": "Failed to get firmware",
                    "details": e.to_string()
                })),
            )
        }
    }
}

/// Handler: GET /api/firmwares/latest/:device
pub async fn get_latest_firmware_handler(
    State(state): State<AppState>,
    Path(device_str): Path<String>,
) -> impl IntoResponse {
    info!("GET /api/firmwares/latest/{}", device_str);

    let device = match device_str.parse::<FirmwareDevice>() {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({
                    "error": "Invalid device type",
                    "valid_devices": ["esp32", "esp32c3", "esp32c5", "esp32c6", "esp32s3"]
                })),
            );
        }
    };

    match get_latest_firmware(&state.db, device).await {
        Ok(Some(firmware)) => (StatusCode::OK, axum::Json(json!(firmware))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            axum::Json(json!({
                "error": "No firmware found for this device"
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to get latest firmware: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "error": "Failed to get latest firmware",
                    "details": e.to_string()
                })),
            )
        }
    }
}

/// Handler: POST /api/firmwares/upload
/// Multipart form with fields: version, device, description, file, git_commit, release_notes, mark_latest
pub async fn upload_firmware_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    info!("POST /api/firmwares/upload");

    let mut version: Option<String> = None;
    let mut device: Option<FirmwareDevice> = None;
    let mut description: Option<String> = None;
    let mut git_commit: Option<String> = None;
    let mut release_notes: Option<String> = None;
    let mut mark_latest = false;
    let mut file_data: Option<Vec<u8>> = None;
    let mut original_filename: Option<String> = None;

    // Parse multipart form
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "version" => version = Some(field.text().await.unwrap_or_default()),
            "device" => {
                let device_str = field.text().await.unwrap_or_default();
                device = device_str.parse::<FirmwareDevice>().ok();
            }
            "description" => {
                let text = field.text().await.unwrap_or_default();
                if !text.is_empty() {
                    description = Some(text);
                }
            }
            "git_commit" => {
                let text = field.text().await.unwrap_or_default();
                if !text.is_empty() {
                    git_commit = Some(text);
                }
            }
            "release_notes" => {
                let text = field.text().await.unwrap_or_default();
                if !text.is_empty() {
                    release_notes = Some(text);
                }
            }
            "mark_latest" => {
                mark_latest = field.text().await.unwrap_or_default() == "true";
            }
            "file" => {
                original_filename = field.file_name().map(|s| s.to_string());
                file_data = Some(field.bytes().await.unwrap_or_default().to_vec());
            }
            _ => {}
        }
    }

    // Validate required fields
    let version = match version {
        Some(v) => v,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "error": "Missing required field: version" })),
            );
        }
    };

    let device = match device {
        Some(d) => d,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "error": "Missing or invalid required field: device" })),
            );
        }
    };

    let file_data = match file_data {
        Some(data) if !data.is_empty() => data,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "error": "Missing or empty firmware file" })),
            );
        }
    };

    // Ensure storage directory exists
    let storage_dir = match ensure_storage_dir().await {
        Ok(dir) => dir,
        Err(e) => {
            tracing::error!("Failed to create storage directory: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to create storage directory" })),
            );
        }
    };

    // Generate unique filename
    let filename = format!(
        "{}-{}-{}.bin",
        device.as_str(),
        version,
        chrono::Utc::now().timestamp()
    );
    let file_path = storage_dir.join(&filename);

    // Write file to disk
    match fs::File::create(&file_path).await {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&file_data).await {
                tracing::error!("Failed to write firmware file: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({ "error": "Failed to write firmware file" })),
                );
            }
        }
        Err(e) => {
            tracing::error!("Failed to create firmware file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to create firmware file" })),
            );
        }
    }

    // Calculate checksum
    let checksum = match calculate_checksum(&file_path).await {
        Ok(sum) => sum,
        Err(e) => {
            tracing::error!("Failed to calculate checksum: {}", e);
            // Clean up file
            let _ = fs::remove_file(&file_path).await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to calculate checksum" })),
            );
        }
    };

    // Create firmware document
    let firmware = Firmware {
        id: ObjectId::new().to_hex(),
        version: version.clone(),
        device: device.clone(),
        description,
        file_path: filename.clone(),
        file_size: file_data.len() as u64,
        checksum: checksum.clone(),
        is_latest: mark_latest,
        git_commit,
        build_time: chrono::Utc::now().timestamp_millis(),
        created_at: chrono::Utc::now().timestamp_millis(),
        release_notes,
    };

    // Save to database
    let collection: Collection<Firmware> = state.db.collection("firmwares");
    let insert_result = match collection.insert_one(&firmware).await {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Failed to insert firmware into database: {}", e);
            // Clean up file
            let _ = fs::remove_file(&file_path).await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to save firmware to database" })),
            );
        }
    };

    let firmware_id = match insert_result.inserted_id.as_object_id() {
        Some(id) => id,
        None => {
            tracing::error!("Failed to get inserted firmware ID");
            let _ = fs::remove_file(&file_path).await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to get inserted firmware ID" })),
            );
        }
    };

    // Mark as latest if requested
    if mark_latest && let Err(e) = mark_as_latest(&state.db, firmware_id, device.clone()).await {
        tracing::error!("Failed to mark firmware as latest: {}", e);
        // Don't fail the upload, just log the error
    }

    let response = FirmwareUploadResponse {
        id: firmware_id.to_hex(),
        version,
        device,
        file_size: file_data.len() as u64,
        checksum,
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    (StatusCode::CREATED, axum::Json(json!(response)))
}

/// Handler: GET /api/firmwares/:id/download
pub async fn download_firmware_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("GET /api/firmwares/{}/download", id);

    let firmware = match get_firmware_by_id(&state.db, &id).await {
        Ok(Some(fw)) => fw,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                axum::Json(json!({ "error": "Firmware not found" })),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get firmware: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to get firmware" })),
            ));
        }
    };

    let storage_dir = get_firmware_storage_dir();
    let file_path = storage_dir.join(&firmware.file_path);

    match fs::read(&file_path).await {
        Ok(contents) => {
            use axum::http::header::{self, HeaderName, HeaderValue};
            let mut headers = axum::http::HeaderMap::new();

            // Safe: These are constant strings
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );

            // Parse header values with error handling
            if let Ok(disposition) =
                format!("attachment; filename=\"{}\"", firmware.file_path).parse()
            {
                headers.insert(header::CONTENT_DISPOSITION, disposition);
            }

            if let Ok(version) = firmware.version.parse() {
                headers.insert(HeaderName::from_static("x-firmware-version"), version);
            }

            if let Ok(checksum) = firmware.checksum.parse() {
                headers.insert(HeaderName::from_static("x-firmware-checksum"), checksum);
            }

            Ok((headers, contents))
        }
        Err(e) => {
            tracing::error!("Failed to read firmware file: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to read firmware file" })),
            ))
        }
    }
}

/// Handler: DELETE /api/firmwares/:id
pub async fn delete_firmware_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("DELETE /api/firmwares/{}", id);

    let firmware = match get_firmware_by_id(&state.db, &id).await {
        Ok(Some(fw)) => fw,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                axum::Json(json!({ "error": "Firmware not found" })),
            );
        }
        Err(e) => {
            tracing::error!("Failed to get firmware: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to get firmware" })),
            );
        }
    };

    // Delete file from disk
    let storage_dir = get_firmware_storage_dir();
    let file_path = storage_dir.join(&firmware.file_path);
    if let Err(e) = fs::remove_file(&file_path).await {
        tracing::warn!("Failed to delete firmware file: {}", e);
        // Continue with database deletion even if file deletion fails
    }

    // Delete from database
    let collection: Collection<Firmware> = state.db.collection("firmwares");
    let oid = match ObjectId::parse_str(&id) {
        Ok(oid) => oid,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "error": format!("Invalid ObjectId: {}", e) })),
            );
        }
    };

    match collection.delete_one(doc! { "_id": oid }).await {
        Ok(_) => (StatusCode::OK, axum::Json(json!({ "status": "deleted" }))),
        Err(e) => {
            tracing::error!("Failed to delete firmware from database: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to delete firmware" })),
            )
        }
    }
}
