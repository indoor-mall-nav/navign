use crate::shared::BASE_URL;
use serde::{Deserialize, Serialize};
use tauri_plugin_http::reqwest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub user_id: Option<String>,
    pub message: String,
}

/// Login to the server and return authentication token
pub async fn login(email: String, password: String) -> anyhow::Result<LoginResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}api/auth/login", BASE_URL);

    let request = LoginRequest { email, password };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

    if response.status().is_success() {
        let login_response = response
            .json::<LoginResponse>()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;
        Ok(login_response)
    } else {
        Ok(LoginResponse {
            success: false,
            token: None,
            user_id: None,
            message: format!("Login failed with status: {}", response.status()),
        })
    }
}

/// Register a new user
pub async fn register(
    email: String,
    username: String,
    password: String,
) -> anyhow::Result<RegisterResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}api/auth/register", BASE_URL);

    let request = RegisterRequest {
        email,
        username,
        password,
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

    if response.status().is_success() {
        let register_response = response
            .json::<RegisterResponse>()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;
        Ok(register_response)
    } else {
        Ok(RegisterResponse {
            success: false,
            user_id: None,
            message: format!("Registration failed with status: {}", response.status()),
        })
    }
}

/// Validate token with the server
pub async fn validate_token(token: String) -> anyhow::Result<bool> {
    let client = reqwest::Client::new();
    let url = format!("{}api/auth/validate", BASE_URL);

    let response = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

    Ok(response.status().is_success())
}

/// Logout and invalidate token
pub async fn logout(token: String) -> anyhow::Result<bool> {
    let client = reqwest::Client::new();
    let url = format!("{}api/auth/logout", BASE_URL);

    let response = client
        .post(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

    Ok(response.status().is_success())
}
