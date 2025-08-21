use serde::{Serialize, Deserialize};
use bson::oid::ObjectId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Connection {
    _id: ObjectId,
    entity: ObjectId, // Reference to the Entity
    name: String,
    description: Option<String>,
    r#type: ConnectionType,
    source_area: ObjectId, // Reference to the source Area
    target_area: ObjectId, // Reference to the target Area
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    Shuttle
}