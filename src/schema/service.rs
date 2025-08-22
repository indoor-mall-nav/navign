use crate::AppState;
use crate::shared::ReadQuery;
use async_trait::async_trait;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bson::doc;
use bson::oid::ObjectId;
use futures::stream::TryStreamExt;
use log::info;
use mongodb::{Collection, Database};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;

#[async_trait]
#[allow(dead_code)]
pub trait Service: Serialize + DeserializeOwned + Send + Sync + Clone {
    fn get_id(&self) -> String;
    fn get_name(&self) -> String;
    fn set_id(&mut self, id: String);
    fn set_name(&mut self, name: String);
    fn get_description(&self) -> Option<String>;
    fn set_description(&mut self, description: Option<String>);
    fn get_collection_name() -> &'static str;
    fn require_unique_name() -> bool;

    async fn get_one_by_id(db: &Database, id: &str) -> Option<Self>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        let oid = ObjectId::parse_str(id).ok()?;
        collection
            .find_one(doc! { "_id": oid })
            .await
            .ok()
            .flatten()
    }

    async fn get_one_by_name(
        db: &Database,
        name: &str,
    ) -> Result<Option<Self>, mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        collection.find_one(doc! { "name": name }).await
    }

    async fn get_all(db: &Database) -> Result<Vec<Self>, mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        let cursor = collection.find(doc! {}).await?;
        cursor.try_collect::<Vec<Self>>().await
    }

    async fn get_with_pagination(
        db: &Database,
        page: u64,
        limit: u64,
        sort: Option<&str>,
        asc: bool,
    ) -> Result<Vec<Self>, mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        let options = mongodb::options::FindOptions::builder()
            .skip(page * limit)
            .limit(limit as i64)
            .sort(sort.map(|s| {
                if asc {
                    doc! { s: 1 }
                } else {
                    doc! { s: -1 }
                }
            }))
            .build();
        let cursor = collection.find(doc! {}).with_options(options).await?;
        cursor.try_collect::<Vec<Self>>().await
    }

    async fn create(&self, db: &Database) -> Result<ObjectId, mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        if Self::require_unique_name() {
            if let Some(existing) = Self::get_one_by_name(db, &self.get_name()).await? {
                return Err(mongodb::error::Error::custom(format!(
                    "An item with the name '{}' already exists.",
                    existing.get_name()
                )));
            }
        }
        let result = collection.insert_one(self.clone()).await;
        match result {
            Ok(insert_result) => {
                if let Some(id) = insert_result.inserted_id.as_object_id() {
                    Ok(id)
                } else {
                    Err(mongodb::error::Error::custom(
                        "Inserted ID is not an ObjectId".to_string(),
                    ))
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn update(&self, db: &Database) -> Result<(), mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        let oid = ObjectId::parse_str(&self.get_id()).map_err(|_| {
            mongodb::error::Error::custom("Invalid ObjectId format for update".to_string())
        })?;
        let result = collection
            .replace_one(doc! { "_id": oid }, self.clone())
            .await?;
        if result.matched_count == 0 {
            Err(mongodb::error::Error::custom(
                "No document found with the given ID".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    async fn delete_by_id(db: &Database, id: &str) -> Result<(), mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        let oid = ObjectId::parse_str(id).map_err(|_| {
            mongodb::error::Error::custom("Invalid ObjectId format for deletion".to_string())
        })?;
        let result = collection.delete_one(doc! { "_id": oid }).await?;
        if result.deleted_count == 0 {
            Err(mongodb::error::Error::custom(
                "No document found with the given ID".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    async fn delete_by_name(db: &Database, name: &str) -> Result<(), mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        let result = collection.delete_one(doc! { "name": name }).await?;
        if result.deleted_count == 0 {
            Err(mongodb::error::Error::custom(
                "No document found with the given name".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    async fn exists_by_name(db: &Database, name: &str) -> Result<bool, mongodb::error::Error>
    where
        Self: Sized,
    {
        Ok(Self::get_one_by_name(db, name).await?.is_some())
    }

    async fn search_by_name_pattern(
        db: &Database,
        pattern: &str,
        case_insensitive: bool,
    ) -> Result<Vec<Self>, mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        let options = if case_insensitive { "i" } else { "" };

        let filter = doc! {
            "name": {
                "$regex": pattern,
                "$options": options
            }
        };

        let cursor = collection.find(filter).await?;
        cursor.try_collect().await
    }

    async fn search_and_page_by_name_pattern(
        db: &Database,
        pattern: &str,
        page: u64,
        limit: u64,
        sort: Option<&str>,
        asc: bool,
        case_insensitive: bool,
    ) -> Result<Vec<Self>, mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        let options = if case_insensitive { "i" } else { "" };

        let filter = doc! {
            "name": {
                "$regex": pattern,
                "$options": options
            }
        };

        let find_options = mongodb::options::FindOptions::builder()
            .skip(page * limit)
            .limit(limit as i64)
            .sort(sort.map(|s| {
                if asc {
                    doc! { s: 1 }
                } else {
                    doc! { s: -1 }
                }
            }))
            .build();

        let cursor = collection.find(filter).with_options(find_options).await?;
        cursor.try_collect().await
    }

    async fn search_by_description_pattern(
        db: &Database,
        pattern: &str,
        case_insensitive: bool,
    ) -> Result<Vec<Self>, mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        let options = if case_insensitive { "i" } else { "" };

        let filter = doc! {
            "description": {
                "$regex": pattern,
                "$options": options
            }
        };

        let cursor = collection.find(filter).await?;
        cursor.try_collect().await
    }

    async fn bulk_create(
        db: &Database,
        services: Vec<Self>,
    ) -> Result<Vec<ObjectId>, mongodb::error::Error>
    where
        Self: Sized,
    {
        let collection: Collection<Self> = db.collection(Self::get_collection_name());
        let result = collection.insert_many(services).await?;

        Ok(result
            .inserted_ids
            .into_iter()
            .filter_map(|(_, bson)| bson.as_object_id())
            .collect())
    }

    async fn get_handler(
        State(state): State<AppState>,
        Query(ReadQuery {
            page,
            limit,
            query,
            sort,
            asc,
            case_sensitive,
        }): Query<ReadQuery>,
    ) -> impl IntoResponse {
        info!(
            "Handling GET request for services with query: {:?} in collection {}",
            query,
            Self::get_collection_name()
        );
        let db = &state.db;
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(0);
        let query = query.unwrap_or_default();
        let sort = sort.as_deref();
        let asc = asc.unwrap_or(true);
        let case_sensitive = case_sensitive.unwrap_or(false);
        info!(
            "Query parameters in {}: page={page}, limit={limit}, query='{query}', sort={sort:?}, asc={asc}, case_sensitive={case_sensitive}",
            Self::get_collection_name()
        );
        match Self::search_and_page_by_name_pattern(
            db,
            query.as_str(),
            page,
            limit,
            sort,
            asc,
            case_sensitive,
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
        axum::extract::Path(id): axum::extract::Path<String>,
    ) -> impl IntoResponse {
        info!("Handling GET request for service with ID: {}", id);
        let db = &state.db;
        match Self::get_one_by_id(db, &id).await {
            Some(service) => (StatusCode::OK, axum::Json(json!(service))),
            None => (
                StatusCode::NOT_FOUND,
                axum::Json(json!({
                    "error": "Service not found"
                })),
            ),
        }
    }

    async fn create_handler(
        State(state): State<AppState>,
        axum::Json(service): axum::Json<Self>,
    ) -> impl IntoResponse {
        let db = &state.db;
        match service.create(db).await {
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
        let db = &state.db;
        match service.update(db).await {
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
        axum::extract::Path(id): axum::extract::Path<String>,
    ) -> impl IntoResponse {
        let db = &state.db;
        match Self::delete_by_id(db, &id).await {
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
