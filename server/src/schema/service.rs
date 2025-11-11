use crate::AppState;
use crate::schema::metadata::{PaginationResponse, PaginationResponseMetadata};
use crate::shared::ReadQuery;
use async_trait::async_trait;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use log::info;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQueryParams<'a> {
    pattern: &'a str,
    offset: i64,
    limit: i64,
    sort: Option<&'a str>,
    asc: bool,
    case_insensitive: bool,
    entity: &'a str,
}

#[async_trait]
#[allow(dead_code)]
pub trait Service: Serialize + DeserializeOwned + Send + Sync + Clone {
    type Id: Send + Sync;

    fn get_id(&self) -> Self::Id;
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn get_description(&self) -> Option<String>;
    fn set_description(&mut self, description: Option<String>);
    fn get_table_name() -> &'static str;
    fn require_unique_name() -> bool;

    async fn get_one_by_id(pool: &PgPool, id: Self::Id) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized;

    async fn get_one_by_name(
        pool: &PgPool,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized;

    async fn get_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized;

    async fn get_with_pagination(
        pool: &PgPool,
        page: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized;

    async fn create(&self, pool: &PgPool) -> Result<Self::Id, sqlx::Error>
    where
        Self: Sized;

    async fn update(&self, pool: &PgPool) -> Result<(), sqlx::Error>
    where
        Self: Sized;

    async fn delete_by_id(pool: &PgPool, id: Self::Id) -> Result<(), sqlx::Error>
    where
        Self: Sized;

    async fn delete_by_name(pool: &PgPool, name: &str) -> Result<(), sqlx::Error>
    where
        Self: Sized;

    async fn exists_by_name(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error>
    where
        Self: Sized,
    {
        Ok(Self::get_one_by_name(pool, name).await?.is_some())
    }

    async fn search_and_page_by_name_pattern(
        pool: &PgPool,
        params: SearchQueryParams<'_>,
    ) -> Result<PaginationResponse<Self>, sqlx::Error>
    where
        Self: Sized;

    async fn search_by_description_pattern(
        pool: &PgPool,
        pattern: &str,
        case_insensitive: bool,
    ) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized;

    async fn bulk_create(
        pool: &PgPool,
        services: Vec<Self>,
    ) -> Result<Vec<Self::Id>, sqlx::Error>
    where
        Self: Sized;

    async fn get_handler(
        State(state): State<AppState>,
        Query(ReadQuery {
            offset,
            limit,
            query,
            sort,
            asc,
            case_sensitive,
        }): Query<ReadQuery>,
        Path(entity): Path<String>,
    ) -> impl IntoResponse {
        info!(
            "Handling GET request for services with query: {:?} in table {}",
            query,
            Self::get_table_name()
        );
        let pool = &state.pool;
        let offset = offset.unwrap_or(0) as i64;
        let limit = limit.unwrap_or(10).min(100) as i64;
        let query = query.unwrap_or_default();
        let sort = sort.as_deref();
        let asc = asc.unwrap_or(true);
        let case_sensitive = case_sensitive.unwrap_or(false);
        info!(
            "Query parameters in {}: offset={offset}, limit={limit}, query='{query}', sort={sort:?}, asc={asc}, case_sensitive={case_sensitive}",
            Self::get_table_name()
        );
        match Self::search_and_page_by_name_pattern(
            pool,
            SearchQueryParams {
                pattern: &query,
                offset,
                limit,
                sort,
                asc,
                case_insensitive: !case_sensitive,
                entity: entity.as_str(),
            },
        )
        .await
        {
            Ok(services) => (StatusCode::OK, axum::Json(json!(services))),
            Err(e) => {
                info!("Failed to retrieve services: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({
                        "error": "Failed to retrieve services",
                        "details": e.to_string()
                    })),
                )
            }
        }
    }

