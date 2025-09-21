//! # Inner Area Routing.
//! This is a module for inner area routing.
//! It provides functionalities to find the best route within an area,
//! considering various factors such as distance, obstacles, and user preferences.
//!
//! Normally, it's straightforward to find a route _inner_ an area.
//! In the database, an area is represented as a polygon that indicating its boundary.
//!
//! Hence, the algorithm to find a route _inner_ an area is as follows:
//! 1. Translate the polygon to a graph (from vertices to nodes, from edges to connections).
//! 2. Use "weighted" Dijkstra's algorithm to find the best route, that is, prefer the route with the main road, avoid obstacles, etc.
//! 3. Translate the route back to a series of instructions and beacons that can drop by.
use crate::kernel::route::instructions::{RouteInstruction, TurnTo};
use crate::schema::area::Area;
use crate::schema::beacon::Beacon;
use bson::oid::ObjectId;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

/// Represents a node in the routing graph
#[derive(Debug, Clone, PartialEq)]
pub struct GraphNode {
    pub id: usize,
    pub position: (f64, f64),
    pub is_beacon: bool,
    pub beacon_id: Option<ObjectId>,
    pub node_type: NodeType,
}

/// Types of nodes in the routing graph
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Vertex,        // Polygon vertex
    Beacon,        // Beacon location
    Intersection,  // Path intersection
    Entrance,      // Area entrance/exit
}

/// Represents an edge in the routing graph with weight
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: usize,
    pub to: usize,
    pub weight: f64,
    pub edge_type: EdgeType,
}

/// Types of edges in the routing graph
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    MainPath,     // Main walkway (lower weight)
    SidePath,     // Side path (higher weight)
    Direct,       // Direct connection
    Obstacle,     // Path with obstacles (highest weight)
}

/// State for Dijkstra's algorithm
#[derive(Debug, Clone, PartialEq)]
struct DijkstraState {
    cost: f64,
    node: usize,
}

impl Eq for DijkstraState {}

impl Ord for DijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for DijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Inner area router
pub struct InnerAreaRouter {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    adjacency_list: HashMap<usize, Vec<usize>>,
}

