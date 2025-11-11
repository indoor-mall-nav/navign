use crate::AppState;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;
use navign_shared::{Firmware, FirmwareDevice, FirmwareQuery, FirmwareUploadResponse};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

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
    pool: &PgPool,
    device: FirmwareDevice,
) -> Result<Option<Firmware>, sqlx::Error> {
    let device_str = device.as_str();

    let firmware = sqlx::query_as!(
        Firmware,
        r#"
        SELECT
            id,
            version,
            device as "device: FirmwareDevice",
            description,
            file_path,
            file_size,
            checksum,
            is_latest,
            git_commit,
            build_time,
            created_at,
            release_notes
        FROM firmwares
        WHERE device = $1 AND is_latest = true
        LIMIT 1
        "#,
        device_str
    )
    .fetch_optional(pool)
    .await?;

    Ok(firmware)
}

/// Get firmware by ID
pub async fn get_firmware_by_id(
    pool: &PgPool,
    id: &str,
) -> Result<Option<Firmware>, sqlx::Error> {
    let uuid = Uuid::parse_str(id)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    let firmware = sqlx::query_as!(
        Firmware,
        r#"
        SELECT
            id,
            version,
            device as "device: FirmwareDevice",
            description,
            file_path,
            file_size,
            checksum,
            is_latest,
            git_commit,
            build_time,
            created_at,
            release_notes
        FROM firmwares
        WHERE id = $1
        "#,
        uuid
    )
    .fetch_optional(pool)
    .await?;

    Ok(firmware)
}

/// Get firmware by version and device
pub async fn get_firmware_by_version(
    pool: &PgPool,
    version: &str,
    device: FirmwareDevice,
) -> Result<Option<Firmware>, sqlx::Error> {
    let device_str = device.as_str();

    let firmware = sqlx::query_as!(
        Firmware,
        r#"
        SELECT
            id,
            version,
            device as "device: FirmwareDevice",
            description,
            file_path,
            file_size,
            checksum,
            is_latest,
            git_commit,
            build_time,
            created_at,
            release_notes
        FROM firmwares
        WHERE version = $1 AND device = $2
        "#,
        version,
        device_str
    )
    .fetch_optional(pool)
    .await?;

    Ok(firmware)
}

/// List all firmwares with optional filtering
pub async fn list_firmwares(
    pool: &PgPool,
    query: FirmwareQuery,
) -> Result<Vec<Firmware>, sqlx::Error> {
    // Build WHERE clauses
    let mut conditions = Vec::new();
    let mut sql = String::from(
        r#"
        SELECT
            id,
            version,
            device as "device: FirmwareDevice",
            description,
            file_path,
            file_size,
            checksum,
            is_latest,
            git_commit,
            build_time,
            created_at,
            release_notes
        FROM firmwares
        "#
    );

    if query.device.is_some() || query.version.is_some() || query.latest_only.is_some() {
        sql.push_str("WHERE ");
    }

    if query.device.is_some() {
        conditions.push("device = $1");
    }
    if query.version.is_some() {
        let idx = if conditions.is_empty() { 1 } else { 2 };
        conditions.push(&format!("version = ${}", idx));
    }
    if let Some(true) = query.latest_only {
        conditions.push("is_latest = true");
    }

    sql.push_str(&conditions.join(" AND "));
    sql.push_str(" ORDER BY created_at DESC");

    // Execute query based on parameters
    let firmwares = match (&query.device, &query.version) {
        (Some(device), Some(version)) => {
            sqlx::query_as::<_, Firmware>(&sql)
                .bind(device.as_str())
                .bind(version)
                .fetch_all(pool)
                .await?
        }
        (Some(device), None) => {
            sqlx::query_as::<_, Firmware>(&sql)
                .bind(device.as_str())
                .fetch_all(pool)
                .await?
        }
        (None, Some(version)) => {
            sqlx::query_as::<_, Firmware>(&sql)
                .bind(version)
                .fetch_all(pool)
                .await?
        }
        (None, None) => {
            sqlx::query_as::<_, Firmware>(&sql)
                .fetch_all(pool)
                .await?
        }
    };

    Ok(firmwares)
}

/// Mark a firmware as latest and unmark previous ones
async fn mark_as_latest(
    pool: &PgPool,
    firmware_id: Uuid,
    device: FirmwareDevice,
) -> Result<(), sqlx::Error> {
    let device_str = device.as_str();

    // Use a transaction to ensure atomicity
    let mut tx = pool.begin().await?;

    // Unmark all previous firmwares for this device
    sqlx::query!(
        r#"
        UPDATE firmwares
        SET is_latest = false
        WHERE device = $1 AND is_latest = true
        "#,
        device_str
    )
    .execute(&mut *tx)
    .await?;

    // Mark the new firmware as latest
    sqlx::query!(
        r#"
        UPDATE firmwares
        SET is_latest = true
        WHERE id = $1
        "#,
        firmware_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

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
            log::error!("Failed to list firmwares: {}", e);
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
            log::error!("Failed to get firmware: {}", e);
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
            log::error!("Failed to get latest firmware: {}", e);
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
            log::error!("Failed to create storage directory: {}", e);
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
                log::error!("Failed to write firmware file: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({ "error": "Failed to write firmware file" })),
                );
            }
        }
        Err(e) => {
            log::error!("Failed to create firmware file: {}", e);
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
            log::error!("Failed to calculate checksum: {}", e);
            // Clean up file
            let _ = fs::remove_file(&file_path).await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to calculate checksum" })),
            );
        }
    };

    // Generate new UUID for firmware
    let firmware_id = Uuid::new_v4();
    let device_str = device.as_str();
    let file_size = file_data.len() as i64;
    let build_time = chrono::Utc::now().timestamp_millis();
    let created_at = chrono::Utc::now().timestamp_millis();

    // Save to database
    let insert_result = sqlx::query!(
        r#"
        INSERT INTO firmwares (
            id, version, device, description, file_path, file_size, checksum,
            is_latest, git_commit, build_time, created_at, release_notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        firmware_id,
        version,
        device_str,
        description,
        filename,
        file_size,
        checksum,
        mark_latest,
        git_commit,
        build_time,
        created_at,
        release_notes
    )
    .execute(&state.db)
    .await;

    if let Err(e) = insert_result {
        log::error!("Failed to insert firmware into database: {}", e);
        // Clean up file
        let _ = fs::remove_file(&file_path).await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({ "error": "Failed to save firmware to database" })),
        );
    }

    // Mark as latest if requested
    if mark_latest && let Err(e) = mark_as_latest(&state.db, firmware_id, device.clone()).await {
        log::error!("Failed to mark firmware as latest: {}", e);
        // Don't fail the upload, just log the error
    }

    let response = FirmwareUploadResponse {
        id: firmware_id.to_string(),
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
            log::error!("Failed to get firmware: {}", e);
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
            log::error!("Failed to read firmware file: {}", e);
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
            log::error!("Failed to get firmware: {}", e);
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
        log::warn!("Failed to delete firmware file: {}", e);
        // Continue with database deletion even if file deletion fails
    }

    // Delete from database
    let uuid = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "error": format!("Invalid UUID: {}", e) })),
            );
        }
    };

    match sqlx::query!(
        r#"
        DELETE FROM firmwares
        WHERE id = $1
        "#,
        uuid
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => (StatusCode::OK, axum::Json(json!({ "status": "deleted" }))),
        Err(e) => {
            log::error!("Failed to delete firmware from database: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": "Failed to delete firmware" })),
            )
        }
    }
}
