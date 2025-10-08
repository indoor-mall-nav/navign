//! Handshake with the server to exchange public keys.

use crate::shared::BASE_URL;
use crate::unlocker::Unlocker;
use aes_gcm::aead::{Aead, OsRng as AesOsRng};
use aes_gcm::{AeadCore, Aes256Gcm, KeyInit};
use base64::Engine;
use p256::ecdsa::{SigningKey, VerifyingKey};
use p256::elliptic_curve::rand_core::OsRng as P256OsRng;
use p256::pkcs8::{EncodePublicKey, LineEnding};
use rsa::pkcs8::DecodePublicKey;
use rsa::RsaPublicKey;
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, State};
#[cfg(mobile)]
use tauri_plugin_biometric::AuthOptions;
#[cfg(mobile)]
use tauri_plugin_biometric::BiometricExt;
use tauri_plugin_http::reqwest;
#[cfg(all(desktop, dev))]
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Mutex;

async fn perform_handshake(
    public_key: &VerifyingKey,
    device_id: String,
    token: String,
) -> anyhow::Result<String> {
    let payload = public_key
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| anyhow::anyhow!("Failed to encode public key to PEM: {}", e))?;
    let client = reqwest::Client::new();

    let certification = client
        .get(BASE_URL.to_string() + "/cert")
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("HTTP error: {}", e))?
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read response text: {}", e))?;

    let key = RsaPublicKey::from_public_key_pem(certification.as_str())
        .map_err(|e| anyhow::anyhow!("Failed to parse server public key: {}", e))?;

    let aes_key = Aes256Gcm::generate_key(&mut AesOsRng);
    let cipher = Aes256Gcm::new(&aes_key);
    let nonce = Aes256Gcm::generate_nonce(&mut AesOsRng);
    let encrypt_payload = json!({
        "device_id": device_id,
        "payload": base64::engine::general_purpose::STANDARD.encode(&payload),
        "timestamp": chrono::Utc::now().timestamp(),
    })
    .to_string();
    let ciphertext = cipher
        .encrypt(&nonce, encrypt_payload.as_bytes())
        .map_err(|e| anyhow::anyhow!("AES encryption failed: {}", e))?;

    let mut message = Vec::new();
    message.extend_from_slice(nonce.as_slice());
    message.extend_from_slice(&ciphertext);

    let encrypted_key = key
        .encrypt(&mut AesOsRng, rsa::Pkcs1v15Encrypt, aes_key.as_slice())
        .map_err(|e| anyhow::anyhow!("RSA encryption failed: {}", e))?;

    let mut final_message = Vec::with_capacity(encrypted_key.len() + message.len());
    final_message.extend_from_slice(&encrypted_key);
    final_message.extend_from_slice(&message);

    let encoded_message = base64::engine::general_purpose::STANDARD.encode(&final_message);
    let response = client
        .post(BASE_URL.to_string() + "/handshake")
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .body(encoded_message)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("HTTP error: {}", e))?
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read response text: {}", e))?;
    Ok(response)
}

#[tauri::command]
pub async fn bind_with_server(
    handle: AppHandle,
    state: State<'_, Arc<Mutex<Unlocker>>>,
) -> Result<String, ()> {
    let key = SigningKey::random(&mut P256OsRng);
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
        .map_err(|_| ())?;

    #[cfg(all(desktop, not(dev)))]
    panic!("Biometric authentication is not supported on desktop.");

    #[cfg(all(desktop, dev))]
    handle
        .notification()
        .builder()
        .body("Biometric authentication passed, generating keys...")
        .show()
        .map_err(|_| ())?;

    let unlocker = state.lock().await;
    let public_key = key.verifying_key();

    match perform_handshake(
        public_key,
        unlocker.get_user_token().to_string(),
        unlocker.device_id().to_string(),
    )
    .await
    {
        Ok(res) => {
            let value = json!({
                "status": "success",
                "message": res,
            });
            Ok(value.to_string())
        }
        Err(e) => {
            let value = json!({
                "status": "error",
                "message": e.to_string(),
            });
            Ok(value.to_string())
        }
    }
}
