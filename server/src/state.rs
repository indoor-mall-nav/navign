//! Application state shared across the server

use crate::pg;
use mongodb::Database;
use p256::ecdsa::SigningKey;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    #[allow(dead_code)] // Used by dual-database handlers, will be integrated
    pub pg_pool: Option<Arc<pg::PgPool>>,
    pub private_key: SigningKey,
    pub prometheus_handle: metrics_exporter_prometheus::PrometheusHandle,
}
