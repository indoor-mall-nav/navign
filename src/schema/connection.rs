use crate::schema::service::Service;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Connection {
    _id: ObjectId,
    /// Reference to the Entity
    entity: ObjectId,
    name: String,
    description: Option<String>,
    r#type: ConnectionType,
    /// List of Area IDs that this connection links
    connected_areas: Vec<ObjectId>,
    /// List of `(start_time, end_time)` in milliseconds on a 24-hour clock
    available_period: Vec<(i64, i64)>,
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Represents the type of connection between areas or entities.
pub enum ConnectionType {
    /// A connection that allows people to pass through, such as a door or gate.
    /// Usually involve authentication or access control.
    Gate,
    /// A connection that allows people to move between different areas, such as a hallway or corridor.
    Escalator,
    /// A connection that allows people to move between different levels, such as stairs or elevators.
    Elevator,
    /// A connection that allows people to move between different areas, such as a pathway or tunnel.
    Stairs,
    /// Like in Hong Kong International Airport, Singapore Changi Airport, or Shanghai Pudong International Airport.
    /// There is a dedicated transportation system that connects different terminals or areas.
    Rail,
    /// Shuttle bus.
    Shuttle,
}

impl Service for Connection {
    fn get_id(&self) -> String {
        self._id.to_hex()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_id(&mut self, id: String) {
        self._id = ObjectId::parse_str(&id).expect("Invalid ObjectId format");
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
