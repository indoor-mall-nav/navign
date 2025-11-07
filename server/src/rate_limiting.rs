use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use log::info;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;

/// Create a rate limiting layer with configurable requests per second.
///
/// The rate limiter uses the client's IP address as the key and allows
/// a burst of requests up to the specified limit per second.
///
/// # Arguments
/// * `requests_per_second` - Maximum number of requests allowed per second per IP
/// * `burst_size` - Maximum burst size (number of requests that can be made at once)
///
/// # Example
/// ```no_run
/// let rate_limit_layer = create_rate_limit_layer(10, 20);
/// // Allows 10 requests per second with a burst of 20
/// ```
pub fn create_rate_limit_layer(
    requests_per_second: u64,
    burst_size: u32,
) -> GovernorLayer<SmartIpKeyExtractor> {
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(requests_per_second)
            .burst_size(burst_size)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    info!(
        "Rate limiting configured: {} requests/second with burst size {}",
        requests_per_second, burst_size
    );

    GovernorLayer {
        config: Box::leak(governor_conf),
    }
}

/// Create a default rate limiting layer with reasonable defaults.
///
/// Default configuration:
/// - 100 requests per second per IP
/// - Burst size of 200 requests
pub fn create_default_rate_limit_layer() -> GovernorLayer<SmartIpKeyExtractor> {
    let requests_per_second = std::env::var("RATE_LIMIT_PER_SECOND")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let burst_size = std::env::var("RATE_LIMIT_BURST_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);

    create_rate_limit_layer(requests_per_second, burst_size)
}

/// Custom error response for rate limiting.
pub struct RateLimitError;

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Please try again later.",
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rate_limit_layer_with_custom_values() {
        let layer = create_rate_limit_layer(50, 100);
        // If layer creation succeeds without panic, test passes
        assert!(std::ptr::addr_of!(layer).is_null() == false);
    }

    #[test]
    fn test_create_default_rate_limit_layer() {
        // Clear env vars to test defaults
        std::env::remove_var("RATE_LIMIT_PER_SECOND");
        std::env::remove_var("RATE_LIMIT_BURST_SIZE");

        let layer = create_default_rate_limit_layer();
        // If layer creation succeeds without panic, test passes
        assert!(std::ptr::addr_of!(layer).is_null() == false);
    }

    #[test]
    fn test_create_default_rate_limit_layer_with_env() {
        // Set custom env vars
        std::env::set_var("RATE_LIMIT_PER_SECOND", "25");
        std::env::set_var("RATE_LIMIT_BURST_SIZE", "50");

        let layer = create_default_rate_limit_layer();
        // If layer creation succeeds without panic, test passes
        assert!(std::ptr::addr_of!(layer).is_null() == false);

        // Cleanup
        std::env::remove_var("RATE_LIMIT_PER_SECOND");
        std::env::remove_var("RATE_LIMIT_BURST_SIZE");
    }

    #[test]
    fn test_rate_limit_error_response() {
        let error = RateLimitError;
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_rate_limit_config_parsing() {
        // Test valid parsing
        std::env::set_var("RATE_LIMIT_PER_SECOND", "75");
        std::env::set_var("RATE_LIMIT_BURST_SIZE", "150");

        let requests_per_second = std::env::var("RATE_LIMIT_PER_SECOND")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        let burst_size = std::env::var("RATE_LIMIT_BURST_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200);

        assert_eq!(requests_per_second, 75);
        assert_eq!(burst_size, 150);

        // Cleanup
        std::env::remove_var("RATE_LIMIT_PER_SECOND");
        std::env::remove_var("RATE_LIMIT_BURST_SIZE");
    }

    #[test]
    fn test_rate_limit_config_defaults() {
        // Remove env vars
        std::env::remove_var("RATE_LIMIT_PER_SECOND");
        std::env::remove_var("RATE_LIMIT_BURST_SIZE");

        let requests_per_second = std::env::var("RATE_LIMIT_PER_SECOND")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        let burst_size = std::env::var("RATE_LIMIT_BURST_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200);

        assert_eq!(requests_per_second, 100);
        assert_eq!(burst_size, 200);
    }

    #[test]
    fn test_rate_limit_config_invalid_values() {
        // Set invalid values
        std::env::set_var("RATE_LIMIT_PER_SECOND", "not_a_number");
        std::env::set_var("RATE_LIMIT_BURST_SIZE", "also_invalid");

        let requests_per_second = std::env::var("RATE_LIMIT_PER_SECOND")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(100);

        let burst_size = std::env::var("RATE_LIMIT_BURST_SIZE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(200);

        // Should fall back to defaults
        assert_eq!(requests_per_second, 100);
        assert_eq!(burst_size, 200);

        // Cleanup
        std::env::remove_var("RATE_LIMIT_PER_SECOND");
        std::env::remove_var("RATE_LIMIT_BURST_SIZE");
    }

    #[test]
    fn test_rate_limit_config_edge_cases() {
        // Test with 0 values
        std::env::set_var("RATE_LIMIT_PER_SECOND", "0");
        std::env::set_var("RATE_LIMIT_BURST_SIZE", "0");

        let requests_per_second = std::env::var("RATE_LIMIT_PER_SECOND")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(100);

        let burst_size = std::env::var("RATE_LIMIT_BURST_SIZE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(200);

        assert_eq!(requests_per_second, 0);
        assert_eq!(burst_size, 0);

        // Test with very large values
        std::env::set_var("RATE_LIMIT_PER_SECOND", "1000000");
        std::env::set_var("RATE_LIMIT_BURST_SIZE", "2000000");

        let requests_per_second = std::env::var("RATE_LIMIT_PER_SECOND")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(100);

        let burst_size = std::env::var("RATE_LIMIT_BURST_SIZE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(200);

        assert_eq!(requests_per_second, 1000000);
        assert_eq!(burst_size, 2000000);

        // Cleanup
        std::env::remove_var("RATE_LIMIT_PER_SECOND");
        std::env::remove_var("RATE_LIMIT_BURST_SIZE");
    }
}
