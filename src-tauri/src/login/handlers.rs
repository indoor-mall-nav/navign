use crate::api::login::{login as api_login, register as api_register, validate_token, logout as api_logout};
use crate::unlocker::Unlocker;
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

#[tauri::command]
pub async fn login_handler(
    _app: AppHandle,
    email: String,
    password: String,
    state: State<'_, Arc<Mutex<Unlocker>>>,
) -> Result<String, String> {
    match api_login(email, password).await {
        Ok(response) => {
            if response.success {
                if let (Some(token), Some(user_id)) = (response.token, response.user_id) {
                    let mut unlocker = state.lock().await;
                    unlocker.set_user_token(token.clone());
                    unlocker.set_user_id(user_id);

                    let result = json!({
                        "status": "success",
                        "token": token,
                        "message": response.message
                    });
                    Ok(result.to_string())
                } else {
                    let result = json!({
                        "status": "error",
                        "message": "Login succeeded but token/user_id missing"
                    });
                    Ok(result.to_string())
                }
            } else {
                let result = json!({
                    "status": "error",
                    "message": response.message
                });
                Ok(result.to_string())
            }
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

#[tauri::command]
pub async fn register_handler(
    _app: AppHandle,
    email: String,
    username: String,
    password: String,
) -> Result<String, String> {
    match api_register(email, username, password).await {
        Ok(response) => {
            let result = json!({
                "status": if response.success { "success" } else { "error" },
                "user_id": response.user_id,
                "message": response.message
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

#[tauri::command]
pub async fn validate_token_handler(
    _app: AppHandle,
    token: String,
) -> Result<String, String> {
    match validate_token(token).await {
        Ok(valid) => {
            let result = json!({
                "status": "success",
                "valid": valid
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

#[tauri::command]
pub async fn logout_handler(
    _app: AppHandle,
    token: String,
    state: State<'_, Arc<Mutex<Unlocker>>>,
) -> Result<String, String> {
    match api_logout(token).await {
        Ok(success) => {
            if success {
                let mut unlocker = state.lock().await;
                unlocker.set_user_token(String::new());
                unlocker.set_user_id(String::new());
            }
            let result = json!({
                "status": if success { "success" } else { "error" },
                "message": if success { "Logged out successfully" } else { "Logout failed" }
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

#[tauri::command]
pub async fn guest_login_handler(
    _app: AppHandle,
    state: State<'_, Arc<Mutex<Unlocker>>>,
) -> Result<String, String> {
    let mut unlocker = state.lock().await;
    let guest_id = format!("guest_{}", nanoid::nanoid!(16));
    unlocker.set_user_id(guest_id.clone());
    unlocker.set_user_token("guest_token".to_string());

    let result = json!({
        "status": "success",
        "user_id": guest_id,
        "message": "Logged in as guest"
    });
    Ok(result.to_string())
}

