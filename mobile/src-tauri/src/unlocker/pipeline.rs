use crate::locate::beacon::BeaconInfo;
use crate::locate::fetch_device;
use crate::locate::scan::{scan_devices, stop_scan};
use crate::shared::BASE_URL;
use crate::unlocker::Unlocker;
use crate::unlocker::challenge::ServerChallenge;
use crate::unlocker::constants::{UNLOCKER_CHARACTERISTIC_UUID, UNLOCKER_SERVICE_UUID};
use crate::unlocker::proof::Proof;
use crate::unlocker::utils::{BleMessage, DeviceCapability};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use std::str::FromStr;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
#[cfg(mobile)]
use tauri_plugin_biometric::AuthOptions;
#[cfg(mobile)]
use tauri_plugin_biometric::BiometricExt;
use tauri_plugin_blec::models::WriteType;
use tauri_plugin_blec::{OnDisconnectHandler, get_handler};
use tauri_plugin_http::reqwest;
use tauri_plugin_log::log::{error, info};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStageResult {
    pub instance_id: String,
    pub challenge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockResult {
    success: bool,
    outcome: String,
}

pub async fn unlock_pipeline(
    handle: AppHandle,
    entity: String,
    target: String,
    state: State<'_, Arc<Mutex<Unlocker>>>,
) -> anyhow::Result<String> {
    info!(
        "Starting unlock pipeline for target: {} in {}",
        target, entity
    );
    let app_state = state.lock().await;
    let dbpath = handle
        .path()
        .app_local_data_dir()
        .map(|p| p.join("navign.db"))
        .map_err(|e| {
            error!("Failed to get app local data dir: {}", e);
            anyhow::anyhow!("Failed to get app local data dir")
        })?;
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(dbpath.parent().unwrap()).map_err(|e| {
        error!("Failed to create app local data dir: {}", e);
        anyhow::anyhow!("Failed to create app local data dir")
    })?;
    let db_str = format!("{}", dbpath.to_string_lossy());
    let conn = SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new()
            .filename(db_str.as_str())
            .create_if_missing(true),
    )
    .await
    .map_err(|e| {
        error!("Failed to connect to database: {}", e);
        anyhow::anyhow!("Failed to connect to database")
    })?;
    let devices = scan_devices(true)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to scan devices: {}", e))?;
    stop_scan()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to stop scan: {}", e))?;
    let mut result_address = None;
    for device in devices.iter() {
        let device_id = fetch_device(&conn, device.address.as_str(), entity.as_str()).await?;
        let device_info = BeaconInfo::get_from_id(&conn, device_id.as_str())
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("Beacon info not found for device"))?;
        info!(
            "Scanned device: {} ({}) - Merchant: {}",
            device_info.id.as_str(),
            device.address.as_str(),
            device_info.merchant.as_str()
        );
        // FIXME merchant not loaded properly from DB?
        if device_info.merchant == target || device_info.merchant == "unknown" {
            info!(
                "Found target device: {} ({})",
                device_info.id.as_str(),
                device.address.as_str()
            );
            result_address = Some(device.address.clone());
            break;
        }
    }
    let target_addr =
        result_address.ok_or_else(|| anyhow::anyhow!("Target device not found during scan"))?;
    info!("Target device address found: {}", target_addr);
    let handler = get_handler().map_err(|e| anyhow::anyhow!("Failed to get BLE handler: {}", e))?;

    info!("Connecting to device: {}", target_addr);

    handler.disconnect().await.ok();
    handler.stop_scan().await.ok();

    handler
        .connect(target_addr.as_str(), OnDisconnectHandler::None, true)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to device {}: {}", target_addr, e))?;

    info!("Connected to device: {}", target_addr);

    handler
        .subscribe(
            Uuid::from_str(UNLOCKER_CHARACTERISTIC_UUID)?,
            Some(Uuid::from_str(UNLOCKER_SERVICE_UUID)?),
            |data| {
                info!("Notification received: {:x?}", data);
            },
        )
        .await?;

    let characteristic = Uuid::from_str(UNLOCKER_CHARACTERISTIC_UUID)?;
    let service = Uuid::from_str(UNLOCKER_SERVICE_UUID)?;

    handler
        .send_data(
            characteristic,
            Some(service),
            &BleMessage::DeviceRequest.packetize(),
            WriteType::WithResponse,
        )
        .await?;

    let received = handler.recv_data(characteristic, Some(service)).await?;
    let depacketized = BleMessage::depacketize(received.as_slice())
        .ok_or_else(|| anyhow::anyhow!("Failed to depacketize device response"))?;

    let BleMessage::DeviceResponse(d_type, d_capabilities, obj_id) = depacketized else {
        return Err(anyhow::anyhow!("Failed to extract device response"));
    };

    let object_id = String::from_utf8(obj_id.as_slice().to_vec())
        .map_err(|_| anyhow::anyhow!("Invalid object ID encoding"))?;

    info!("Object ID: {}", object_id);

    if object_id.len() != 24 {
        return Err(anyhow::anyhow!("Invalid object ID length"));
    }

    info!(
        "Device Type: {:?}, Capabilities: {:?}, Object ID: {}",
        d_type, d_capabilities, object_id
    );

    if !d_capabilities.contains(&DeviceCapability::UnlockGate) {
        return Err(anyhow::anyhow!("Device does not support unlocking"));
    }

    // Step 2: get the nonce
    handler
        .send_data(
            characteristic,
            Some(service),
            &BleMessage::NonceRequest.packetize(),
            WriteType::WithResponse,
        )
        .await?;
    let received = handler.recv_data(characteristic, Some(service)).await?;

    let nonce_packet = BleMessage::depacketize(received.as_slice())
        .ok_or_else(|| anyhow::anyhow!("Failed to depacketize nonce"))?;

    // Step 3: construct the unlock request
    let BleMessage::NonceResponse(nonce, verification) = nonce_packet else {
        return Err(anyhow::anyhow!("Failed to extract nonce"));
    };

    info!("Nonce: {:x?}", nonce);
    info!("Verification: {:x?}", verification);

    let mut payload = [0u8; 24];
    payload.copy_from_slice(nonce.as_slice());
    payload[16..24].copy_from_slice(&verification[0..8]);
    let encoded = base64::engine::general_purpose::STANDARD.encode(payload);
    info!("Payload: {}", encoded);

    let client = reqwest::Client::new();
    let instance = client
        .post(
            BASE_URL.to_string()
                + "api/entities/"
                + entity.as_str()
                + "/beacons/"
                + object_id.as_str()
                + "/unlocker",
        )
        .bearer_auth(app_state.user_id.as_str())
        .header("Content-Type", "application/json")
        .body(json!({ "payload": encoded }).to_string())
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
        .json::<CreateStageResult>()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get response text: {}", e))?;

    let current = chrono::Utc::now().timestamp() as u64;
    let nonce = hex::decode(instance.challenge.as_str())
        .map_err(|e| anyhow::anyhow!("Failed to decode challenge nonce: {}", e))?;
    let mut nonce_buffer = [0u8; 16];
    nonce_buffer.copy_from_slice(&nonce);
    let challenge = ServerChallenge::new(
        nonce_buffer,
        instance.instance_id.as_str(),
        current,
        app_state.user_id.clone(),
    );
    info!("Challenge: {:?}", challenge);
    let device_key = app_state
        .ensure_signing_key(&handle)
        .map_err(|e| anyhow::anyhow!("Failed to get device key: {}", e))?;

    #[cfg(mobile)]
    let auth_options = AuthOptions {
        allow_device_credential: true,
        cancel_title: Some("Cancel".to_string()),
        fallback_title: Some("Use Passcode".to_string()),
        title: Some("Authenticate to unlock".to_string()),
        subtitle: Some("Please authenticate to proceed".to_string()),
        confirmation_required: Some(true),
    };

    #[cfg(mobile)]
    handle
        .biometric()
        .authenticate("Please authenticate to unlock".to_string(), auth_options)
        .map_err(|_| anyhow::anyhow!("Biometric authentication failed"))?;

    let (challenge_packet, validator) = challenge.packetize(&device_key);

    info!("Challenge Packet: {}", challenge_packet);

    let client_response = client
        .put(
            BASE_URL.to_string()
                + "api/entities/"
                + entity.as_str()
                + "/beacons/"
                + object_id.as_str()
                + "/unlocker/"
                + instance.instance_id.as_str()
                + "/status",
        )
        .bearer_auth(app_state.user_id.as_str())
        .header("Content-Type", "application/json")
        .body(json!({ "payload": challenge_packet }).to_string())
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("HTTP error: {}", e))?
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get response text: {}", e))?;

    let server_proof = base64::engine::general_purpose::STANDARD
        .decode(client_response)
        .map_err(|e| anyhow::anyhow!("Failed to decode server response: {}", e))?;

    if server_proof.len() != 72 {
        return Err(anyhow::anyhow!("Invalid server proof length"));
    }

    let mut server_signature = [0u8; 64];
    server_signature.copy_from_slice(&server_proof[0..64]);
    let mut beacon_verifier = [0u8; 8];
    beacon_verifier.copy_from_slice(&server_proof[64..72]);

    info!("Server Signature: {:x?}", server_signature);
    info!("Beacon Verifier: {:x?}", beacon_verifier);

    // Step 4: send the unlock request
    let proof = Proof::new(
        nonce_buffer,
        validator,
        beacon_verifier,
        current,
        server_signature,
    );

    let proof_packet = BleMessage::UnlockRequest(proof).packetize();

    info!("Proof Packet: {:x?}", proof_packet);

    handler
        .send_data(
            characteristic,
            Some(service),
            &proof_packet,
            WriteType::WithResponse,
        )
        .await?;
    let received = handler.recv_data(characteristic, Some(service)).await?;

    let depacketized = BleMessage::depacketize(received.as_slice())
        .ok_or_else(|| anyhow::anyhow!("Failed to depacketize unlock response"))?;

    let BleMessage::UnlockResponse(success, error) = depacketized else {
        return Err(anyhow::anyhow!("Failed to extract unlock response"));
    };

    // Step 5: report the result
    let result = UnlockResult {
        success,
        outcome: error.to_string(),
    };
    info!("Unlock Result: {:?}", result);

    let eventual = client
        .put(
            BASE_URL.to_string()
                + "api/entities/"
                + entity.as_str()
                + "/beacons/"
                + object_id.as_str()
                + "/unlocker/"
                + instance.instance_id.as_str()
                + "/outcome",
        )
        .bearer_auth(app_state.user_id.as_str())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&result)?)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get response text: {}", e))?;

    handler
        .unsubscribe(Uuid::from_str(UNLOCKER_CHARACTERISTIC_UUID)?)
        .await?;
    handler.disconnect().await?;

    if eventual == "Unlock result recorded" {
        Ok("Unlock successful".to_string())
    } else {
        Err(anyhow::anyhow!("Failed to record unlock result"))
    }
}

#[tauri::command]
pub async fn unlock_handler(
    handle: AppHandle,
    entity: String,
    target: String,
    state: State<'_, Arc<Mutex<Unlocker>>>,
) -> Result<String, ()> {
    match unlock_pipeline(handle, entity, target, state).await {
        Ok(res) => {
            let payload = json!({
                "status": "success",
                "message": res
            });
            info!("Unlock pipeline succeeded: {}", res);
            Ok(payload.to_string())
        }
        Err(e) => {
            info!("Unlock pipeline failed: {}", e);
            let payload = json!({
                "status": "error",
                "message": e.to_string()
            });
            info!("Unlock pipeline error payload: {}", payload);
            Ok(payload.to_string())
        }
    }
}