impl InnerAreaRouter {
    /// Create a new inner area router
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            adjacency_list: HashMap::new(),
        }
    }

    /// Build graph from area polygon and beacons
    pub fn build_graph(&mut self, area: &Area, beacons: &[Beacon]) -> Result<(), String> {
        self.nodes.clear();
        self.edges.clear();
        self.adjacency_list.clear();

        // Step 1: Convert polygon vertices to nodes
        self.add_polygon_vertices(area)?;

        // Step 2: Add beacon nodes
        self.add_beacon_nodes(beacons)?;

        // Step 3: Create edges between nodes
        self.create_edges(area)?;

        // Step 4: Build adjacency list
        self.build_adjacency_list();

        Ok(())
    }

    /// Add polygon vertices as graph nodes
    fn add_polygon_vertices(&mut self, area: &Area) -> Result<(), String> {
        let polygon = area.get_polygon();

        for (i, &(x, y)) in polygon.iter().enumerate() {
            let node = GraphNode {
                id: i,
                position: (x, y),
                is_beacon: false,
                beacon_id: None,
                node_type: if i == 0 { NodeType::Entrance } else { NodeType::Vertex },
            };
            self.nodes.push(node);
        }

        Ok(())
    }

    /// Add beacon locations as graph nodes
    fn add_beacon_nodes(&mut self, beacons: &[Beacon]) -> Result<(), String> {
        let start_id = self.nodes.len();

        for (i, beacon) in beacons.iter().enumerate() {
            let position = beacon.get_location();
            let node = GraphNode {
                id: start_id + i,
                position,
                is_beacon: true,
                beacon_id: Some(beacon.get_object_id()),
                node_type: NodeType::Beacon,
            };
            self.nodes.push(node);
        }

        Ok(())
    }

    /// Create weighted edges between nodes
    fn create_edges(&mut self, area: &Area) -> Result<(), String> {
        let polygon = area.get_polygon();
        let polygon_len = polygon.len();

        // Create edges along polygon perimeter
        for i in 0..polygon_len {
            let next = (i + 1) % polygon_len;
            self.add_edge(i, next, EdgeType::MainPath);
        }

        // Create edges from beacons to nearest polygon vertices
        self.nodes.clone().iter().filter(|n| n.is_beacon).for_each(|beacon_node| {
            let nearest_vertices = self.find_nearest_vertices(beacon_node.position, 2);
            for vertex_id in nearest_vertices {
                self.add_edge(beacon_node.id, vertex_id, EdgeType::SidePath);
                self.add_edge(vertex_id, beacon_node.id, EdgeType::SidePath);
            }
        });

        // Create direct connections between nearby nodes
        self.create_direct_connections()?;

        Ok(())
    }

    /// Add an edge with calculated weight
    fn add_edge(&mut self, from: usize, to: usize, edge_type: EdgeType) {
        if let (Some(from_node), Some(to_node)) = (self.nodes.get(from), self.nodes.get(to)) {
            let distance = self.calculate_distance(from_node.position, to_node.position);
            let weight = self.calculate_edge_weight(distance, &edge_type);

            let edge = GraphEdge {
                from,
                to,
                weight,
                edge_type,
            };
            self.edges.push(edge);
        }
    }

    /// Calculate edge weight based on distance and type
    fn calculate_edge_weight(&self, distance: f64, edge_type: &EdgeType) -> f64 {
        let base_weight = distance;
        match edge_type {
            EdgeType::MainPath => base_weight,
            EdgeType::SidePath => base_weight * 1.2,
            EdgeType::Direct => base_weight * 1.1,
            EdgeType::Obstacle => base_weight * 2.0,
        }
    }

    /// Find nearest vertices to a given position
    fn find_nearest_vertices(&self, position: (f64, f64), count: usize) -> Vec<usize> {
        let mut distances: Vec<(usize, f64)> = self.nodes
            .iter()
            .filter(|node| !node.is_beacon)
            .map(|node| {
                let dist = self.calculate_distance(position, node.position);
                (node.id, dist)
            })
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        distances.into_iter().take(count).map(|(id, _)| id).collect()
    }

    /// Create direct connections between nearby nodes
    fn create_direct_connections(&mut self) -> Result<(), String> {
        let threshold = 50.0; // Maximum distance for direct connection

        for i in 0..self.nodes.len() {
            for j in (i + 1)..self.nodes.len() {
                let dist = self.calculate_distance(self.nodes[i].position, self.nodes[j].position);
                if dist <= threshold && !self.has_edge(i, j) {
                    self.add_edge(i, j, EdgeType::Direct);
                    self.add_edge(j, i, EdgeType::Direct);
                }
            }
        }

        Ok(())
    }

    /// Check if edge exists between two nodes
    fn has_edge(&self, from: usize, to: usize) -> bool {
        self.edges.iter().any(|edge| edge.from == from && edge.to == to)
    }

    /// Build adjacency list for efficient graph traversal
    fn build_adjacency_list(&mut self) {
        for edge in &self.edges {
            self.adjacency_list
                .entry(edge.from)
                .or_insert_with(Vec::new)
                .push(edge.to);
        }
    }

    /// Calculate Euclidean distance between two points
    fn calculate_distance(&self, p1: (f64, f64), p2: (f64, f64)) -> f64 {
        let dx = p1.0 - p2.0;
        let dy = p1.1 - p2.1;
        (dx * dx + dy * dy).sqrt()
    }

    /// Find the best route using weighted Dijkstra's algorithm
    pub fn find_route(&self, start_beacon_id: ObjectId, end_beacon_id: ObjectId) -> Result<Vec<RouteInstruction>, String> {
        let start_node = self.find_beacon_node(start_beacon_id)
            .ok_or("Start beacon not found")?;
        let end_node = self.find_beacon_node(end_beacon_id)
            .ok_or("End beacon not found")?;

        let path = self.dijkstra(start_node.id, end_node.id)?;
        self.generate_instructions(path)
    }

    /// Find beacon node by ObjectId
    fn find_beacon_node(&self, beacon_id: ObjectId) -> Option<&GraphNode> {
        self.nodes.iter().find(|node| {
            node.is_beacon && node.beacon_id == Some(beacon_id)
        })
    }

    /// Weighted Dijkstra's algorithm implementation
    fn dijkstra(&self, start: usize, end: usize) -> Result<Vec<usize>, String> {
        let mut dist = vec![f64::INFINITY; self.nodes.len()];
        let mut prev = vec![None; self.nodes.len()];
        let mut heap = BinaryHeap::new();

        dist[start] = 0.0;
        heap.push(DijkstraState { cost: 0.0, node: start });

        while let Some(DijkstraState { cost, node }) = heap.pop() {
            if node == end {
                break;
            }

            if cost > dist[node] {
                continue;
            }

            if let Some(neighbors) = self.adjacency_list.get(&node) {
                for &neighbor in neighbors {
                    if let Some(edge) = self.find_edge(node, neighbor) {
                        let new_cost = dist[node] + edge.weight;
                        if new_cost < dist[neighbor] {
                            dist[neighbor] = new_cost;
                            prev[neighbor] = Some(node);
                            heap.push(DijkstraState { cost: new_cost, node: neighbor });
                        }
                    }
                }
            }
        }

        if dist[end] == f64::INFINITY {
            return Err("No path found".to_string());
        }

        // Reconstruct path
        let mut path = Vec::new();
        let mut current = end;
        while let Some(previous) = prev[current] {
            path.push(current);
            current = previous;
        }
        path.push(start);
        path.reverse();

        Ok(path)
    }

    /// Find edge between two nodes
    fn find_edge(&self, from: usize, to: usize) -> Option<&GraphEdge> {
        self.edges.iter().find(|edge| edge.from == from && edge.to == to)
    }

    /// Generate route instructions from path
    fn generate_instructions(&self, path: Vec<usize>) -> Result<Vec<RouteInstruction>, String> {
        let mut instructions = Vec::new();

        for i in 0..path.len() {
            let current_node = &self.nodes[path[i]];

            if i == 0 {
                // Start instruction
                if let Some(beacon_id) = current_node.beacon_id {
                    // Starting from a beacon, no special instruction needed
                }
            } else if i == path.len() - 1 {
                // End instruction
                instructions.push(RouteInstruction::ArriveDestination);
            } else {
                // Middle instructions
                let prev_node = &self.nodes[path[i - 1]];
                let next_node = &self.nodes[path[i + 1]];

                // Calculate turn direction
                if let Some(turn) = self.calculate_turn_direction(
                    prev_node.position,
                    current_node.position,
                    next_node.position,
                ) {
                    match turn {
                        TurnTo::Left | TurnTo::Right | TurnTo::Back => {
                            instructions.push(RouteInstruction::Turn(turn));
                        }
                    }
                } else {
                    instructions.push(RouteInstruction::GoStraight);
                }

                // Add beacon instructions if current node is a beacon
                if current_node.is_beacon {
                    if let Some(beacon_id) = current_node.beacon_id {
                        // This would be a waypoint beacon
                    }
                }
            }
        }

        if instructions.is_empty() {
            instructions.push(RouteInstruction::GoStraight);
            instructions.push(RouteInstruction::ArriveDestination);
        }

        Ok(instructions)
    }

    /// Calculate turn direction based on three consecutive points
    fn calculate_turn_direction(
        &self,
        prev: (f64, f64),
        current: (f64, f64),
        next: (f64, f64),
    ) -> Option<TurnTo> {
        let vec1 = (current.0 - prev.0, current.1 - prev.1);
        let vec2 = (next.0 - current.0, next.1 - current.1);

        // Calculate cross product to determine turn direction
        let cross_product = vec1.0 * vec2.1 - vec1.1 * vec2.0;
        let dot_product = vec1.0 * vec2.0 + vec1.1 * vec2.1;

        let angle_threshold = 0.5; // Threshold for considering it a turn

        if cross_product.abs() < angle_threshold {
            if dot_product < -0.5 {
                Some(TurnTo::Back)
            } else {
                None // Go straight
            }
        } else if cross_product > 0.0 {
            Some(TurnTo::Left)
        } else {
            Some(TurnTo::Right)
        }
    }
}
