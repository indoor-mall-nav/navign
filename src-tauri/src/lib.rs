use crate::unlocker::{Challenge, DeviceProof, Unlocker};
use anyhow::Result;
use base64::Engine;
use std::sync::Mutex;
use tauri::State;

pub(crate) mod api;
pub(crate) mod shared;
pub(crate) mod unlocker;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    tauri_plugin_blec::check_permissions()
        .unwrap_or_default()
        .to_string()
        + name
}

#[tauri::command]
async fn request_challenge(
    state: State<'_, Mutex<Unlocker>>,
    nonce: String,
    beacon: String,
) -> Result<Challenge> {
    let nonce =
        TryInto::<[u8; 16]>::try_into(base64::engine::general_purpose::STANDARD.decode(nonce)?)
            .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;
    state
        .lock()
        .map_err(|e| anyhow::anyhow!("Could not get the state manager."))?
        .request_unlock(nonce, beacon)
        .await
}

#[tauri::command]
async fn generate_device_proof(
    state: State<'_, Mutex<Unlocker>>,
    challenge: Challenge,
) -> Result<DeviceProof> {
    state
        .lock()
        .map_err(|e| anyhow::anyhow!("Could not get the state manager."))?
        .generate_device_proof(&challenge)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.handle().plugin(tauri_plugin_opener::init())?;
            app.handle().plugin(tauri_plugin_http::init())?;
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
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
