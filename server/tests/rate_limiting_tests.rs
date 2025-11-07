use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
};
use std::net::SocketAddr;
use tower::ServiceExt;

// Import the rate limiting module
// Note: This assumes the rate limiting module is accessible from tests
// You may need to make it pub in the main crate or expose it via lib.rs

/// Helper function to create a test router with rate limiting
fn create_test_router_with_rate_limit(requests_per_second: u64, burst_size: u32) -> Router {
    use tower_governor::GovernorLayer;
    use tower_governor::governor::GovernorConfigBuilder;
    use tower_governor::key_extractor::SmartIpKeyExtractor;

    async fn handler() -> impl IntoResponse {
        (StatusCode::OK, "success")
    }

    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(requests_per_second)
            .burst_size(burst_size)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    let rate_limit_layer = GovernorLayer {
        config: Box::leak(governor_conf),
    };

    Router::new()
        .route("/test", get(handler))
        .layer(rate_limit_layer)
}

#[tokio::test]
async fn test_rate_limit_allows_within_limit() {
    let app = create_test_router_with_rate_limit(10, 20);

    // Make 5 requests (well within the limit)
    for i in 0..5 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("X-Forwarded-For", "192.168.1.100")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Request {} should succeed",
            i
        );
    }
}

#[tokio::test]
async fn test_rate_limit_blocks_over_limit() {
    // Very restrictive rate limit: 1 per second, burst of 2
    let app = create_test_router_with_rate_limit(1, 2);

    // First 2 requests should succeed (within burst)
    for i in 0..2 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("X-Forwarded-For", "192.168.1.101")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Request {} should succeed",
            i
        );
    }

    // Third request should be rate limited
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("X-Forwarded-For", "192.168.1.101")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "Request should be rate limited"
    );
}

#[tokio::test]
async fn test_rate_limit_per_ip() {
    // Rate limit: 1 per second, burst of 1
    let app = create_test_router_with_rate_limit(1, 1);

    // First IP makes a request - should succeed
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("X-Forwarded-For", "192.168.1.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Second IP makes a request - should also succeed (different IP)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("X-Forwarded-For", "192.168.1.2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // First IP tries again immediately - should be rate limited
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("X-Forwarded-For", "192.168.1.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_rate_limit_respects_x_real_ip() {
    let app = create_test_router_with_rate_limit(1, 1);

    // Request with X-Real-IP header
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("X-Real-IP", "10.0.0.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Second request from same IP should be limited
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("X-Real-IP", "10.0.0.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_rate_limit_burst_size() {
    // Allow burst of 5 requests, then 1 per second
    let app = create_test_router_with_rate_limit(1, 5);

    // First 5 requests should succeed (burst)
    for i in 0..5 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("X-Forwarded-For", "192.168.2.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Burst request {} should succeed",
            i
        );
    }

    // 6th request should be rate limited
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("X-Forwarded-For", "192.168.2.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_rate_limit_recovery_after_wait() {
    let app = create_test_router_with_rate_limit(10, 2);

    // Use up burst
    for _ in 0..2 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("X-Forwarded-For", "192.168.3.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Next request should be limited
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("X-Forwarded-For", "192.168.3.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    // Wait for rate limit to recover (200ms should allow 2 more requests at 10/sec)
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Should succeed now
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("X-Forwarded-For", "192.168.3.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_rate_limit_with_no_forwarded_headers() {
    let app = create_test_router_with_rate_limit(1, 1);

    // Request without any forwarded headers (will use connection IP)
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Second request should be limited
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_high_throughput_rate_limiting() {
    // Test with higher limits
    let app = create_test_router_with_rate_limit(100, 200);

    let mut success_count = 0;
    let mut rate_limited_count = 0;

    // Make 250 requests rapidly from same IP
    for _ in 0..250 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("X-Forwarded-For", "192.168.4.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        match response.status() {
            StatusCode::OK => success_count += 1,
            StatusCode::TOO_MANY_REQUESTS => rate_limited_count += 1,
            _ => panic!("Unexpected status code"),
        }
    }

    // Should allow burst of 200
    assert_eq!(success_count, 200, "Should allow burst of 200 requests");
    assert_eq!(
        rate_limited_count, 50,
        "Should rate limit remaining 50 requests"
    );
}

#[test]
fn test_rate_limit_configuration_from_env() {
    use std::env;

    // Test default values when env vars not set
    env::remove_var("RATE_LIMIT_PER_SECOND");
    env::remove_var("RATE_LIMIT_BURST_SIZE");

    let requests_per_second = env::var("RATE_LIMIT_PER_SECOND")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(100);

    let burst_size = env::var("RATE_LIMIT_BURST_SIZE")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(200);

    assert_eq!(requests_per_second, 100);
    assert_eq!(burst_size, 200);

    // Test custom values
    env::set_var("RATE_LIMIT_PER_SECOND", "50");
    env::set_var("RATE_LIMIT_BURST_SIZE", "100");

    let requests_per_second = env::var("RATE_LIMIT_PER_SECOND")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(100);

    let burst_size = env::var("RATE_LIMIT_BURST_SIZE")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(200);

    assert_eq!(requests_per_second, 50);
    assert_eq!(burst_size, 100);

    // Cleanup
    env::remove_var("RATE_LIMIT_PER_SECOND");
    env::remove_var("RATE_LIMIT_BURST_SIZE");
}

#[test]
fn test_rate_limit_configuration_invalid_env() {
    use std::env;

    // Test with invalid env values (should fall back to defaults)
    env::set_var("RATE_LIMIT_PER_SECOND", "invalid");
    env::set_var("RATE_LIMIT_BURST_SIZE", "not_a_number");

    let requests_per_second = env::var("RATE_LIMIT_PER_SECOND")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(100);

    let burst_size = env::var("RATE_LIMIT_BURST_SIZE")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(200);

    assert_eq!(requests_per_second, 100);
    assert_eq!(burst_size, 200);

    // Cleanup
    env::remove_var("RATE_LIMIT_PER_SECOND");
    env::remove_var("RATE_LIMIT_BURST_SIZE");
}
