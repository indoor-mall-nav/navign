use crate::schema::metadata::{PaginationResponse, PaginationResponseMetadata};
use crate::schema::service::{SearchQueryParams, Service};
use crate::AppState;
use async_trait::async_trait;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// Re-export from navign-shared
pub use navign_shared::{Entity, EntityType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityQuery {
    nation: Option<String>,
    region: Option<String>,
    city: Option<String>,
    name: Option<String>,
    longitude: Option<f64>,
    latitude: Option<f64>,
}

#[async_trait]
impl Service for Entity {
    type Id = Uuid;

    fn get_id(&self) -> Uuid {
        self.id.expect("Entity must have an ID")
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    fn get_table_name() -> &'static str {
        "entities"
    }

    fn require_unique_name() -> bool {
        false
    }

    async fn get_one_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Entity,
            r#"
            SELECT
                id as "id: Uuid",
                type as "type: String",
                name,
                description,
                longitude_min,
                longitude_max,
                latitude_min,
                latitude_max,
                altitude_min,
                altitude_max,
                nation,
                region,
                city,
                tags as "tags: sqlx::types::JsonValue",
                created_at,
                updated_at
            FROM entities WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map(|opt| opt.map(|e| Entity {
            id: Some(e.id.unwrap()),
            r#type: match e.r#type.as_str() {
                "mall" => EntityType::Mall,
                "transportation" => EntityType::Transportation,
                "school" => EntityType::School,
                "hospital" => EntityType::Hospital,
                _ => EntityType::Mall,
            },
            name: e.name,
            description: e.description,
            longitude_range: (e.longitude_min, e.longitude_max),
            latitude_range: (e.latitude_min, e.latitude_max),
            altitude_range: match (e.altitude_min, e.altitude_max) {
                (Some(min), Some(max)) => Some((min, max)),
                _ => None,
            },
            nation: e.nation,
            region: e.region,
            city: e.city,
            tags: serde_json::from_value(e.tags).unwrap_or_default(),
            created_at: e.created_at,
            updated_at: e.updated_at,
        }))
    }

    async fn get_one_by_name(pool: &PgPool, name: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Entity,
            r#"
            SELECT
                id as "id: Uuid",
                type as "type: String",
                name,
                description,
                longitude_min,
                longitude_max,
                latitude_min,
                latitude_max,
                altitude_min,
                altitude_max,
                nation,
                region,
                city,
                tags as "tags: sqlx::types::JsonValue",
                created_at,
                updated_at
            FROM entities WHERE name = $1 LIMIT 1
            "#,
            name
        )
        .fetch_optional(pool)
        .await
        .map(|opt| opt.map(|e| Entity {
            id: Some(e.id.unwrap()),
            r#type: match e.r#type.as_str() {
                "mall" => EntityType::Mall,
                "transportation" => EntityType::Transportation,
                "school" => EntityType::School,
                "hospital" => EntityType::Hospital,
                _ => EntityType::Mall,
            },
            name: e.name,
            description: e.description,
            longitude_range: (e.longitude_min, e.longitude_max),
            latitude_range: (e.latitude_min, e.latitude_max),
            altitude_range: match (e.altitude_min, e.altitude_max) {
                (Some(min), Some(max)) => Some((min, max)),
                _ => None,
            },
            nation: e.nation,
            region: e.region,
            city: e.city,
            tags: serde_json::from_value(e.tags).unwrap_or_default(),
            created_at: e.created_at,
            updated_at: e.updated_at,
        }))
    }

    async fn get_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Entity,
            r#"
            SELECT
                id as "id: Uuid",
                type as "type: String",
                name,
                description,
                longitude_min,
                longitude_max,
                latitude_min,
                latitude_max,
                altitude_min,
                altitude_max,
                nation,
                region,
                city,
                tags as "tags: sqlx::types::JsonValue",
                created_at,
                updated_at
            FROM entities
            "#
        )
        .fetch_all(pool)
        .await
        .map(|entities| {
            entities.into_iter().map(|e| Entity {
                id: Some(e.id.unwrap()),
                r#type: match e.r#type.as_str() {
                    "mall" => EntityType::Mall,
                    "transportation" => EntityType::Transportation,
                    "school" => EntityType::School,
                    "hospital" => EntityType::Hospital,
                    _ => EntityType::Mall,
                },
                name: e.name,
                description: e.description,
                longitude_range: (e.longitude_min, e.longitude_max),
                latitude_range: (e.latitude_min, e.latitude_max),
                altitude_range: match (e.altitude_min, e.altitude_max) {
                    (Some(min), Some(max)) => Some((min, max)),
                    _ => None,
                },
                nation: e.nation,
                region: e.region,
                city: e.city,
                tags: serde_json::from_value(e.tags).unwrap_or_default(),
                created_at: e.created_at,
                updated_at: e.updated_at,
            }).collect()
        })
    }

    async fn get_with_pagination(
        pool: &PgPool,
        page: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = page * limit;
        let sort_column = sort.unwrap_or("name");
        let order = if asc { "ASC" } else { "DESC" };

        let query = format!(
            r#"
            SELECT
                id, type, name, description,
                longitude_min, longitude_max,
                latitude_min, latitude_max,
                altitude_min, altitude_max,
                nation, region, city, tags,
                created_at, updated_at
            FROM entities
            ORDER BY {} {}
            LIMIT $1 OFFSET $2
            "#,
            sort_column, order
        );

        sqlx::query_as(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    async fn create(&self, pool: &PgPool) -> Result<Uuid, sqlx::Error> {
        let entity_type = match self.r#type {
            EntityType::Mall => "mall",
            EntityType::Transportation => "transportation",
            EntityType::School => "school",
            EntityType::Hospital => "hospital",
        };

        let tags_json = serde_json::to_value(&self.tags).unwrap_or(serde_json::json!([]));
        let now = chrono::Utc::now().timestamp_millis();

        let result = sqlx::query!(
            r#"
            INSERT INTO entities (
                type, name, description,
                longitude_min, longitude_max,
                latitude_min, latitude_max,
                altitude_min, altitude_max,
                nation, region, city, tags,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id
            "#,
            entity_type,
            self.name,
            self.description,
            self.longitude_range.0,
            self.longitude_range.1,
            self.latitude_range.0,
            self.latitude_range.1,
            self.altitude_range.as_ref().map(|r| r.0),
            self.altitude_range.as_ref().map(|r| r.1),
            self.nation,
            self.region,
            self.city,
            tags_json,
            now,
            now
        )
        .fetch_one(pool)
        .await?;

        Ok(result.id)
    }

    async fn update(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let entity_type = match self.r#type {
            EntityType::Mall => "mall",
            EntityType::Transportation => "transportation",
            EntityType::School => "school",
            EntityType::Hospital => "hospital",
        };

        let tags_json = serde_json::to_value(&self.tags).unwrap_or(serde_json::json!([]));
        let now = chrono::Utc::now().timestamp_millis();
        let id = self.id.expect("Entity must have an ID for update");

        sqlx::query!(
            r#"
            UPDATE entities SET
                type = $1, name = $2, description = $3,
                longitude_min = $4, longitude_max = $5,
                latitude_min = $6, latitude_max = $7,
                altitude_min = $8, altitude_max = $9,
                nation = $10, region = $11, city = $12,
                tags = $13, updated_at = $14
            WHERE id = $15
            "#,
            entity_type,
            self.name,
            self.description,
            self.longitude_range.0,
            self.longitude_range.1,
            self.latitude_range.0,
            self.latitude_range.1,
            self.altitude_range.as_ref().map(|r| r.0),
            self.altitude_range.as_ref().map(|r| r.1),
            self.nation,
            self.region,
            self.city,
            tags_json,
            now,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM entities WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn delete_by_name(pool: &PgPool, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM entities WHERE name = $1", name)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn search_and_page_by_name_pattern(
        pool: &PgPool,
        params: SearchQueryParams<'_>,
    ) -> Result<PaginationResponse<Self>, sqlx::Error> {
        let sort_column = params.sort.unwrap_or("name");
        let order = if params.asc { "ASC" } else { "DESC" };
        let pattern = if params.case_insensitive {
            format!("%{}%", params.pattern.to_lowercase())
        } else {
            format!("%{}%", params.pattern)
        };

        let count_query = if params.case_insensitive {
            "SELECT COUNT(*) as count FROM entities WHERE LOWER(name) LIKE $1"
        } else {
            "SELECT COUNT(*) as count FROM entities WHERE name LIKE $1"
        };

        let total: i64 = sqlx::query_scalar(count_query)
            .bind(&pattern)
            .fetch_one(pool)
            .await?;

        let query = format!(
            r#"
            SELECT
                id, type, name, description,
                longitude_min, longitude_max,
                latitude_min, latitude_max,
                altitude_min, altitude_max,
                nation, region, city, tags,
                created_at, updated_at
            FROM entities
            WHERE {} LIKE $1
            ORDER BY {} {}
            LIMIT $2 OFFSET $3
            "#,
            if params.case_insensitive { "LOWER(name)" } else { "name" },
            sort_column,
            order
        );

        let items: Vec<Self> = sqlx::query_as(&query)
            .bind(&pattern)
            .bind(params.limit)
            .bind(params.offset)
            .fetch_all(pool)
            .await?;

        Ok(PaginationResponse {
            items,
            metadata: PaginationResponseMetadata {
                total,
                offset: params.offset,
                limit: params.limit,
            },
        })
    }

    async fn search_by_description_pattern(
        pool: &PgPool,
        pattern: &str,
        case_insensitive: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let pattern = if case_insensitive {
            format!("%{}%", pattern.to_lowercase())
        } else {
            format!("%{}%", pattern)
        };

        let query = if case_insensitive {
            r#"
            SELECT
                id, type, name, description,
                longitude_min, longitude_max,
                latitude_min, latitude_max,
                altitude_min, altitude_max,
                nation, region, city, tags,
                created_at, updated_at
            FROM entities
            WHERE LOWER(description) LIKE $1
            "#
        } else {
            r#"
            SELECT
                id, type, name, description,
                longitude_min, longitude_max,
                latitude_min, latitude_max,
                altitude_min, altitude_max,
                nation, region, city, tags,
                created_at, updated_at
            FROM entities
            WHERE description LIKE $1
            "#
        };

        sqlx::query_as(query)
            .bind(&pattern)
            .fetch_all(pool)
            .await
    }

    async fn bulk_create(
        pool: &PgPool,
        entities: Vec<Self>,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        let mut ids = Vec::with_capacity(entities.len());

        for entity in entities {
            let id = entity.create(pool).await?;
            ids.push(id);
        }

        Ok(ids)
    }
}

#[async_trait]
pub(crate) trait EntityServiceAddons
where
    Self: Sized + Send + Sync + Service,
{
    async fn search_entity_by_fields(
        pool: &PgPool,
        nation: Option<String>,
        region: Option<String>,
        city: Option<String>,
        name: Option<String>,
        longitude: Option<f64>,
        latitude: Option<f64>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let name_pattern = format!("%{}%", name.unwrap_or_default());

        let mut query_str = String::from(
            r#"
            SELECT
                id, type, name, description,
                longitude_min, longitude_max,
                latitude_min, latitude_max,
                altitude_min, altitude_max,
                nation, region, city, tags,
                created_at, updated_at
            FROM entities
            WHERE LOWER(name) LIKE LOWER($1)
            "#
        );

        let mut bind_count = 2;

        if nation.is_some() {
            query_str.push_str(&format!(" AND nation = ${}", bind_count));
            bind_count += 1;
        }
        if region.is_some() {
            query_str.push_str(&format!(" AND region = ${}", bind_count));
            bind_count += 1;
        }
        if city.is_some() {
            query_str.push_str(&format!(" AND city = ${}", bind_count));
            bind_count += 1;
        }
        if longitude.is_some() {
            query_str.push_str(&format!(" AND longitude_min <= ${} AND longitude_max >= ${}", bind_count, bind_count));
            bind_count += 1;
        }
        if latitude.is_some() {
            query_str.push_str(&format!(" AND latitude_min <= ${} AND latitude_max >= ${}", bind_count, bind_count));
        }

        let mut query = sqlx::query_as(&query_str).bind(&name_pattern);

        if let Some(n) = nation {
            query = query.bind(n);
        }
        if let Some(r) = region {
            query = query.bind(r);
        }
        if let Some(c) = city {
            query = query.bind(c);
        }
        if let Some(lon) = longitude {
            query = query.bind(lon);
        }
        if let Some(lat) = latitude {
            query = query.bind(lat);
        }

        query.fetch_all(pool).await
    }

    async fn search_entity_handler(
        State(state): State<AppState>,
        Query(entity_query): Query<EntityQuery>,
    ) -> impl IntoResponse {
        let EntityQuery {
            nation,
            region,
            city,
            name,
            longitude,
            latitude,
        } = entity_query;
        match Self::search_entity_by_fields(&state.db, nation, region, city, name, longitude, latitude)
            .await
        {
            Ok(entities) => axum::Json(entities),
            Err(e) => {
                eprintln!("Error searching entities: {}", e);
                axum::Json(vec![]) // Return an empty vector on error
            }
        }
    }
}

impl EntityServiceAddons for Entity {}
