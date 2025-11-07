//! Dijkstra pathfinding between connected areas

use super::inner_area::find_path_in_area;
use super::polygon::Polygon;
use crate::ConnectionType;
use alloc::collections::{BTreeMap, BinaryHeap};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Error types for inter-area pathfinding
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InterPathError {
    InvalidStartArea,
    InvalidEndArea,
    NoPathFound,
    InvalidConnection,
    InnerPathError(String),
}

impl core::fmt::Display for InterPathError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InterPathError::InvalidStartArea => write!(f, "Invalid start area"),
            InterPathError::InvalidEndArea => write!(f, "Invalid end area"),
            InterPathError::NoPathFound => write!(f, "No path found between areas"),
            InterPathError::InvalidConnection => write!(f, "Invalid connection"),
            InterPathError::InnerPathError(msg) => write!(f, "Inner path error: {}", msg),
        }
    }
}

/// Navigation instruction
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum RouteInstruction {
    /// Move to a coordinate (x, y)
    Move(f64, f64),
    /// Take a connection (connection_id, target_area_id, connection_type)
    Transport(String, String, ConnectionType),
}

/// Connectivity limits for pathfinding
#[derive(Debug, Clone, Copy)]
pub struct ConnectivityLimits {
    pub elevator: bool,
    pub stairs: bool,
    pub escalator: bool,
}

impl Default for ConnectivityLimits {
    fn default() -> Self {
        Self {
            elevator: true,
            stairs: true,
            escalator: true,
        }
    }
}

/// Area data for pathfinding
#[derive(Debug, Clone)]
pub struct AreaData {
    pub id: String,
    pub polygon: Polygon,
    pub connections: Vec<ConnectionData>,
}

/// Connection data
#[derive(Debug, Clone)]
pub struct ConnectionData {
    pub id: String,
    pub conn_type: ConnectionType,
    /// List of (area_id, x, y, enabled) for connected areas
    pub connected_areas: Vec<(String, f64, f64, bool)>,
}

/// Node for Dijkstra priority queue
#[derive(Debug, Clone)]
struct PathNode {
    area_id: String,
    distance: u64,
    position: (f64, f64),
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.distance.cmp(&self.distance)
    }
}

/// Calculate Manhattan distance
fn manhattan_distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

