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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_serialization() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("password123"));

        let deserialized: LoginRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request.email, deserialized.email);
        assert_eq!(request.password, deserialized.password);
    }

    #[test]
    fn test_login_response_serialization() {
        let response = LoginResponse {
            success: true,
            token: Some("test_token_123".to_string()),
            user_id: Some("user_456".to_string()),
            message: "Login successful".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: LoginResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.success, deserialized.success);
        assert_eq!(response.token, deserialized.token);
        assert_eq!(response.user_id, deserialized.user_id);
        assert_eq!(response.message, deserialized.message);
    }

    #[test]
    fn test_register_request_serialization() {
        let request = RegisterRequest {
            email: "newuser@example.com".to_string(),
            username: "newuser".to_string(),
            password: "securepass".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: RegisterRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.email, deserialized.email);
        assert_eq!(request.username, deserialized.username);
        assert_eq!(request.password, deserialized.password);
    }

    #[test]
    fn test_register_response_serialization() {
        let response = RegisterResponse {
            success: false,
            user_id: None,
            message: "Email already exists".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: RegisterResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.success, deserialized.success);
        assert_eq!(response.user_id, deserialized.user_id);
        assert_eq!(response.message, deserialized.message);
    }

    #[test]
    fn test_login_response_with_none_values() {
        let response = LoginResponse {
            success: false,
            token: None,
            user_id: None,
            message: "Invalid credentials".to_string(),
        };

        assert!(!response.success);
        assert!(response.token.is_none());
        assert!(response.user_id.is_none());
        assert_eq!(response.message, "Invalid credentials");
    }

    #[test]
    fn test_login_request_clone() {
        let original = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(original.email, cloned.email);
        assert_eq!(original.password, cloned.password);
    }

    #[test]
    fn test_register_response_with_user_id() {
        let response = RegisterResponse {
            success: true,
            user_id: Some("user_789".to_string()),
            message: "Registration successful".to_string(),
        };

        assert!(response.success);
        assert!(response.user_id.is_some());
        assert_eq!(
            response.user_id.expect("user_id should be present"),
            "user_789"
        );
    }
}
