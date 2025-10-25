use crate::AppState;
use crate::schema::service::Service;
use async_trait::async_trait;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use bson::doc;
use bson::oid::ObjectId;
use futures::TryStreamExt;
use mongodb::Database;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Entity {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub r#type: EntityType,
    pub name: String,
    pub description: Option<String>,
    pub longitude_range: (f64, f64), // (min_longitude, max_longitude)
    pub latitude_range: (f64, f64),  // (min_latitude, max_latitude)
    pub altitude_range: Option<(f64, f64)>, // (min_altitude, max_altitude)
    pub nation: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub tags: Vec<String>,
    pub created_at: i64, // Timestamp in milliseconds
    pub updated_at: i64, // Timestamp in milliseconds
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum EntityType {
    #[default]
    Mall,
    Transportation,
    School,
    Hospital,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Mall => write!(f, "Mall"),
            EntityType::Transportation => write!(f, "Transportation"),
            EntityType::School => write!(f, "School"),
            EntityType::Hospital => write!(f, "Hospital"),
        }
    }
}

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
    fn get_id(&self) -> String {
        self.id.to_hex()
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

    fn get_collection_name() -> &'static str {
        "entities"
    }

    fn require_unique_name() -> bool {
        false
    }
}

#[async_trait]
pub(crate) trait EntityServiceAddons
where
    Self: Sized + Send + Sync + Service,
{
    async fn search_entity_by_fields(
        db: &Database,
        nation: Option<String>,
        region: Option<String>,
        city: Option<String>,
        name: Option<String>,
        longitude: Option<f64>,
        latitude: Option<f64>,
    ) -> Result<Vec<Self>, mongodb::error::Error> {
        let collection = db.collection::<Self>("entities");
        let mut filter = doc! { "name": {
            "$regex": name.unwrap_or_default(),
            "$options": "i" // Case-insensitive search
        } };

        if let Some(n) = nation {
            filter.insert("nation", n);
        }
        if let Some(r) = region {
            filter.insert("region", r);
        }
        if let Some(c) = city {
            filter.insert("city", c);
        }
        if let Some(lon) = longitude {
            filter.insert("longitude_range.0", doc! { "$lte": lon });
            filter.insert("longitude_range.1", doc! { "$gte": lon });
        }
        if let Some(lat) = latitude {
            filter.insert("latitude_range.0", doc! { "$lte": lat });
            filter.insert("latitude_range.1", doc! { "$gte": lat });
        }

        let cursor = collection.find(filter).await?;
        let entities: Vec<Self> = cursor.try_collect().await?;
        Ok(entities)
    }

    async fn search_entity_handler(
        State(db): State<AppState>,
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
        match Self::search_entity_by_fields(&db.db, nation, region, city, name, longitude, latitude)
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