/// Find path between areas using Dijkstra's algorithm
///
/// # Arguments
/// * `areas` - List of all areas with their connections
/// * `start_area_id` - Starting area ID
/// * `start_pos` - Starting position (x, y) within start area
/// * `end_area_id` - Ending area ID
/// * `end_pos` - Ending position (x, y) within end area
/// * `limits` - Connectivity limits (which connection types are allowed)
/// * `block_size` - Block size for inner-area pathfinding (default: 1.0)
///
/// # Returns
/// * `Ok(Vec<RouteInstruction>)` - List of navigation instructions
/// * `Err(InterPathError)` - Error if pathfinding fails
pub fn find_path_between_areas(
    areas: &[AreaData],
    start_area_id: &str,
    start_pos: (f64, f64),
    end_area_id: &str,
    end_pos: (f64, f64),
    limits: ConnectivityLimits,
    block_size: f64,
) -> Result<Vec<RouteInstruction>, InterPathError> {
    // Find start and end areas
    let start_area = areas
        .iter()
        .find(|a| a.id == start_area_id)
        .ok_or(InterPathError::InvalidStartArea)?;

    let _end_area = areas
        .iter()
        .find(|a| a.id == end_area_id)
        .ok_or(InterPathError::InvalidEndArea)?;

    // If same area, just do inner-area pathfinding
    if start_area_id == end_area_id {
        let waypoints = find_path_in_area(&start_area.polygon, start_pos, end_pos, block_size)
            .map_err(|e| InterPathError::InnerPathError(alloc::format!("{:?}", e)))?;

        return Ok(waypoints
            .into_iter()
            .map(|(x, y)| RouteInstruction::Move(x, y))
            .collect());
    }

    // Create area map for quick lookup
    let area_map: BTreeMap<&str, &AreaData> = areas.iter().map(|a| (a.id.as_str(), a)).collect();

    // Dijkstra's algorithm
    let mut heap = BinaryHeap::new();
    let mut visited = BTreeMap::new();
    let mut came_from: BTreeMap<String, (String, String)> = BTreeMap::new(); // area_id -> (prev_area_id, connection_id)
    let mut distance_map = BTreeMap::new();

    heap.push(PathNode {
        area_id: start_area_id.to_string(),
        distance: 0,
        position: start_pos,
    });
    distance_map.insert(start_area_id.to_string(), 0u64);

    while let Some(PathNode {
        area_id: current_area_id,
        distance: current_distance,
        position: current_pos,
    }) = heap.pop()
    {
        if visited.contains_key(current_area_id.as_str()) {
            continue;
        }
        visited.insert(current_area_id.clone(), current_pos);

        // Check if we reached the end
        if current_area_id == end_area_id {
            // Reconstruct path
            return reconstruct_full_path(
                &came_from,
                &area_map,
                start_area_id,
                start_pos,
                end_area_id,
                end_pos,
                block_size,
            );
        }

        // Get current area
        let current_area = match area_map.get(current_area_id.as_str()) {
            Some(a) => a,
            None => continue,
        };

        // Explore connections
        for conn in &current_area.connections {
            // Check if connection type is allowed
            let allowed = match conn.conn_type {
                ConnectionType::Elevator => limits.elevator,
                ConnectionType::Stairs => limits.stairs,
                ConnectionType::Escalator => limits.escalator,
                _ => true,
            };

            if !allowed {
                continue;
            }

            // Find connected areas
            for (neighbor_id, conn_x, conn_y, enabled) in &conn.connected_areas {
                if !enabled || neighbor_id == &current_area_id {
                    continue;
                }

                if visited.contains_key(neighbor_id.as_str()) {
                    continue;
                }

                // Calculate distance to connection point
                let edge_distance = manhattan_distance(current_pos, (*conn_x, *conn_y));
                let tentative_distance = current_distance + (edge_distance * 100.0) as u64;

                if tentative_distance < *distance_map.get(neighbor_id.as_str()).unwrap_or(&u64::MAX)
                {
                    came_from.insert(
                        neighbor_id.clone(),
                        (current_area_id.clone(), conn.id.clone()),
                    );
                    distance_map.insert(neighbor_id.clone(), tentative_distance);

                    heap.push(PathNode {
                        area_id: neighbor_id.clone(),
                        distance: tentative_distance,
                        position: (*conn_x, *conn_y),
                    });
                }
            }
        }
    }

    Err(InterPathError::NoPathFound)
}

