use crate::shared::BASE_URL;
use crate::unlocker::Challenge;
use anyhow::Result;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri_plugin_http::reqwest;

pub async fn request_unlock_permission(
    nonce: [u8; 16],
    beacon: String,
    timestamp: u64,
    user_token: &str,
) -> Result<Challenge> {
    let nonce_encoded = base64::engine::general_purpose::STANDARD.encode(&nonce);
    let request_body = json!({
        "nonce": nonce_encoded,
        "beacon": beacon,
        "timestamp": timestamp
    });

    match reqwest::Client::new()
        .post(BASE_URL.to_string() + "api/unlocker")
        .header("Authorization", format!("Bearer {}", user_token))
        .body(request_body.to_string())
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let challenge: Challenge = resp.json().await?;
                Ok(challenge)
            } else {
                Err(anyhow::anyhow!(
                    "Failed to request unlock permission: HTTP {}",
                    resp.status()
                ))
            }
        }
        Err(e) => Err(anyhow::anyhow!("HTTP request error: {}", e)),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconInformation {
    pub uuid: String,
    pub epoch_time: u64,
    pub major: u16,
    pub minor: u16,
}

pub async fn fetch_beacon_information(
    beacon_id: &str,
    entity_id: &str,
    user_token: &str,
) -> Result<BeaconInformation> {
    match reqwest::Client::new()
        .get(BASE_URL.to_string() + "api/entity/" + entity_id + "/hello/beacons/" + beacon_id)
        .header("Authorization", format!("Bearer {}", user_token))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let beacon: BeaconInformation = resp.json().await?;
                Ok(beacon)
            } else {
                Err(anyhow::anyhow!(
                    "Failed to fetch beacon information: HTTP {}",
                    resp.status()
                ))
            }
        }
        Err(e) => Err(anyhow::anyhow!("HTTP request error: {}", e)),
    }
}
