use crate::unlocker::{Challenge, DeviceProof, Unlocker};
use base64::Engine;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

pub(crate) mod api;
pub(crate) mod shared;
pub(crate) mod unlocker;

#[tauri::command]
async fn request_challenge(
    state: State<'_, Arc<Mutex<Unlocker>>>,
    nonce: String,
    entity: String,
    beacon: String,
) -> Result<String, ()> {
    let nonce = TryInto::<[u8; 16]>::try_into(
        base64::engine::general_purpose::STANDARD
            .decode(nonce)
            .map_err(|_| ())?,
    )
    .map_err(|_| ())?;
    let result = state
        .lock()
        .await
        .request_unlock(nonce, entity, beacon)
        .await
        .map_err(|_| ())?;

    let json_result = serde_json::to_string(&result).map_err(|_| ())?;
    Ok(json_result)
}

#[tauri::command]
async fn generate_device_proof(
    state: State<'_, Arc<Mutex<Unlocker>>>,
    challenge: Challenge,
) -> Result<String, ()> {
    let result = state
        .lock()
        .await
        .generate_device_proof(&challenge)
        .map_err(|_| ())?;
    let json_result = serde_json::to_string(&result).map_err(|_| ())?;
    Ok(json_result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.handle().plugin(tauri_plugin_opener::init())?;
            app.handle().plugin(tauri_plugin_http::init())?;
            app.handle().plugin(tauri_plugin_notification::init())?;
            let example_public_key = [0u8; 32];
            let state = Arc::new(Mutex::new(Unlocker::new(
                SigningKey::random(&mut OsRng),
                *SigningKey::random(&mut OsRng).verifying_key(),
                "example_user".to_string(),
                "example_token".to_string(),
            )));
            app.manage(state);
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_biometric::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_blec::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_geolocation::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_nfc::init())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            request_challenge,
            generate_device_proof
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
