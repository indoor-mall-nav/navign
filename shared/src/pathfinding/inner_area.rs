//! A* pathfinding within a polygon area

use super::polygon::{BoundedBlock, Polygon};
use alloc::collections::{BTreeMap, BinaryHeap};
use alloc::vec::Vec;
use core::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Error types for inner-area pathfinding
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InnerPathError {
    StartOutsidePolygon,
    EndOutsidePolygon,
    NoPathFound,
    InvalidPolygon,
}

impl core::fmt::Display for InnerPathError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InnerPathError::StartOutsidePolygon => write!(f, "Start point is outside the polygon"),
            InnerPathError::EndOutsidePolygon => write!(f, "End point is outside the polygon"),
            InnerPathError::NoPathFound => write!(f, "No path found between points"),
            InnerPathError::InvalidPolygon => write!(f, "Invalid polygon"),
        }
    }
}

/// Node for A* priority queue
#[derive(Debug, Clone)]
struct PathNode {
    index: usize,
    f_score: u64,
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_score == other.f_score
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
        // Reverse ordering for min-heap
        other.f_score.cmp(&self.f_score)
    }
}

/// Calculate Manhattan distance between two points
fn manhattan_distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

/// Reconstruct path from came_from map
fn reconstruct_path(came_from: &BTreeMap<usize, usize>, mut current: usize) -> Vec<usize> {
    let mut path = Vec::new();
    path.push(current);

    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }

    path.reverse();
    path
}

/// Get neighboring block indices in a grid
fn get_neighbors(index: usize, cols: usize, rows: usize, blocks: &[BoundedBlock]) -> Vec<usize> {
    let row = index / cols;
    let col = index % cols;
    let mut neighbors = Vec::new();

    // Check all 8 directions (including diagonals)
    let directions = [
        (-1, 0),  // left
        (1, 0),   // right
        (0, -1),  // up
        (0, 1),   // down
        (-1, -1), // up-left
        (-1, 1),  // down-left
        (1, -1),  // up-right
        (1, 1),   // down-right
    ];

    for (dx, dy) in directions.iter() {
        let new_row = row as i32 + dy;
        let new_col = col as i32 + dx;

        if new_row >= 0 && new_row < rows as i32 && new_col >= 0 && new_col < cols as i32 {
            let new_index = new_row as usize * cols + new_col as usize;
            if new_index < blocks.len() && blocks[new_index].is_bounded {
                neighbors.push(new_index);
            }
        }
    }

    neighbors
}

/// Find a path within a polygon area using A* algorithm
///
/// # Arguments
/// * `polygon` - The polygon area to navigate within
/// * `start` - Starting coordinates (x, y)
/// * `end` - Ending coordinates (x, y)
/// * `block_size` - Size of each block in the grid (default: 1.0)
///
/// # Returns
/// * `Ok(Vec<(f64, f64)>)` - List of waypoints (block centers) from start to end
/// * `Err(InnerPathError)` - Error if pathfinding fails
pub fn find_path_in_area(
    polygon: &Polygon,
    start: (f64, f64),
    end: (f64, f64),
    block_size: f64,
) -> Result<Vec<(f64, f64)>, InnerPathError> {
    if polygon.vertices.len() < 3 {
        return Err(InnerPathError::InvalidPolygon);
    }

    // Check if start and end are inside the polygon
    if !polygon.contains(start.0, start.1) {
        return Err(InnerPathError::StartOutsidePolygon);
    }

    if !polygon.contains(end.0, end.1) {
        return Err(InnerPathError::EndOutsidePolygon);
    }

    // Convert polygon to grid of blocks
    let blocks = polygon.to_bounded_blocks(block_size);
    let (cols, rows) = polygon.grid_dimensions(block_size);

    // Find start and end block indices
    let start_index = blocks
        .iter()
        .position(|b| b.is_bounded && b.contains(start.0, start.1))
        .ok_or(InnerPathError::StartOutsidePolygon)?;

    let end_index = blocks
        .iter()
        .position(|b| b.is_bounded && b.contains(end.0, end.1))
        .ok_or(InnerPathError::EndOutsidePolygon)?;

    // If start and end are in the same block, return direct path
    if start_index == end_index {
        return Ok(alloc::vec![start, end]);
    }

    // A* algorithm
    let mut open_set = BinaryHeap::new();
    let mut came_from = BTreeMap::new();
    let mut g_score = BTreeMap::new();

    let h_score = manhattan_distance(blocks[start_index].center(), blocks[end_index].center());

    g_score.insert(start_index, 0);
    open_set.push(PathNode {
        index: start_index,
        f_score: (h_score * 100.0) as u64,
    });

    while let Some(current_node) = open_set.pop() {
        let current = current_node.index;

        // Check if we reached the end
        if current == end_index {
            let path_indices = reconstruct_path(&came_from, current);
            let waypoints: Vec<(f64, f64)> =
                path_indices.iter().map(|&i| blocks[i].center()).collect();
            return Ok(waypoints);
        }

        // Explore neighbors
        for neighbor in get_neighbors(current, cols, rows, &blocks) {
            let current_center = blocks[current].center();
            let neighbor_center = blocks[neighbor].center();
            let edge_cost = manhattan_distance(current_center, neighbor_center);

            let tentative_g_score =
                g_score.get(&current).unwrap_or(&u64::MAX) + (edge_cost * 100.0) as u64;

            if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&u64::MAX) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g_score);

                let h = manhattan_distance(neighbor_center, blocks[end_index].center());
                let f_score = tentative_g_score + (h * 100.0) as u64;

                open_set.push(PathNode {
                    index: neighbor,
                    f_score,
                });
            }
        }
    }

    Err(InnerPathError::NoPathFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_path() {
        let polygon = Polygon::from_coords(vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]);

        let result = find_path_in_area(&polygon, (1.0, 1.0), (9.0, 9.0), 1.0);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.len() > 0);
        assert!(path.len() < 20); // Should be reasonably short
    }

    #[test]
    fn test_same_block() {
        let polygon = Polygon::from_coords(vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]);

        let result = find_path_in_area(&polygon, (1.0, 1.0), (1.2, 1.2), 1.0);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn test_start_outside() {
        let polygon = Polygon::from_coords(vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]);

        let result = find_path_in_area(&polygon, (-1.0, -1.0), (5.0, 5.0), 1.0);
        assert_eq!(result, Err(InnerPathError::StartOutsidePolygon));
    }

    #[test]
    fn test_end_outside() {
        let polygon = Polygon::from_coords(vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]);

        let result = find_path_in_area(&polygon, (5.0, 5.0), (15.0, 15.0), 1.0);
        assert_eq!(result, Err(InnerPathError::EndOutsidePolygon));
    }

    #[test]
    fn test_l_shaped_polygon() {
        // L-shaped polygon
        let polygon = Polygon::from_coords(vec![
            (0.0, 0.0),
            (0.0, 10.0),
            (5.0, 10.0),
            (5.0, 5.0),
            (10.0, 5.0),
            (10.0, 0.0),
            (0.0, 0.0),
        ]);

        let result = find_path_in_area(&polygon, (1.0, 1.0), (9.0, 1.0), 1.0);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.len() > 0);
    }
}
