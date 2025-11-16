use crate::schema::service::Service;
use async_trait::async_trait;

// Re-export from navign-shared
pub use navign_shared::{Area, Floor, FloorType};

#[async_trait]
impl Service for Area {
    fn get_id(&self) -> String {
        self.id.clone()
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
        "areas"
    }

    fn require_unique_name() -> bool {
        false
    }
}
