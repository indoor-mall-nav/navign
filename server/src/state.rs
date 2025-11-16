//! Application state shared across the server

use crate::pg;
use p256::ecdsa::SigningKey;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: pg::PgPool,
    pub private_key: SigningKey,
    pub prometheus_handle: metrics_exporter_prometheus::PrometheusHandle,
}
