use crate::schema::service::Service;

// Re-export from navign-shared
pub use navign_shared::{ConnectedArea, Connection, ConnectionType};

impl Service for Connection {
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
        "connections"
    }

    fn require_unique_name() -> bool {
        true
    }
}
