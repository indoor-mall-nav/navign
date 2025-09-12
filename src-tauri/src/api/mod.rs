use crate::shared::BASE_URL;
use base64::Engine;
use tauri_plugin_http::reqwest;

pub(crate) mod login;
pub(crate) mod unlocker;

/// Response is a base64-encoded string of the server's public key bytes.
async fn fetch_server_public_key() -> anyhow::Result<Vec<u8>> {
    match reqwest::Client::new()
        .get(BASE_URL.to_string() + "api/cert")
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let key_response: serde_json::Value = resp.json().await?;
                if let Some(key_base64) = key_response.get("public_key").and_then(|v| v.as_str()) {
                    let key_bytes = base64::engine::general_purpose::STANDARD.decode(key_base64)?;
                    Ok(key_bytes)
                } else {
                    Err(anyhow::anyhow!(
                        "Invalid response format: missing 'public_key' field"
                    ))
                }
            } else {
                Err(anyhow::anyhow!(
                    "Failed to fetch server public key: HTTP {}",
                    resp.status()
                ))
            }
        }
        Err(e) => Err(anyhow::anyhow!("HTTP request error: {}", e)),
    }
}
