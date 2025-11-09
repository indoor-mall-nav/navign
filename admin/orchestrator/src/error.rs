use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;
use tonic::Status;

/// Orchestrator error types with detailed context
#[derive(Error, Debug)]
pub enum OrchestratorError {
    // Robot management errors
    #[error("Robot not found: {0}")]
    RobotNotFound(String),

    #[error("Robot registration failed: {0}")]
    RobotRegistrationFailed(String),

    #[error("No suitable robot available for task {0}")]
    NoSuitableRobot(String),

    #[error("Robot is busy: {0}")]
    RobotBusy(String),

    #[error("Robot battery too low: {robot_id} ({battery}%)")]
    RobotBatteryTooLow { robot_id: String, battery: f64 },

    // Task management errors
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Task assignment failed: {0}")]
    TaskAssignmentFailed(String),

    #[error("Invalid task: {0}")]
    InvalidTask(String),

    #[error("Task queue full: {0}")]
    TaskQueueFull(String),

    #[error("Task priority invalid: {0}")]
    InvalidTaskPriority(i32),

    // Entity/Tower errors
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("No tower connected for entity: {0}")]
    NoTowerConnected(String),

    #[error("Tower connection lost: {0}")]
    TowerConnectionLost(String),

    #[error("Failed to send task to tower: {0}")]
    TowerSendFailed(String),

    // Firmware management errors
    #[error("Firmware not found: {0}")]
    FirmwareNotFound(String),

    #[error("Invalid firmware: {0}")]
    InvalidFirmware(String),

    #[error("Firmware download failed: {0}")]
    FirmwareDownloadFailed(String),

    #[error("Firmware server unavailable")]
    FirmwareServerUnavailable,

    // gRPC errors
    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::transport::Error),

    #[error("gRPC status error: {0}")]
    GrpcStatus(#[from] Status),

    #[error("Invalid gRPC request: {0}")]
    InvalidGrpcRequest(String),

    // Network/Communication errors
    #[error("HTTP request failed: {0}")]
    HttpRequestFailed(#[from] reqwest::Error),

    #[error("Network timeout: {0}")]
    NetworkTimeout(String),

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    // Serialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    // Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid input: {field} - {reason}")]
    InvalidInput { field: String, reason: String },

    #[error("Missing required field: {0}")]
    MissingField(String),

    // File I/O errors
    #[error("File I/O error: {0}")]
    FileIo(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    // Configuration errors
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),

    // General errors
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl OrchestratorError {
    /// Get the appropriate HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 400 Bad Request
            Self::InvalidTask(_)
            | Self::InvalidInput { .. }
            | Self::MissingField(_)
            | Self::ValidationError(_)
            | Self::InvalidTaskPriority(_)
            | Self::InvalidGrpcRequest(_)
            | Self::InvalidFirmware(_) => StatusCode::BAD_REQUEST,

            // 404 Not Found
            Self::RobotNotFound(_)
            | Self::TaskNotFound(_)
            | Self::EntityNotFound(_)
            | Self::FirmwareNotFound(_)
            | Self::FileNotFound(_) => StatusCode::NOT_FOUND,

            // 409 Conflict
            Self::RobotBusy(_) => StatusCode::CONFLICT,

            // 422 Unprocessable Entity
            Self::NoSuitableRobot(_) | Self::RobotBatteryTooLow { .. } | Self::TaskQueueFull(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }

            // 500 Internal Server Error
            Self::RobotRegistrationFailed(_)
            | Self::TaskAssignmentFailed(_)
            | Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            // 501 Not Implemented
            Self::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,

            // 502 Bad Gateway
            Self::FirmwareServerUnavailable
            | Self::FirmwareDownloadFailed(_)
            | Self::HttpRequestFailed(_) => StatusCode::BAD_GATEWAY,

            // 503 Service Unavailable
            Self::NoTowerConnected(_)
            | Self::TowerConnectionLost(_)
            | Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,

            // 504 Gateway Timeout
            Self::NetworkTimeout(_) => StatusCode::GATEWAY_TIMEOUT,

            // Default to 500
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Convert to gRPC Status for gRPC responses
    pub fn to_status(&self) -> Status {
        match self {
            Self::RobotNotFound(msg) | Self::TaskNotFound(msg) | Self::EntityNotFound(msg) => {
                Status::not_found(msg)
            }

            Self::InvalidTask(msg)
            | Self::InvalidInput { .. }
            | Self::ValidationError(msg)
            | Self::InvalidGrpcRequest(msg) => Status::invalid_argument(msg),

            Self::NoSuitableRobot(msg) | Self::TaskQueueFull(msg) => {
                Status::resource_exhausted(msg)
            }

            Self::RobotBusy(msg) => Status::already_exists(msg),

            Self::NoTowerConnected(msg) | Self::TowerConnectionLost(msg) => {
                Status::unavailable(msg)
            }

            Self::NotImplemented(msg) => Status::unimplemented(msg),

            Self::GrpcStatus(status) => status.clone(),

            _ => Status::internal(self.to_string()),
        }
    }

    /// Convert error to user-friendly message (hide sensitive details)
    pub fn user_message(&self) -> String {
        match self {
            Self::InternalError(_) => "An internal error occurred".to_string(),
            Self::RobotRegistrationFailed(_) => "Failed to register robot".to_string(),
            Self::TaskAssignmentFailed(_) => "Failed to assign task".to_string(),
            _ => self.to_string(),
        }
    }

    /// Check if error should be logged with full details
    pub fn should_log_details(&self) -> bool {
        matches!(
            self,
            Self::InternalError(_)
                | Self::RobotRegistrationFailed(_)
                | Self::TaskAssignmentFailed(_)
                | Self::GrpcError(_)
                | Self::HttpRequestFailed(_)
                | Self::FileIo(_)
        )
    }
}

impl IntoResponse for OrchestratorError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let user_msg = self.user_message();

        // Log errors with appropriate level
        if self.should_log_details() {
            log::error!("Orchestrator error: {:?}", self);
        } else {
            log::warn!("Request error: {}", self);
        }

        let body = Json(json!({
            "error": user_msg,
            "code": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

// Conversion from String errors (for gradual migration)
impl From<String> for OrchestratorError {
    fn from(s: String) -> Self {
        OrchestratorError::InternalError(s)
    }
}

// Conversion from &str errors
impl From<&str> for OrchestratorError {
    fn from(s: &str) -> Self {
        OrchestratorError::InternalError(s.to_string())
    }
}

/// Type alias for Results using OrchestratorError
pub type Result<T> = std::result::Result<T, OrchestratorError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            OrchestratorError::InvalidTask("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            OrchestratorError::RobotNotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            OrchestratorError::RobotBusy("test".to_string()).status_code(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            OrchestratorError::NoSuitableRobot("test".to_string()).status_code(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            OrchestratorError::InternalError("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_to_status() {
        let not_found = OrchestratorError::RobotNotFound("robot-1".to_string());
        assert_eq!(not_found.to_status().code(), tonic::Code::NotFound);

        let invalid = OrchestratorError::InvalidTask("bad task".to_string());
        assert_eq!(invalid.to_status().code(), tonic::Code::InvalidArgument);

        let unavailable = OrchestratorError::NoTowerConnected("entity-1".to_string());
        assert_eq!(unavailable.to_status().code(), tonic::Code::Unavailable);
    }

    #[test]
    fn test_user_message() {
        let internal_error = OrchestratorError::InternalError("secret details".to_string());
        assert_eq!(internal_error.user_message(), "An internal error occurred");

        let not_found = OrchestratorError::RobotNotFound("robot-1".to_string());
        assert_eq!(not_found.user_message(), "Robot not found: robot-1");
    }

    #[test]
    fn test_should_log_details() {
        assert!(OrchestratorError::InternalError("test".to_string()).should_log_details());
        assert!(!OrchestratorError::ValidationError("test".to_string()).should_log_details());
    }

    #[test]
    fn test_string_conversion() {
        let error: OrchestratorError = "test error".into();
        matches!(error, OrchestratorError::InternalError(_));
    }
}
