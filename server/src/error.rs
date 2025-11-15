use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

/// Server error types with detailed context
#[derive(Error, Debug)]
#[allow(dead_code)] // Allow unused variants during gradual migration
pub enum ServerError {
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] mongodb::error::Error),

    #[error("Database connection failed: {0}")]
    DatabaseConnection(String),

    #[error("Database query error: {0}")]
    DatabaseQuery(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid ObjectId format: {0}")]
    InvalidObjectId(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    // Authentication & Authorization errors
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),

    #[error("Token validation failed: {0}")]
    TokenValidationFailed(String),

    #[error("Unauthorized access: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    // Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    // Cryptography errors
    #[error("Cryptography error: {0}")]
    CryptographyError(String),

    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    // Pathfinding errors
    #[error("Pathfinding error: {0}")]
    PathfindingError(String),

    #[error("No path found from {from} to {to}")]
    NoPathFound { from: String, to: String },

    #[error("Invalid location: {0}")]
    InvalidLocation(String),

    #[error("Area not found: {0}")]
    AreaNotFound(String),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),

    // External API errors
    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("OAuth error: {provider} - {message}")]
    OAuthError { provider: String, message: String },

    #[error("HTTP request failed: {0}")]
    HttpRequestFailed(#[from] reqwest::Error),

    // File I/O errors
    #[error("File I/O error: {0}")]
    FileIo(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    // Serialization/Deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("BSON serialization error: {0}")]
    BsonSerError(String),

    #[error("BSON deserialization error: {0}")]
    BsonDeError(String),

    // Rate limiting
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    // Firmware management errors
    #[error("Firmware not found: {0}")]
    FirmwareNotFound(String),

    #[error("Invalid firmware: {0}")]
    InvalidFirmware(String),

    #[error("Firmware upload failed: {0}")]
    FirmwareUploadFailed(String),

    // Multipart errors
    #[error("Multipart error: {0}")]
    MultipartError(String),

    // General errors
    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    // Legacy anyhow integration for gradual migration
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ServerError {
    /// Get the appropriate HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 400 Bad Request
            Self::ValidationError(_)
            | Self::InvalidInput(_)
            | Self::MissingField(_)
            | Self::InvalidFormat(_)
            | Self::InvalidObjectId(_)
            | Self::InvalidLocation(_)
            | Self::InvalidFirmware(_)
            | Self::MultipartError(_) => StatusCode::BAD_REQUEST,

            // 401 Unauthorized
            Self::AuthenticationFailed(_)
            | Self::InvalidCredentials
            | Self::TokenValidationFailed(_)
            | Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,

            // 403 Forbidden
            Self::Forbidden(_) => StatusCode::FORBIDDEN,

            // 404 Not Found
            Self::NotFound(_)
            | Self::EntityNotFound(_)
            | Self::AreaNotFound(_)
            | Self::ConnectionNotFound(_)
            | Self::FileNotFound(_)
            | Self::FirmwareNotFound(_) => StatusCode::NOT_FOUND,

            // 409 Conflict
            Self::DuplicateEntry(_) => StatusCode::CONFLICT,

            // 422 Unprocessable Entity
            Self::NoPathFound { .. } => StatusCode::UNPROCESSABLE_ENTITY,

            // 429 Too Many Requests
            Self::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,

            // 501 Not Implemented
            Self::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,

            // 503 Service Unavailable
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,

            // 500 Internal Server Error (default)
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Convert error to user-friendly message (hide sensitive details)
    pub fn user_message(&self) -> String {
        match self {
            Self::Database(_) => "A database error occurred".to_string(),
            Self::DatabaseConnection(_) => "Could not connect to database".to_string(),
            Self::InternalError(_) => "An internal server error occurred".to_string(),
            Self::CryptographyError(_) => "A cryptography error occurred".to_string(),
            _ => self.to_string(),
        }
    }

    /// Check if error should be logged with full details
    pub fn should_log_details(&self) -> bool {
        matches!(
            self,
            Self::Database(_)
                | Self::DatabaseConnection(_)
                | Self::InternalError(_)
                | Self::CryptographyError(_)
                | Self::KeyGenerationFailed(_)
                | Self::ExternalApi(_)
                | Self::HttpRequestFailed(_)
                | Self::FileIo(_)
        )
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let user_msg = self.user_message();

        // Log errors with appropriate level
        if self.should_log_details() {
            tracing::error!("Server error: {:?}", self);
        } else {
            tracing::warn!("Request error: {}", self);
        }

        let body = Json(json!({
            "error": user_msg,
            "code": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

/// Type alias for Results using ServerError
pub type Result<T> = std::result::Result<T, ServerError>;

/// Helper trait to convert Results into ServerError
#[allow(dead_code)] // Reserved for future use
pub trait ResultExt<T> {
    fn map_err_to_server(self, msg: &str) -> Result<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for std::result::Result<T, E> {
    fn map_err_to_server(self, msg: &str) -> Result<T> {
        self.map_err(|e| ServerError::InternalError(format!("{}: {}", msg, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            ServerError::ValidationError("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ServerError::InvalidCredentials.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ServerError::EntityNotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ServerError::DuplicateEntry("test".to_string()).status_code(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            ServerError::InternalError("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_user_message() {
        let db_error = ServerError::Database(mongodb::error::Error::custom("test"));
        assert_eq!(db_error.user_message(), "A database error occurred");

        let validation_error = ServerError::ValidationError("Invalid email".to_string());
        assert_eq!(
            validation_error.user_message(),
            "Validation error: Invalid email"
        );
    }

    #[test]
    fn test_should_log_details() {
        assert!(ServerError::InternalError("test".to_string()).should_log_details());
        assert!(!ServerError::ValidationError("test".to_string()).should_log_details());
    }
}
