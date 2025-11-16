//! Authentication handlers with dual-database support
//!
//! These handlers implement user registration and login with PostgreSQL support,
//! following the same patterns as the existing MongoDB-based auth handlers.

use crate::error::Result;
use crate::kernel::auth::Token;
use crate::pg::adapters::{pg_user_to_user, user_to_pg_user};
use crate::pg::repository::{Repository, UserRepository};
use crate::schema::User;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use mongodb::Collection;
use navign_shared::{AuthResponse, LoginRequest, RegisterRequest};
use serde_json::json;
use tracing::{error, info};

/// Default device identifier for web-based authentication
const DEFAULT_DEVICE: &str = "web";

/// Register a new user with dual-database support
pub async fn register_pg_handler(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse> {
    info!("Handling user registration for: {}", request.username);

    if let Some(pg_pool) = state.pg_pool.as_ref() {
        // PostgreSQL registration
        info!("Registering user in PostgreSQL");
        let repo = UserRepository::new(pg_pool.as_ref().clone());

        // Check if username already exists
        if let Some(_existing) = repo.get_by_username(&request.username).await? {
            return Ok((
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "Username already exists"
                })),
            ));
        }

        // Check if email already exists
        if let Some(_existing) = repo.get_by_email(&request.email).await? {
            return Ok((
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "Email already exists"
                })),
            ));
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

        // Convert to PostgreSQL user and insert
        let pg_user = user_to_pg_user(user.clone());
        let user_id = repo.create(&pg_user).await?;

        info!("User registered successfully in PostgreSQL: {}", user_id);

        // Generate JWT token
        let token = Token::from((&user, DEFAULT_DEVICE.to_string()));
        let token_string = token.to_string();

        let response = AuthResponse {
            token: token_string,
            user_id,
            username: request.username,
        };

        Ok((StatusCode::CREATED, Json(json!(response))))
    } else {
        // MongoDB fallback
        info!("Registering user in MongoDB");
        let db = &state.db;
        let collection: Collection<User> = db.collection("users");

        // Check if username already exists
        match collection
            .find_one(bson::doc! { "username": &request.username })
            .await
        {
            Ok(Some(_)) => {
                return Ok((
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "Username already exists"
                    })),
                ));
            }
            Ok(None) => {}
            Err(e) => {
                error!("Database error checking username: {}", e);
                return Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Internal server error"
                    })),
                ));
            }
        }

        // Check if email already exists
        match collection
            .find_one(bson::doc! { "email": &request.email })
            .await
        {
            Ok(Some(_)) => {
                return Ok((
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "Email already exists"
                    })),
                ));
            }
            Ok(None) => {}
            Err(e) => {
                error!("Database error checking email: {}", e);
                return Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Internal server error"
                    })),
                ));
            }
        }

        // Create user
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
                        return Ok((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "Internal server error"
                            })),
                        ));
                    }
                };
                info!("User registered successfully in MongoDB: {}", user_id);

                // Generate JWT token
                let token = Token::from((&user, DEFAULT_DEVICE.to_string()));
                let token_string = token.to_string();

                let response = AuthResponse {
                    token: token_string,
                    user_id,
                    username: request.username,
                };
                Ok((StatusCode::CREATED, Json(json!(response))))
            }
            Err(e) => {
                error!("Failed to create user: {}", e);
                Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to create user"
                    })),
                ))
            }
        }
    }
}

/// Login a user with dual-database support
pub async fn login_pg_handler(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse> {
    info!("Handling user login for: {}", request.username);

    if let Some(pg_pool) = state.pg_pool.as_ref() {
        // PostgreSQL login
        info!("Authenticating user with PostgreSQL");
        let repo = UserRepository::new(pg_pool.as_ref().clone());

        // Find user by username
        let pg_user = match repo.get_by_username(&request.username).await? {
            Some(user) => user,
            None => {
                return Ok((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Invalid username or password"
                    })),
                ));
            }
        };

        // Convert to User for password verification
        let user = pg_user_to_user(pg_user);

        // Verify password
        if !user.verify_password(&request.password) {
            return Ok((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid username or password"
                })),
            ));
        }

        // Generate JWT token
        let user_id = user.id.to_hex();
        let token = Token::from((&user, DEFAULT_DEVICE.to_string()));
        let token_string = token.to_string();

        info!("User logged in successfully (PostgreSQL): {}", user_id);
        let response = AuthResponse {
            token: token_string,
            user_id,
            username: user.username,
        };
        Ok((StatusCode::OK, Json(json!(response))))
    } else {
        // MongoDB fallback
        info!("Authenticating user with MongoDB");
        let db = &state.db;
        let collection: Collection<User> = db.collection("users");

        // Find user by username
        let user = match collection
            .find_one(bson::doc! { "username": &request.username })
            .await
        {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Ok((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Invalid username or password"
                    })),
                ));
            }
            Err(e) => {
                error!("Database error: {}", e);
                return Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Internal server error"
                    })),
                ));
            }
        };

        // Verify password
        if !user.verify_password(&request.password) {
            return Ok((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid username or password"
                })),
            ));
        }

        // Generate JWT token
        let user_id = user.id.to_hex();
        let token = Token::from((&user, DEFAULT_DEVICE.to_string()));
        let token_string = token.to_string();

        info!("User logged in successfully (MongoDB): {}", user_id);
        let response = AuthResponse {
            token: token_string,
            user_id,
            username: user.username,
        };
        Ok((StatusCode::OK, Json(json!(response))))
    }
}