/// Reconstruct the full path with instructions
fn reconstruct_full_path(
    came_from: &BTreeMap<String, (String, String)>,
    area_map: &BTreeMap<&str, &AreaData>,
    _start_area_id: &str,
    start_pos: (f64, f64),
    end_area_id: &str,
    end_pos: (f64, f64),
    block_size: f64,
) -> Result<Vec<RouteInstruction>, InterPathError> {
    // Build path from end to start
    let mut path = Vec::new();
    path.push(end_area_id.to_string());

    let mut current = end_area_id.to_string();
    while let Some((prev, _)) = came_from.get(&current) {
        path.push(prev.clone());
        current = prev.clone();
    }
    path.reverse();

    // Generate instructions
    let mut instructions = Vec::new();
    let mut current_pos = start_pos;

    for i in 0..path.len() {
        let current_area_id = &path[i];
        let current_area = area_map
            .get(current_area_id.as_str())
            .ok_or(InterPathError::InvalidStartArea)?;

        if i == path.len() - 1 {
            // Final area - navigate to end position
            let waypoints =
                find_path_in_area(&current_area.polygon, current_pos, end_pos, block_size)
                    .map_err(|e| InterPathError::InnerPathError(alloc::format!("{:?}", e)))?;

            instructions.extend(
                waypoints
                    .into_iter()
                    .map(|(x, y)| RouteInstruction::Move(x, y)),
            );
        } else {
            // Navigate to connection
            let next_area_id = &path[i + 1];
            let (_, conn_id) = came_from
                .get(next_area_id)
                .ok_or(InterPathError::InvalidConnection)?;

            // Find the connection
            let conn = current_area
                .connections
                .iter()
                .find(|c| &c.id == conn_id)
                .ok_or(InterPathError::InvalidConnection)?;

            // Find connection point in current area
            let conn_point = conn
                .connected_areas
                .iter()
                .find(|(aid, _, _, _)| aid == current_area_id)
                .map(|(_, x, y, _)| (*x, *y))
                .ok_or(InterPathError::InvalidConnection)?;

            // Navigate to connection point
            let waypoints =
                find_path_in_area(&current_area.polygon, current_pos, conn_point, block_size)
                    .map_err(|e| InterPathError::InnerPathError(alloc::format!("{:?}", e)))?;

            instructions.extend(
                waypoints
                    .into_iter()
                    .map(|(x, y)| RouteInstruction::Move(x, y)),
            );

            // Add transport instruction
            instructions.push(RouteInstruction::Transport(
                conn_id.clone(),
                next_area_id.clone(),
                conn.conn_type,
            ));

            // Update current position to entry point in next area
            current_pos = conn
                .connected_areas
                .iter()
                .find(|(aid, _, _, _)| aid == next_area_id)
                .map(|(_, x, y, _)| (*x, *y))
                .ok_or(InterPathError::InvalidConnection)?;
        }
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_area_routing() {
        let area = AreaData {
            id: "area1".to_string(),
            polygon: Polygon::from_coords(vec![
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 10.0),
                (0.0, 10.0),
                (0.0, 0.0),
            ]),
            connections: Vec::new(),
        };

        let result = find_path_between_areas(
            &[area],
            "area1",
            (1.0, 1.0),
            "area1",
            (9.0, 9.0),
            ConnectivityLimits::default(),
            1.0,
        );

        assert!(result.is_ok());
        let instructions = result.unwrap();
        assert!(instructions.len() > 0);
        assert!(matches!(instructions[0], RouteInstruction::Move(_, _)));
    }

    #[test]
    fn test_two_connected_areas() {
        let area1 = AreaData {
            id: "area1".to_string(),
            polygon: Polygon::from_coords(vec![
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 10.0),
                (0.0, 10.0),
                (0.0, 0.0),
            ]),
            connections: vec![ConnectionData {
                id: "conn1".to_string(),
                conn_type: ConnectionType::Escalator,
                connected_areas: vec![
                    ("area1".to_string(), 9.0, 5.0, true),
                    ("area2".to_string(), 1.0, 5.0, true),
                ],
            }],
        };

        let area2 = AreaData {
            id: "area2".to_string(),
            polygon: Polygon::from_coords(vec![
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 10.0),
                (0.0, 10.0),
                (0.0, 0.0),
            ]),
            connections: vec![ConnectionData {
                id: "conn1".to_string(),
                conn_type: ConnectionType::Escalator,
                connected_areas: vec![
                    ("area1".to_string(), 9.0, 5.0, true),
                    ("area2".to_string(), 1.0, 5.0, true),
                ],
            }],
        };

        let result = find_path_between_areas(
            &[area1, area2],
            "area1",
            (1.0, 1.0),
            "area2",
            (9.0, 9.0),
            ConnectivityLimits::default(),
            1.0,
        );

        assert!(result.is_ok());
        let instructions = result.unwrap();
        assert!(instructions.len() > 1);

        // Should have at least one Transport instruction
        let has_transport = instructions
            .iter()
            .any(|i| matches!(i, RouteInstruction::Transport(_, _, _)));
        assert!(has_transport);
    }
}
