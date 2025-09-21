use crate::unlocker::Unlocker;
use base64::Engine;
use p256::ecdsa::{SigningKey, VerifyingKey};
use p256::elliptic_curve::rand_core::OsRng;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;
#[cfg(mobile)]
use tauri_plugin_biometric::BiometricExt;
#[cfg(mobile)]
use tauri_plugin_biometric::AuthOptions;

pub(crate) mod api;
pub(crate) mod shared;
pub(crate) mod unlocker;

#[tauri::command]
async fn unlock_door(
    app: AppHandle,
    state: State<'_, Arc<Mutex<Unlocker>>>,
    nonce: String,
    entity: String,
    beacon: String,
) -> Result<String, ()> {
    println!("requesting challenge");
    let nonce = TryInto::<[u8; 16]>::try_into(
        base64::engine::general_purpose::STANDARD
            .decode(nonce)
            .map_err(|_| ())?,
    )
    .map_err(|_| ())?;
    println!("Nonce received: {:?}", nonce);
    let mut app_state = state
        .lock()
        .await;
    let challenge = app_state
        .request_unlock(nonce, entity, beacon)
        .await
        .map_err(|_| ())?;

    println!("Challenge adopted: {:?}", challenge);

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
    app.biometric().authenticate("Please authenticate to unlock".to_string(), auth_options).map_err(|_| ())?;

    println!("Biometric authentication passed, generating proof...");

    let proof = app_state
        .generate_device_proof(&challenge)
        .map_err(|_| ())?;

    println!("Proof generated: {:?}", proof);

    let proof_result = serde_json::to_string(&proof).map_err(|_| ())?;
    Ok(proof_result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.handle().plugin(tauri_plugin_opener::init())?;
            app.handle().plugin(tauri_plugin_http::init())?;
            app.handle().plugin(tauri_plugin_notification::init())?;
            let server_pub_key = [4, 29, 160, 114, 228, 62, 157, 118, 19, 35, 126, 85, 206, 135, 190, 151, 236, 195, 95, 99, 206, 111, 205, 177, 216, 26, 195, 79, 55, 241, 128, 164, 145, 102, 56, 204, 234, 113, 61, 127, 195, 42, 145, 240, 3, 252, 125, 166, 19, 72, 90, 139, 188, 180, 164, 185, 54, 236, 168, 224, 71, 40, 179, 51, 105];
            let state = Arc::new(Mutex::new(Unlocker::new(
                SigningKey::random(&mut OsRng),
                VerifyingKey::from_sec1_bytes(&server_pub_key).unwrap(),
                "7086cmd".to_string(),
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
            unlock_door,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
