use crate::AppState;
use crate::kernel::auth::Token;
use crate::schema::User;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bson::doc;
use mongodb::Collection;
use navign_shared::{AuthResponse, LoginRequest, RegisterRequest};
use serde_json::json;
use tracing::{error, info};

/// Default device identifier for web-based authentication
const DEFAULT_DEVICE: &str = "web";

/// Register a new user
pub async fn register_handler(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> impl IntoResponse {
    info!("Handling user registration for: {}", request.username);

    let db = &state.db;
    let collection: Collection<User> = db.collection("users");

    // Check if username already exists
    match collection
        .find_one(doc! { "username": &request.username })
        .await
    {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "Username already exists"
                })),
            );
        }
        Ok(None) => {}
        Err(e) => {
            error!("Database error checking username: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error"
                })),
            );
        }
    }

    // Check if email already exists
    match collection.find_one(doc! { "email": &request.email }).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "Email already exists"
                })),
            );
        }
        Ok(None) => {}
        Err(e) => {
            error!("Database error checking email: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error"
                })),
            );
        }
    }

    // Create user (password will be hashed by User::new)
    let user = User::new(
        request.username.clone(),
        request.email,
        None,
        None,
        None,
        request.password,
    );

    // Insert user into database
    match collection.insert_one(&user).await {
        Ok(result) => {
            let user_id = match result.inserted_id.as_object_id() {
                Some(oid) => oid.to_hex(),
                None => {
                    error!("Inserted ID is not an ObjectId");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "Internal server error"
                        })),
                    );
                }
            };
            info!("User registered successfully: {}", user_id);

            // Generate JWT token using existing Token infrastructure
            let token = Token::from((&user, DEFAULT_DEVICE.to_string()));
            let token_string = token.to_string();

            let response = AuthResponse {
                token: token_string,
                user_id,
                username: request.username,
            };
            (StatusCode::CREATED, Json(json!(response)))
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to create user"
                })),
            )
        }
    }
}

/// Login a user
pub async fn login_handler(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
    info!("Handling user login for: {}", request.username);

    let db = &state.db;
    let collection: Collection<User> = db.collection("users");

    // Find user by username
    let user = match collection
        .find_one(doc! { "username": &request.username })
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid username or password"
                })),
            );
        }
        Err(e) => {
            error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error"
                })),
            );
        }
    };

    // Verify password
    if !user.verify_password(&request.password) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Invalid username or password"
            })),
        );
    }

    // Generate JWT token using existing Token infrastructure
    let user_id = user.id.to_hex();
    let token = Token::from((&user, DEFAULT_DEVICE.to_string()));
    let token_string = token.to_string();

    info!("User logged in successfully: {}", user_id);
    let response = AuthResponse {
        token: token_string,
        user_id,
        username: user.username,
    };
    (StatusCode::OK, Json(json!(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are unit tests for the logic. Integration tests would need a test database.

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            None,
            None,
            None,
            "password123".to_string(),
        );

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(!user.activated);
        assert!(!user.is_privileged());
    }

    #[test]
    fn test_password_verification() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            None,
            None,
            None,
            "password123".to_string(),
        );

        assert!(user.verify_password("password123"));
        assert!(!user.verify_password("wrongpassword"));
    }
}
