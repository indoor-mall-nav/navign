use crate::shared::BASE_URL;
use crate::unlocker::Challenge;
use anyhow::Result;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri_plugin_http::reqwest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CustomizedObjectId {
    #[serde(rename = "$oid")]
    pub oid: String,
}

pub async fn request_unlock_permission(
    nonce: [u8; 16],
    entity: String,
    beacon: String,
    timestamp: u64,
    user_token: &str,
) -> Result<Challenge> {
    let nonce_encoded = base64::engine::general_purpose::STANDARD.encode(nonce);
    let request_body = json!({
        "nonce": nonce_encoded,
        "beacon": beacon,
        "timestamp": timestamp
    });

    println!("The body is: {:?}", request_body);

    let url = format!("{}api/entities/{entity}/beacons/unlocker", BASE_URL);

    println!("Making request to: {}", url);

    println!("Hello!");

    match reqwest::Client::new()
        .post(url)
        .header("Authorization", format!("Bearer {}", user_token))
        .header("Content-Type", "application/json")
        .body(request_body.to_string())
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let challenge: reqwest::Result<Challenge> = resp.json().await;
                match challenge {
                    Ok(res) => {
                        println!("Challenge received: {:?}", res);
                        Ok(res)
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON response: {:?}", e);
                        Err(anyhow::anyhow!("Failed to parse JSON response: {}", e))
                    }
                }
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
    pub _id: CustomizedObjectId,
    pub entity: CustomizedObjectId,
    pub area: CustomizedObjectId,
    pub merchant: Option<CustomizedObjectId>,
    pub connection: Option<CustomizedObjectId>,
    pub name: String,
    pub description: String,
    pub r#type: String,
    pub location: [f64; 2],
    pub device: String,
    pub last_boot: u64,
}

pub async fn fetch_beacon_information(
    beacon_id: &str,
    entity_id: &str,
    user_token: &str,
) -> Result<BeaconInformation> {
    let url = format!(
        "{}api/entities/{}/beacons/{}",
        BASE_URL, entity_id, beacon_id
    );
    println!("Making request to: {}", url);
    match reqwest::Client::new()
        .get(url)
        .header("Authorization", format!("Bearer {}", user_token))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let result: reqwest::Result<BeaconInformation> = resp.json().await;
                match result {
                    Ok(resp) => Ok(resp),
                    Err(e) => {
                        eprintln!("Failed to parse JSON response: {:?}", e);
                        Err(anyhow::anyhow!("Failed to parse JSON response: {}", e))
                    }
                }
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
