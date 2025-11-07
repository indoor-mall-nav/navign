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
