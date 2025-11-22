use crate::error::ServerError;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use navign_shared::{IntRepository, IntRepositoryInArea, UuidRepository};
use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResponseMetadata {
    pub total_items: u64,
    pub current_offset: u64,
    pub current_limit: u64,
    pub next_page_url: Option<String>,
    pub prev_page_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResponse<T> {
    pub metadata: PaginationResponseMetadata,
    pub data: Vec<T>,
}

macro_rules! extract_uuid {
    ($id:ident) => {
        match uuid::Uuid::parse_str($id.as_str()) {
            Ok(uuid) => uuid,
            Err(_) => return ServerError::InvalidInput("Invalid UUID".to_string()).into_response(),
        }
    };
}

macro_rules! extract_i32id {
    ($id:ident) => {
        match $id.parse::<i32>() {
            Ok(id) => id,
            Err(_) => {
                return ServerError::InvalidInput("Invalid i32 ID".to_string()).into_response()
            }
        }
    };
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub case_insensitive: bool,
    #[serde(default)]
    pub offset: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub sort: Option<String>,
    #[serde(default = "default_asc")]
    pub asc: bool,
}

fn default_limit() -> i64 {
    50
}

fn default_asc() -> bool {
    false
}

#[async_trait::async_trait]
pub trait IntCrudRepository: IntRepository<Postgres> + Serialize + Send + Sync {
    const API_ENDPOINT: &'static str;
    const WRAPPER_NAME: &'static str;

    async fn crud_read_one(
        State(app): State<AppState>,
        Path((entity, id)): Path<(String, String)>,
    ) -> Response {
        info!(
            "GET /api/{}/{entity}/{}/{id}/",
            Self::WRAPPER_NAME,
            Self::API_ENDPOINT
        );

        let entity_uuid = extract_uuid!(entity);
        let item_id = extract_i32id!(id);

        match Self::get_by_id(app.pg_pool.inner(), item_id, entity_uuid).await {
            Ok(Some(item)) => {
                (StatusCode::OK, serde_json::to_string(&item).unwrap()).into_response()
            }
            Ok(None) => ServerError::NotFound("Item not found".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_search(
        State(app): State<AppState>,
        Path(entity): Path<String>,
        Query(query): Query<SearchParams>,
    ) -> Response {
        let SearchParams {
            query,
            case_insensitive,
            offset,
            limit,
            sort,
            asc,
        } = query;

        info!(
            "GET /api/{}/{entity}/{}/?query={query}&case_insensitive={case_insensitive}&offset={offset}&limit={limit}&sort={sort:?}&asc={asc}",
            Self::WRAPPER_NAME,
            Self::API_ENDPOINT
        );

        let entity_uuid = extract_uuid!(entity);

        let document_count = match Self::count(
            app.pg_pool.inner(),
            entity_uuid,
            query.as_str(),
            case_insensitive,
        )
        .await
        {
            Ok(count) => count,
            Err(err) => return ServerError::Database(err).into_response(),
        };

        match Self::search(
            app.pg_pool.inner(),
            query.as_str(),
            case_insensitive,
            offset,
            limit,
            sort.as_deref(),
            asc,
            entity_uuid,
        )
        .await
        {
            Ok(items) => {
                let next_page_url = if (offset + limit) < document_count {
                    Some(format!(
                        "/api/{}/{}/?query={}&case_insensitive={}&offset={}&limit={}&sort={:?}&asc={}",
                        Self::WRAPPER_NAME,
                        Self::API_ENDPOINT,
                        query,
                        case_insensitive,
                        offset + limit,
                        limit,
                        sort,
                        asc
                    ))
                } else {
                    None
                };
                let prev_page_url = if offset - limit >= 0 {
                    Some(format!(
                        "/api/{}/{}/?query={}&case_insensitive={}&offset={}&limit={}&sort={:?}&asc={}",
                        Self::WRAPPER_NAME,
                        Self::API_ENDPOINT,
                        query,
                        case_insensitive,
                        offset - limit,
                        limit,
                        sort,
                        asc
                    ))
                } else {
                    None
                };
                let metadata = PaginationResponseMetadata {
                    total_items: document_count as u64,
                    current_offset: offset as u64,
                    current_limit: limit as u64,
                    next_page_url,
                    prev_page_url,
                };
                let result = PaginationResponse {
                    metadata,
                    data: items,
                };
                (StatusCode::OK, serde_json::to_string(&result).unwrap()).into_response()
            }
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_create(
        State(app): State<AppState>,
        Path(entity): Path<String>,
        Json(data): Json<Self>,
    ) -> Response {
        info!(
            "POST /api/{}/{entity}/{}/",
            Self::WRAPPER_NAME,
            Self::API_ENDPOINT
        );
        let entity_uuid = extract_uuid!(entity);

        match Self::create(app.pg_pool.inner(), &data, entity_uuid).await {
            Ok(_) => ("Item created".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_update(
        State(app): State<AppState>,
        Path((entity, id)): Path<(String, String)>,
        Json(data): Json<Self>,
    ) -> Response {
        info!(
            "PUT /api/{}/{entity}/{}/{id}/",
            Self::WRAPPER_NAME,
            Self::API_ENDPOINT
        );

        let entity_uuid = extract_uuid!(entity);
        let item_id = extract_i32id!(id);

        match Self::update(app.pg_pool.inner(), item_id, &data, entity_uuid).await {
            Ok(_) => (StatusCode::OK, "Item updated".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_delete(
        State(app): State<AppState>,
        Path((entity, id)): Path<(String, String)>,
    ) -> Response {
        info!(
            "DELETE /api/{}/{entity}/{}/{id}/",
            Self::WRAPPER_NAME,
            Self::API_ENDPOINT
        );
        let entity_uuid = extract_uuid!(entity);
        let item_id = extract_i32id!(id);

        match Self::delete(app.pg_pool.inner(), item_id, entity_uuid).await {
            Ok(_) => (StatusCode::OK, "Item deleted".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }
}

#[async_trait::async_trait]
pub trait IntCrudRepositoryInArea: IntRepositoryInArea<Postgres> + Serialize + Send + Sync {
    const API_ENDPOINT: &'static str;
    const WRAPPER_NAME: &'static str;

    async fn crud_search_in_area(
        State(app): State<AppState>,
        Path((entity, area)): Path<(String, i32)>,
        Query(params): Query<SearchParams>,
    ) -> Response {
        let SearchParams {
            query,
            case_insensitive,
            offset,
            limit,
            sort,
            asc,
        } = params;

        info!(
            "GET /api/{}/{entity}/areas/{area}/{}/?query={query}&case_insensitive={case_insensitive}&offset={offset}&limit={limit}&sort={sort:?}&asc={asc}",
            Self::WRAPPER_NAME,
            Self::API_ENDPOINT
        );

        let entity_uuid = extract_uuid!(entity);

        let document_count = match Self::count_in_area(
            app.pg_pool.inner(),
            entity_uuid,
            area,
            query.as_str(),
            case_insensitive,
        )
        .await
        {
            Ok(count) => count,
            Err(err) => return ServerError::Database(err).into_response(),
        };

        match Self::search_in_area(
            app.pg_pool.inner(),
            query.as_str(),
            case_insensitive,
            offset,
            limit,
            sort.as_deref(),
            asc,
            area,
            entity_uuid,
        )
        .await
        {
            Ok(items) => {
                let next_page_url = if (offset + limit) < document_count {
                    Some(format!(
                        "/api/{}/{}/?query={}&case_insensitive={}&offset={}&limit={}&sort={:?}&asc={}",
                        Self::WRAPPER_NAME,
                        Self::API_ENDPOINT,
                        query,
                        case_insensitive,
                        offset + limit,
                        limit,
                        sort,
                        asc
                    ))
                } else {
                    None
                };
                let prev_page_url = if offset - limit >= 0 {
                    Some(format!(
                        "/api/{}/{}/?query={}&case_insensitive={}&offset={}&limit={}&sort={:?}&asc={}",
                        Self::WRAPPER_NAME,
                        Self::API_ENDPOINT,
                        query,
                        case_insensitive,
                        offset - limit,
                        limit,
                        sort,
                        asc
                    ))
                } else {
                    None
                };
                let metadata = PaginationResponseMetadata {
                    total_items: document_count as u64,
                    current_offset: offset as u64,
                    current_limit: limit as u64,
                    next_page_url,
                    prev_page_url,
                };
                let result = PaginationResponse {
                    metadata,
                    data: items,
                };
                (StatusCode::OK, serde_json::to_string(&result).unwrap()).into_response()
            }
            Err(err) => ServerError::Database(err).into_response(),
        }
    }
}

#[async_trait::async_trait]
pub trait UuidCrudRepository: UuidRepository<Postgres> + Serialize + Send + Sync {
    const API_ENDPOINT: &'static str;

    async fn crud_read_one(State(app): State<AppState>, Path(uuid): Path<String>) -> Response {
        info!("GET /api/{}/{} /", Self::API_ENDPOINT, uuid);

        let item_uuid = extract_uuid!(uuid);

        match Self::get_by_uuid(app.pg_pool.inner(), item_uuid).await {
            Ok(Some(item)) => {
                (StatusCode::OK, serde_json::to_string(&item).unwrap()).into_response()
            }
            Ok(None) => ServerError::NotFound("Item not found".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_search(
        State(app): State<AppState>,
        Query(params): Query<SearchParams>,
    ) -> Response {
        let SearchParams {
            query,
            case_insensitive,
            offset,
            limit,
            sort,
            asc,
        } = params;

        info!(
            "GET /api/{}/?query={query}&case_insensitive={case_insensitive}&offset={offset}&limit={limit}&sort={sort:?}&asc={asc}",
            Self::API_ENDPOINT
        );

        let document_count =
            match Self::count(app.pg_pool.inner(), query.as_str(), case_insensitive).await {
                Ok(count) => count,
                Err(err) => return ServerError::Database(err).into_response(),
            };

        match Self::search(
            app.pg_pool.inner(),
            query.as_str(),
            case_insensitive,
            offset,
            limit,
            sort.as_deref(),
            asc,
        )
        .await
        {
            Ok(items) => {
                let next_page_url = if (offset + limit) < document_count {
                    Some(format!(
                        "/api/{}/?query={}&case_insensitive={}&offset={}&limit={}&sort={:?}&asc={}",
                        Self::API_ENDPOINT,
                        query,
                        case_insensitive,
                        offset + limit,
                        limit,
                        sort,
                        asc
                    ))
                } else {
                    None
                };
                let prev_page_url = if offset - limit >= 0 {
                    Some(format!(
                        "/api/{}/?query={}&case_insensitive={}&offset={}&limit={}&sort={:?}&asc={}",
                        Self::API_ENDPOINT,
                        query,
                        case_insensitive,
                        offset - limit,
                        limit,
                        sort,
                        asc
                    ))
                } else {
                    None
                };
                let metadata = PaginationResponseMetadata {
                    total_items: document_count as u64,
                    current_offset: offset as u64,
                    current_limit: limit as u64,
                    next_page_url,
                    prev_page_url,
                };
                let result = PaginationResponse {
                    metadata,
                    data: items,
                };
                (StatusCode::OK, serde_json::to_string(&result).unwrap()).into_response()
            }
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_create(State(app): State<AppState>, Json(data): Json<Self>) -> Response {
        info!("POST /api/{}/", Self::API_ENDPOINT);

        match Self::create(app.pg_pool.inner(), &data).await {
            Ok(_) => (StatusCode::CREATED, "Item created".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_update(
        State(app): State<AppState>,
        Path(uuid): Path<String>,
        Json(data): Json<Self>,
    ) -> Response {
        info!("PUT /api/{}/{uuid}/", Self::API_ENDPOINT);

        let uuid = extract_uuid!(uuid);

        match Self::update(app.pg_pool.inner(), uuid, &data).await {
            Ok(_) => (StatusCode::OK, "Item updated".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_delete(State(app): State<AppState>, Path(uuid): Path<String>) -> Response {
        info!("DELETE /api/{}/{uuid}/", Self::API_ENDPOINT);
        let item_uuid = extract_uuid!(uuid);

        match Self::delete(app.pg_pool.inner(), item_uuid).await {
            Ok(_) => (StatusCode::OK, "Item deleted".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }
}