    async fn get_one_handler(
        State(state): State<AppState>,
        Path(id): Path<(String, String)>,
    ) -> impl IntoResponse
    where
        Self::Id: std::str::FromStr,
        <Self::Id as std::str::FromStr>::Err: std::fmt::Display,
    {
        info!("Handling GET request for service with ID: {:?}", id);
        let pool = &state.pool;

        let parsed_id = match id.1.parse::<Self::Id>() {
            Ok(id) => id,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "error": format!("Invalid ID format: {}", e)
                    })),
                )
            }
        };

        match Self::get_one_by_id(pool, parsed_id).await {
            Ok(Some(service)) => (StatusCode::OK, axum::Json(json!(service))),
            Ok(None) => (
                StatusCode::NOT_FOUND,
                axum::Json(json!({
                    "error": "Service not found"
                })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "error": "Database error",
                    "details": e.to_string()
                })),
            ),
        }
    }

    async fn create_handler(
        State(state): State<AppState>,
        axum::Json(service): axum::Json<Self>,
    ) -> impl IntoResponse
    where
        Self::Id: Serialize,
    {
        let pool = &state.pool;
        match service.create(pool).await {
            Ok(id) => (StatusCode::CREATED, axum::Json(json!({ "id": id }))),
            Err(e) => {
                log::error!("Failed to create service: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({
                        "error": "Failed to create service",
                        "details": e.to_string()
                    })),
                )
            }
        }
    }

    async fn update_handler(
        State(state): State<AppState>,
        axum::Json(service): axum::Json<Self>,
    ) -> impl IntoResponse {
        let pool = &state.pool;
        match service.update(pool).await {
            Ok(_) => (StatusCode::OK, axum::Json(json!({ "status": "updated" }))),
            Err(e) => {
                log::error!("Failed to update service: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({
                        "error": "Failed to update service",
                        "details": e.to_string()
                    })),
                )
            }
        }
    }

    async fn delete_handler(
        State(state): State<AppState>,
        Path(id): Path<(String, String)>,
    ) -> impl IntoResponse
    where
        Self::Id: std::str::FromStr,
        <Self::Id as std::str::FromStr>::Err: std::fmt::Display,
    {
        let pool = &state.pool;

        let parsed_id = match id.1.parse::<Self::Id>() {
            Ok(id) => id,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "error": format!("Invalid ID format: {}", e)
                    })),
                )
            }
        };

        match Self::delete_by_id(pool, parsed_id).await {
            Ok(_) => (
                StatusCode::NO_CONTENT,
                axum::Json(json!({ "status": "deleted" })),
            ),
            Err(e) => {
                log::error!("Failed to delete service: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({
                        "error": "Failed to delete service",
                        "details": e.to_string()
                    })),
                )
            }
        }
    }
}

pub trait OneInArea: Service {
    async fn get_all_in_area(
        pool: &PgPool,
        area_id: i64,
        entity_id: Uuid,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> anyhow::Result<PaginationResponse<Self>>;

    async fn get_all_in_area_handler(
        State(state): State<AppState>,
        Query(params): Query<ReadQuery>,
        Path((entity, area)): Path<(String, String)>,
    ) -> impl IntoResponse {
        let entity_uuid = match Uuid::parse_str(&entity) {
            Ok(uuid) => uuid,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("{{\"error\": \"Invalid entity UUID: {}\"}}", e),
                )
            }
        };

        let area_id = match area.parse::<i64>() {
            Ok(id) => id,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("{{\"error\": \"Invalid area ID: {}\"}}", e),
                )
            }
        };

        match Self::get_all_in_area(
            &state.pool,
            area_id,
            entity_uuid,
            params.offset.unwrap_or(0) as i64,
            params.limit.unwrap_or(10).min(100) as i64,
            params.sort.as_deref(),
            params.asc.unwrap_or(true),
        )
        .await
        {
            Ok(items) => match serde_json::to_string(&items) {
                Ok(json) => (StatusCode::OK, json),
                Err(e) => {
                    log::error!("Failed to serialize response: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        r#"{"error": "Failed to serialize response"}"#.to_string(),
                    )
                }
            },
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{{\"error\": \"{}\"}}", e),
            ),
        }
    }
}
