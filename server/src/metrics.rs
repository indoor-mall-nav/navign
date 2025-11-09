use axum::{
    extract::{MatchedPath, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use metrics::{counter, describe_counter, describe_histogram, histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::time::Instant;

use crate::AppState;

/// Initialize the Prometheus metrics exporter
pub fn init_metrics() -> anyhow::Result<PrometheusHandle> {
    // Create Prometheus exporter with custom configuration
    let handle = PrometheusBuilder::new()
        // Set buckets for HTTP request duration histogram (in seconds)
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            &[
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        )?
        .install_recorder()?;

    // Describe metrics for better documentation
    describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests received"
    );
    describe_histogram!(
        "http_requests_duration_seconds",
        "HTTP request duration in seconds"
    );
    describe_counter!(
        "http_requests_errors_total",
        "Total number of HTTP requests that resulted in errors"
    );
    describe_counter!(
        "database_operations_total",
        "Total number of database operations"
    );
    describe_counter!(
        "auth_attempts_total",
        "Total number of authentication attempts"
    );
    describe_counter!("unlock_attempts_total", "Total number of unlock attempts");

    Ok(handle)
}

/// Middleware for collecting HTTP request metrics
pub async fn track_metrics(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    // Call the next handler
    let response = next.run(req).await;

    // Record metrics
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // HTTP request count
    counter!(
        "http_requests_total",
        "method" => method.to_string(),
        "path" => path.clone(),
        "status" => status.clone(),
    )
    .increment(1);

    // HTTP request duration
    histogram!(
        "http_requests_duration_seconds",
        "method" => method.to_string(),
        "path" => path.clone(),
    )
    .record(latency);

    // Track errors (4xx and 5xx status codes)
    if response.status().is_client_error() || response.status().is_server_error() {
        counter!(
            "http_requests_errors_total",
            "method" => method.to_string(),
            "path" => path,
            "status" => status,
        )
        .increment(1);
    }

    response
}

/// Handler for the /metrics endpoint
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, state.prometheus_handle.render())
}

/// Record a database operation metric
#[allow(dead_code)]
pub fn record_database_operation(operation: &str, collection: &str) {
    counter!(
        "database_operations_total",
        "operation" => operation.to_string(),
        "collection" => collection.to_string(),
    )
    .increment(1);
}

/// Record an authentication attempt metric
#[allow(dead_code)]
pub fn record_auth_attempt(method: &str, success: bool) {
    counter!(
        "auth_attempts_total",
        "method" => method.to_string(),
        "success" => success.to_string(),
    )
    .increment(1);
}

/// Record an unlock attempt metric
#[allow(dead_code)]
pub fn record_unlock_attempt(success: bool, error: Option<&str>) {
    counter!(
        "unlock_attempts_total",
        "success" => success.to_string(),
        "error" => error.unwrap_or("none").to_string(),
    )
    .increment(1);
}
