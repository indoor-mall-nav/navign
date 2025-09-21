//! # Cross Area Navigation
//!
//! This module provides functionalities to find the best route between different areas,
//! considering various factors such as distance, connections, and user preferences.
//!
//! I think that rather than sticking to Dijkstra, we have a better assumption-based logic (you can refer to the project files for better term learning):
//! 1. In different areas that can connect: Prefer entering the target area before any other actions. In malls, we may always head to the target floor first.
//! 2. In a certain area that has only one entry: Let the entry area be an "agent," and use the agent area as the target.
//!
//! When talking about the transportation type,
//! 1. If you include a basement that is a parking lot, choose an elevator. (For other purpose just ignore.)
//! 2. If the floor diff is > three floors, choose an elevator rather than an escalator.
//! 3. If ≤ three floors, choose an escalator rather than an elevator.
use crate::kernel::route::instructions::RouteInstruction;
use crate::schema::area::{Area, Floor, FloorType};
use crate::schema::connection::{Connection, ConnectionType};
use bson::oid::ObjectId;
use std::collections::{HashMap, HashSet, VecDeque};
use mongodb::Database;
use crate::schema::Service;

/// Represents a route segment between areas
#[derive(Debug, Clone)]
pub struct RouteSegment {
    pub from_area: ObjectId,
    pub to_area: ObjectId,
    pub connection: ObjectId,
    pub connection_type: ConnectionType,
    pub instructions: Vec<RouteInstruction>,
}

/// Cross area navigation router
pub struct CrossAreaRouter {
    areas: HashMap<ObjectId, Area>,
    connections: HashMap<ObjectId, Connection>,
    area_connections: HashMap<ObjectId, Vec<ObjectId>>, // area_id -> connection_ids
    connection_graph: HashMap<ObjectId, Vec<ObjectId>>, // area_id -> connected_area_ids
}

impl CrossAreaRouter {
    /// Create a new cross area router
    pub fn new() -> Self {
        Self {
            areas: HashMap::new(),
            connections: HashMap::new(),
            area_connections: HashMap::new(),
            connection_graph: HashMap::new(),
        }
    }

    /// Initialize router with areas and connections
    pub fn initialize(&mut self, areas: Vec<Area>, connections: Vec<Connection>) -> Result<(), String> {
        // Store areas
        for area in areas {
            let area_id = area.get_object_id();
            self.areas.insert(area_id, area);
        }

        // Store connections and build graph
        for connection in connections {
            let connection_id = connection.get_object_id();
            let connected_areas = connection.get_connected_areas();

            self.connections.insert(connection_id, connection.clone());

            // Build area connections mapping
            connected_areas.iter().for_each(|(area_id, _, _)| {
                self.area_connections
                    .entry(*area_id)
                    .or_insert_with(Vec::new)
                    .push(connection_id);
            });

            // Build connection graph (area to area mapping)
            for i in 0..connected_areas.len() {
                for j in 0..connected_areas.len() {
                    if i != j {
                        let from_area = connected_areas[i].0;
                        let to_area = connected_areas[j].0;

                        self.connection_graph
                            .entry(from_area)
                            .or_insert_with(Vec::new)
                            .push(to_area);
                    }
                }
            }
        }

        Ok(())
    }

    /// Find route between two areas using assumption-based logic
    pub async fn find_cross_area_route(
        &self,
        start_area_id: ObjectId,
        end_area_id: ObjectId,
        db: &Database,
    ) -> Result<Vec<RouteSegment>, String> {
        // If start and end are the same area
        if start_area_id == end_area_id {
            return Ok(Vec::new());
        }

        // Find path using BFS with preference logic
        let path = self.find_area_path(start_area_id, end_area_id, db).await?;

        // Convert path to route segments with optimal connection selection
        self.build_route_segments(path)
    }

    /// Find path between areas using BFS with mall-specific preferences
    async fn find_area_path(&self, start: ObjectId, end: ObjectId, db: &Database) -> Result<Vec<ObjectId>, String> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(current_area) = queue.pop_front() {
            if current_area == end {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = end;

                while let Some(&prev) = parent.get(&current) {
                    path.push(current);
                    current = prev;
                }
                path.push(start);
                path.reverse();

                return Ok(path);
            }

            // Get connected areas with preference logic
            if let Some(connected_areas) = self.connection_graph.get(&current_area) {
                let mut prioritized_areas = self.prioritize_connected_areas(
                    current_area,
                    connected_areas,
                    end,
                    db
                ).await?;

                for &next_area in &prioritized_areas {
                    if !visited.contains(&next_area) {
                        visited.insert(next_area);
                        parent.insert(next_area, current_area);
                        queue.push_back(next_area);
                    }
                }
            }
        }

