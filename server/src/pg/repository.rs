use crate::error::ServerError;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use navign_shared::schema::repository::{IntRepository, IntRepositoryInArea, UuidRepository};
use serde::{Deserialize, Serialize};
use sqlx::Postgres;

macro_rules! extract_uuid {
    ($entity:ident) => {
        match uuid::Uuid::parse_str($entity.as_str()) {
            Ok(uuid) => uuid,
            Err(_) => return ServerError::InvalidInput("Invalid UUID".to_string()).into_response(),
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
    async fn crud_read_one(
        State(app): State<AppState>,
        Path((entity, id)): Path<(String, String)>,
    ) -> Response {
        let entity_uuid = extract_uuid!(entity);

        match Self::get_by_id(
            app.pg_pool.inner(),
            id.parse().unwrap_or_default(),
            entity_uuid,
        )
        .await
        {
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

        let entity_uuid = extract_uuid!(entity);

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
            Ok(items) => (StatusCode::OK, serde_json::to_string(&items).unwrap()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_create(
        State(app): State<AppState>,
        Path(entity): Path<String>,
        Json(data): Json<Self>,
    ) -> Response {
        let entity_uuid = extract_uuid!(entity);

        match Self::create(app.pg_pool.inner(), &data, entity_uuid).await {
            Ok(_) => ("Item created".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_update(
        State(app): State<AppState>,
        Path(entity): Path<String>,
        Json(data): Json<Self>,
    ) -> Response {
        let entity_uuid = extract_uuid!(entity);

        match Self::update(app.pg_pool.inner(), &data, entity_uuid).await {
            Ok(_) => (StatusCode::OK, "Item updated".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_delete(
        State(app): State<AppState>,
        Path((entity, id)): Path<(String, String)>,
    ) -> Response {
        let entity_uuid = extract_uuid!(entity);

        match Self::delete(
            app.pg_pool.inner(),
            id.parse().unwrap_or_default(),
            entity_uuid,
        )
        .await
        {
            Ok(_) => (StatusCode::OK, "Item deleted".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }
}

#[async_trait::async_trait]
pub trait IntCrudRepositoryInArea: IntRepositoryInArea<Postgres> + Serialize + Send + Sync {
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

        let entity_uuid = extract_uuid!(entity);

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
            Ok(items) => (StatusCode::OK, serde_json::to_string(&items).unwrap()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }
}

#[async_trait::async_trait]
pub trait UuidCrudRepository: UuidRepository<Postgres> + Serialize + Send + Sync {
    async fn crud_read_one(State(app): State<AppState>, Path(uuid): Path<String>) -> Response {
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
            Ok(items) => (StatusCode::OK, serde_json::to_string(&items).unwrap()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_create(State(app): State<AppState>, Json(data): Json<Self>) -> Response {
        match Self::create(app.pg_pool.inner(), &data).await {
            Ok(_) => (StatusCode::CREATED, "Item created".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_update(State(app): State<AppState>, Json(data): Json<Self>) -> Response {
        match Self::update(app.pg_pool.inner(), &data).await {
            Ok(_) => (StatusCode::OK, "Item updated".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }

    async fn crud_delete(State(app): State<AppState>, Path(uuid): Path<String>) -> Response {
        let item_uuid = extract_uuid!(uuid);

        match Self::delete(app.pg_pool.inner(), item_uuid).await {
            Ok(_) => (StatusCode::OK, "Item deleted".to_string()).into_response(),
            Err(err) => ServerError::Database(err).into_response(),
        }
    }
}
