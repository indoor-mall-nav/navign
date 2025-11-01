use crate::AppState;
use crate::schema::metadata::{PaginationResponse, PaginationResponseMetadata};
use crate::schema::service::{OneInArea, Service};
use crate::shared::ReadQuery;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use bson::doc;
use bson::oid::ObjectId;
use futures::TryStreamExt;
use mongodb::Database;
use std::str::FromStr;

// Re-export from navign-shared
pub use navign_shared::{Beacon, BeaconDevice, BeaconType};

impl Service for Beacon {
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
        "beacons"
    }

    fn require_unique_name() -> bool {
        true
    }
}

impl OneInArea for Beacon {}