        Err("No path found between areas".to_string())
    }

    /// Prioritize connected areas based on mall navigation preferences
    async fn prioritize_connected_areas(
        &self,
        current_area: ObjectId,
        connected_areas: &[ObjectId],
        target_area: ObjectId,
        db: &Database,
    ) -> Result<Vec<ObjectId>, String> {
        let mut prioritized = connected_areas.to_vec();

        let current_area_data = Area::get_one_by_id(db, current_area.to_hex().as_str()).await.ok_or("Current area not found")?;
        let target_area_data = Area::get_one_by_id(db, target_area.to_hex().as_str()).await
            .ok_or("Target area not found")?;

        // Sort by priority: target area first, then by floor preference
        prioritized.sort_by(|&a, &b| {
            // Highest priority: direct target area
            if a == target_area {
                return std::cmp::Ordering::Less;
            }
            if b == target_area {
                return std::cmp::Ordering::Greater;
            }

            // Second priority: same floor as target
            let area_a = self.areas.get(&a);
            let area_b = self.areas.get(&b);

            if let (Some(area_a), Some(area_b)) = (area_a, area_b) {
                let target_floor = target_area_data.get_floor();
                let floor_a = area_a.get_floor();
                let floor_b = area_b.get_floor();

                // Prefer areas on the same floor as target
                match (self.floors_match(floor_a, target_floor), self.floors_match(floor_b, target_floor)) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => {}
                }

                // Third priority: prefer going towards target floor
                if let (Some(current_floor), Some(target_floor)) = (current_area_data.get_floor(), target_floor) {
                    let current_level = self.get_floor_level(current_floor);
                    let target_level = self.get_floor_level(target_floor);
                    let level_a = area_a.get_floor().map(|f| self.get_floor_level(f)).unwrap_or(0);
                    let level_b = area_b.get_floor().map(|f| self.get_floor_level(f)).unwrap_or(0);

                    let diff_a = (level_a - target_level).abs();
                    let diff_b = (level_b - target_level).abs();

                    return diff_a.cmp(&diff_b);
                }
            }

            std::cmp::Ordering::Equal
        });

        Ok(prioritized)
    }

    /// Check if two floors match
    fn floors_match(&self, floor1: Option<&Floor>, floor2: Option<&Floor>) -> bool {
        match (floor1, floor2) {
            (Some(f1), Some(f2)) => f1 == f2,
            (None, None) => true,
            _ => false,
        }
    }

    /// Get numeric level for floor comparison
    fn get_floor_level(&self, floor: &Floor) -> i32 {
        match floor.r#type {
            FloorType::Basement => -(floor.name as i32),
            FloorType::Level => {
                // European style: Ground=0, First=1, etc.
                if floor.name == 0 { 0 } else { floor.name as i32 }
            },
            FloorType::Floor => floor.name as i32,
        }
    }

    /// Build route segments from area path
    fn build_route_segments(&self, path: Vec<ObjectId>) -> Result<Vec<RouteSegment>, String> {
        let mut segments = Vec::new();

        for i in 0..path.len() - 1 {
            let from_area = path[i];
            let to_area = path[i + 1];

            let connection = self.find_optimal_connection(from_area, to_area)?;
            let connection_data = self.connections.get(&connection)
                .ok_or("Connection not found")?;

            let instructions = self.generate_connection_instructions(
                from_area,
                to_area,
                connection,
                &connection_data.r#type,
            )?;

            let segment = RouteSegment {
                from_area,
                to_area,
                connection,
                connection_type: connection_data.r#type.clone(),
                instructions,
            };

            segments.push(segment);
        }

        Ok(segments)
    }

    /// Find optimal connection between two areas based on transportation preferences
    fn find_optimal_connection(&self, from_area: ObjectId, to_area: ObjectId) -> Result<ObjectId, String> {
        // Get all connections from the source area
        let from_connections = self.area_connections.get(&from_area)
            .ok_or("No connections found from source area")?;

        // Find connections that connect to the target area
        let mut valid_connections = Vec::new();

        for &connection_id in from_connections {
            if let Some(connection) = self.connections.get(&connection_id) {
                let connected_areas = connection.get_connected_areas();
                let connects_areas = connected_areas.iter()
                    .any(|(area_id, _, _)| *area_id == from_area) &&
                    connected_areas.iter()
                    .any(|(area_id, _, _)| *area_id == to_area);

                if connects_areas {
                    valid_connections.push(connection_id);
                }
            }
        }

        if valid_connections.is_empty() {
            return Err("No valid connection found between areas".to_string());
        }

        // Select optimal connection based on transportation preferences
        self.select_optimal_transportation(from_area, to_area, valid_connections)
    }

    /// Select optimal transportation based on the rules in the documentation
    fn select_optimal_transportation(
        &self,
        from_area: ObjectId,
        to_area: ObjectId,
        connections: Vec<ObjectId>,
    ) -> Result<ObjectId, String> {
        let from_area_data = self.areas.get(&from_area).ok_or("From area not found")?;
        let to_area_data = self.areas.get(&to_area).ok_or("To area not found")?;

        let from_floor = from_area_data.get_floor();
        let to_floor = to_area_data.get_floor();

        // Calculate floor difference
        let floor_diff = match (from_floor, to_floor) {
            (Some(from), Some(to)) => {
                let from_level = self.get_floor_level(from);
                let to_level = self.get_floor_level(to);
                (from_level - to_level).abs()
            },
            _ => 0,
        };

        // Check if either floor is a basement parking lot
        let has_basement_parking = self.is_basement_parking(from_floor) || self.is_basement_parking(to_floor);

        // Sort connections by preference
        let mut prioritized_connections = connections;
        prioritized_connections.sort_by(|&a, &b| {
            let conn_a = self.connections.get(&a);
            let conn_b = self.connections.get(&b);

            if let (Some(conn_a), Some(conn_b)) = (conn_a, conn_b) {
                let type_a = &conn_a.r#type;
                let type_b = &conn_b.r#type;

                // Rule 1: If basement parking, prefer elevator
                if has_basement_parking {
                    match (type_a, type_b) {
                        (ConnectionType::Elevator, ConnectionType::Escalator) => return std::cmp::Ordering::Less,
                        (ConnectionType::Escalator, ConnectionType::Elevator) => return std::cmp::Ordering::Greater,
                        _ => {}
                    }
                }

                // Rule 2: If floor diff > 3, prefer elevator
                if floor_diff > 3 {
                    match (type_a, type_b) {
                        (ConnectionType::Elevator, ConnectionType::Escalator) => return std::cmp::Ordering::Less,
                        (ConnectionType::Escalator, ConnectionType::Elevator) => return std::cmp::Ordering::Greater,
                        _ => {}
                    }
                }

                // Rule 3: If floor diff <= 3, prefer escalator
                if floor_diff <= 3 && floor_diff > 0 {
                    match (type_a, type_b) {
                        (ConnectionType::Escalator, ConnectionType::Elevator) => return std::cmp::Ordering::Less,
                        (ConnectionType::Elevator, ConnectionType::Escalator) => return std::cmp::Ordering::Greater,
                        _ => {}
                    }
                }

                // Default preference order: Escalator > Elevator > Stairs > Gate > Rail > Shuttle
                let priority_a = self.get_connection_priority(type_a);
                let priority_b = self.get_connection_priority(type_b);
                priority_a.cmp(&priority_b)
            } else {
                std::cmp::Ordering::Equal
            }
        });

        prioritized_connections.into_iter().next()
            .ok_or("No connections available".to_string())
    }

    /// Check if a floor is a basement parking lot
    fn is_basement_parking(&self, floor: Option<&Floor>) -> bool {
        match floor {
            Some(floor) => matches!(floor.r#type, FloorType::Basement),
            None => false,
        }
    }

    /// Get priority value for connection types (lower = higher priority)
    fn get_connection_priority(&self, connection_type: &ConnectionType) -> u8 {
        match connection_type {
            ConnectionType::Escalator => 1,
            ConnectionType::Elevator => 2,
            ConnectionType::Stairs => 3,
            ConnectionType::Gate => 4,
            ConnectionType::Rail => 5,
            ConnectionType::Shuttle => 6,
        }
    }

    /// Generate instructions for using a connection
    fn generate_connection_instructions(
        &self,
        from_area: ObjectId,
        to_area: ObjectId,
        connection_id: ObjectId,
        connection_type: &ConnectionType,
    ) -> Result<Vec<RouteInstruction>, String> {
        let mut instructions = Vec::new();

        // Enter connection instruction
        instructions.push(RouteInstruction::EnterConnection(connection_id));

        // Add specific instructions based on connection type
        match connection_type {
            ConnectionType::Elevator => {
                // For elevators, we might want to add specific floor selection instructions
                // This would depend on the specific implementation requirements
            },
            ConnectionType::Escalator => {
                // For escalators, just go straight typically
                instructions.push(RouteInstruction::GoStraight);
            },
            ConnectionType::Stairs => {
                // For stairs, go straight
                instructions.push(RouteInstruction::GoStraight);
            },
            ConnectionType::Gate => {
                // For gates, typically just pass through
                instructions.push(RouteInstruction::GoStraight);
            },
            ConnectionType::Rail | ConnectionType::Shuttle => {
                // For transportation, wait and board
                instructions.push(RouteInstruction::GoStraight);
            },
        }

        // Exit connection instruction
        instructions.push(RouteInstruction::ExitConnection(connection_id));

        Ok(instructions)
    }

    /// Get all areas in the router
    pub fn get_areas(&self) -> &HashMap<ObjectId, Area> {
        &self.areas
    }

    /// Get all connections in the router
    pub fn get_connections(&self) -> &HashMap<ObjectId, Connection> {
        &self.connections
    }

    /// Check if two areas are directly connected
    pub fn are_areas_connected(&self, area1: ObjectId, area2: ObjectId) -> bool {
        if let Some(connected_areas) = self.connection_graph.get(&area1) {
            connected_areas.contains(&area2)
        } else {
            false
        }
    }

    /// Get all areas reachable from a given area
    pub fn get_reachable_areas(&self, from_area: ObjectId) -> Vec<ObjectId> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(from_area);
        reachable.insert(from_area);

        while let Some(current) = queue.pop_front() {
            if let Some(connected) = self.connection_graph.get(&current) {
                for &next_area in connected {
                    if !reachable.contains(&next_area) {
                        reachable.insert(next_area);
                        queue.push_back(next_area);
                    }
                }
            }
        }

        reachable.into_iter().collect()
    }
}
