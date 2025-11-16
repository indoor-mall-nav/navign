//! Polygon and block structures for pathfinding
//!
//! This module provides two approaches for pathfinding within polygon areas:
//!
//! ## 1. Grid-Based Pathfinding (Manhattan-style)
//!
//! The grid-based approach divides a polygon into rectangular blocks and uses A* pathfinding.
//! This works well for Manhattan-style layouts (buildings with rectangular rooms and hallways).
//!
//! ```ignore
//! use navign_shared::pathfinding::Polygon;
//!
//! let polygon = Polygon::from_coords(vec![
//!     (0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)
//! ]);
//!
//! let blocks = polygon.to_bounded_blocks(1.0); // 1.0 meter blocks
//! // Use blocks for pathfinding...
//! ```
//!
//! ## 2. Triangulation-Based Pathfinding (Non-Manhattan)
//!
//! For irregular polygon shapes, triangulation provides more natural paths that follow
//! the polygon boundaries. This uses the Earcut algorithm for fast polygon triangulation
//! and A* pathfinding on the resulting triangle mesh.
//!
//! **Requires:** `geo` feature flag
//!
//! **Advantages:**
//! - More natural paths for irregular polygons
//! - Better handling of non-rectangular areas
//! - Reduced memory usage for large irregular areas
//! - Paths follow polygon shape more precisely
//!
//! **Example:**
//!
//! ```ignore
//! use navign_shared::pathfinding::{Polygon, TriangulationMesh};
//!
//! // Create an L-shaped polygon
//! let polygon = Polygon::from_coords(vec![
//!     (0.0, 0.0), (5.0, 0.0), (5.0, 5.0),
//!     (10.0, 5.0), (10.0, 10.0), (0.0, 10.0)
//! ]);
//!
//! // Triangulate the polygon
//! let mesh = polygon.to_triangulation_mesh().unwrap();
//!
//! // Find a path through the mesh
//! let path = mesh.find_path((2.0, 2.0), (8.0, 8.0)).unwrap();
//!
//! // path contains waypoints from start to end
//! assert_eq!(path.first(), Some(&(2.0, 2.0)));
//! assert_eq!(path.last(), Some(&(8.0, 8.0)));
//! ```
//!
//! ## When to Use Each Approach
//!
//! - **Grid-based**: Best for Manhattan layouts (malls, offices with rectangular rooms)
//! - **Triangulation**: Best for irregular areas (parks, atriums, curved hallways)
//!
//! Both approaches are available and can be used together in the same application.

use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "geo")]
use geo::{BoundingRect, Contains, Coord, LineString, Point, Polygon as GeoPolygon};

#[cfg(feature = "geo")]
use earcutr;

/// A rectangular block with bounds
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BoundedBlock {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub is_bounded: bool,
}

impl BoundedBlock {
    /// Get the center point of the block
    pub fn center(&self) -> (f64, f64) {
        ((self.x1 + self.x2) / 2.0, (self.y1 + self.y2) / 2.0)
    }

    /// Check if a point is inside the block
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }
}

/// A polygon defined by a list of vertices
#[derive(Debug, Clone)]
pub struct Polygon {
    pub vertices: Vec<(f64, f64)>,
}

impl Polygon {
    /// Create a new polygon from vertices
    pub fn new(vertices: Vec<(f64, f64)>) -> Self {
        Self { vertices }
    }

    /// Convert WKT POLYGON string to Polygon
    /// WKT format: "POLYGON((x1 y1, x2 y2, x3 y3, ...))"
    pub fn from_wkt(wkt: &str) -> Result<Self, &'static str> {
        let wkt = wkt.trim();

        // Check if it starts with POLYGON
        if !wkt.starts_with("POLYGON") {
            return Err("WKT string must start with POLYGON");
        }

        // Find the content between double parentheses
        let start = wkt.find("((").ok_or("Missing opening ((")?;
        let end = wkt.rfind("))").ok_or("Missing closing ))")?;

        if start >= end {
            return Err("Invalid WKT format");
        }

        let coords_str = &wkt[start + 2..end];

        let mut vertices = Vec::new();
        for coord in coords_str.split(',') {
            let parts: Vec<&str> = coord.split_whitespace().collect();
            if parts.len() != 2 {
                return Err("Each coordinate must have exactly 2 values");
            }

            let x = parts[0]
                .parse::<f64>()
                .map_err(|_| "Invalid x coordinate")?;
            let y = parts[1]
                .parse::<f64>()
                .map_err(|_| "Invalid y coordinate")?;
            vertices.push((x, y));
        }

        Ok(Self { vertices })
    }

    /// Convert Vec<(f64, f64)> to Polygon
    pub fn from_coords(coords: Vec<(f64, f64)>) -> Self {
        Self { vertices: coords }
    }

    /// Get bounding box of the polygon
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        #[cfg(feature = "geo")]
        {
            if self.vertices.is_empty() {
                return (0.0, 0.0, 0.0, 0.0);
            }

            let coords: Vec<Coord<f64>> =
                self.vertices.iter().map(|&(x, y)| Coord { x, y }).collect();

            let mut closed_coords = coords;
            if let Some(&first) = self.vertices.first() {
                closed_coords.push(Coord {
                    x: first.0,
                    y: first.1,
                });
            }

            let line_string = LineString::new(closed_coords);
            let geo_polygon = GeoPolygon::new(line_string, vec![]);

            if let Some(rect) = geo_polygon.bounding_rect() {
                return (rect.min().x, rect.min().y, rect.max().x, rect.max().y);
            }

            (0.0, 0.0, 0.0, 0.0)
        }

        #[cfg(not(feature = "geo"))]
        {
            let mut min_x = f64::MAX;
            let mut max_x = f64::MIN;
            let mut min_y = f64::MAX;
            let mut max_y = f64::MIN;

            for (x, y) in &self.vertices {
                min_x = min_x.min(*x);
                max_x = max_x.max(*x);
                min_y = min_y.min(*y);
                max_y = max_y.max(*y);
            }

            (min_x, min_y, max_x, max_y)
        }
    }

    /// Check if a point is inside the polygon
    /// Uses geo crate's optimized Contains trait when available
    pub fn contains(&self, x: f64, y: f64) -> bool {
        if self.vertices.len() < 3 {
            return false;
        }

        #[cfg(feature = "geo")]
        {
            // Convert to geo::Polygon for efficient point-in-polygon check
            let coords: Vec<Coord<f64>> =
                self.vertices.iter().map(|&(x, y)| Coord { x, y }).collect();

            // Create a closed LineString (geo requires first == last point)
            let mut closed_coords = coords;
            if let Some(&first) = self.vertices.first() {
                closed_coords.push(Coord {
                    x: first.0,
                    y: first.1,
                });
            }

            let line_string = LineString::new(closed_coords);
            let geo_polygon = GeoPolygon::new(line_string, vec![]);
            let point = Point::new(x, y);

            geo_polygon.contains(&point)
        }

        #[cfg(not(feature = "geo"))]
        {
            // Fallback to manual ray-casting algorithm
            let n = self.vertices.len();
            let mut inside = false;
            let mut j = n - 1;

            for i in 0..n {
                let (xi, yi) = self.vertices[i];
                let (xj, yj) = self.vertices[j];

                if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
                    inside = !inside;
                }
                j = i;
            }

            inside
        }
    }

    /// Convert polygon to a grid of bounded blocks for pathfinding
    /// block_size: the size of each block in the grid
    pub fn to_bounded_blocks(&self, block_size: f64) -> Vec<BoundedBlock> {
        let (min_x, min_y, max_x, max_y) = self.bounds();
        let mut blocks = Vec::new();

        let cols = ((max_x - min_x) / block_size).ceil() as usize;
        let rows = ((max_y - min_y) / block_size).ceil() as usize;

        for row in 0..rows {
            for col in 0..cols {
                let x1 = min_x + col as f64 * block_size;
                let y1 = min_y + row as f64 * block_size;
                let x2 = x1 + block_size;
                let y2 = y1 + block_size;

                let center_x = (x1 + x2) / 2.0;
                let center_y = (y1 + y2) / 2.0;

                let is_bounded = self.contains(center_x, center_y);

                blocks.push(BoundedBlock {
                    x1,
                    y1,
                    x2,
                    y2,
                    is_bounded,
                });
            }
        }

        blocks
    }

    /// Get grid dimensions (cols, rows) for a given block size
    pub fn grid_dimensions(&self, block_size: f64) -> (usize, usize) {
        let (min_x, min_y, max_x, max_y) = self.bounds();
        let cols = ((max_x - min_x) / block_size).ceil() as usize;
        let rows = ((max_y - min_y) / block_size).ceil() as usize;
        (cols, rows)
    }

    /// Convert polygon to a triangulation mesh for pathfinding
    ///
    /// This uses Earcut triangulation to create a navigation mesh
    /// that follows the polygon shape more precisely than axis-aligned blocks.
    ///
    /// Advantages over grid-based:
    /// - More natural paths following polygon boundaries
    /// - Better handling of irregular polygon shapes
    /// - Reduced memory usage for large areas
    ///
    /// Requires the `geo` feature to be enabled.
    #[cfg(feature = "geo")]
    pub fn to_triangulation_mesh(&self) -> Result<TriangulationMesh, &'static str> {
        if self.vertices.len() < 3 {
            return Err("Polygon must have at least 3 vertices");
        }

        // Flatten vertices for earcutr
        // earcutr expects flat array: [x0, y0, x1, y1, x2, y2, ...]
        let mut flat_vertices = Vec::with_capacity(self.vertices.len() * 2);
        for (x, y) in &self.vertices {
            flat_vertices.push(*x);
            flat_vertices.push(*y);
        }

        // Triangulate using earcutr
        let triangle_indices =
            earcutr::earcut(&flat_vertices, &[], 2).map_err(|_| "Triangulation failed")?;

        if triangle_indices.is_empty() {
            return Err("No triangles generated");
        }

        // Build triangles from indices
        let mut triangles = Vec::new();
        for chunk in triangle_indices.chunks(3) {
            if chunk.len() != 3 {
                continue;
            }

            let i0 = chunk[0];
            let i1 = chunk[1];
            let i2 = chunk[2];

            let v0 = self.vertices.get(i0).ok_or("Invalid vertex index")?;
            let v1 = self.vertices.get(i1).ok_or("Invalid vertex index")?;
            let v2 = self.vertices.get(i2).ok_or("Invalid vertex index")?;

            let center_x = (v0.0 + v1.0 + v2.0) / 3.0;
            let center_y = (v0.1 + v1.1 + v2.1) / 3.0;

            triangles.push(Triangle {
                vertices: [*v0, *v1, *v2],
                center: (center_x, center_y),
                indices: [i0, i1, i2],
            });
        }

        // Build adjacency graph
        // Two triangles are adjacent if they share an edge (2 vertices)
        let adjacency = build_triangle_adjacency(&triangles);

        Ok(TriangulationMesh {
            triangles,
            adjacency,
        })
    }
}

/// Triangulation mesh structure for pathfinding on non-Manhattan polygons
#[derive(Debug, Clone)]
#[cfg(feature = "geo")]
pub struct TriangulationMesh {
    /// List of triangles in the mesh
    pub triangles: Vec<Triangle>,
    /// Adjacency list: adjacency[i] = list of triangle indices adjacent to triangle i
    pub adjacency: Vec<Vec<usize>>,
}

#[cfg(feature = "geo")]
impl TriangulationMesh {
    /// Find which triangle contains a point
    pub fn find_triangle(&self, x: f64, y: f64) -> Option<usize> {
        self.triangles.iter().position(|tri| tri.contains(x, y))
    }

    /// Get the center of a triangle by index
    pub fn triangle_center(&self, index: usize) -> Option<(f64, f64)> {
        self.triangles.get(index).map(|tri| tri.center)
    }

    /// Find path between two points using A* on the triangle mesh
    pub fn find_path(&self, start: (f64, f64), end: (f64, f64)) -> Option<Vec<(f64, f64)>> {
        use alloc::collections::BTreeMap;
        use alloc::collections::BTreeSet;

        let start_tri = self.find_triangle(start.0, start.1)?;
        let end_tri = self.find_triangle(end.0, end.1)?;

        if start_tri == end_tri {
            // Same triangle, direct path
            return Some(vec![start, end]);
        }

        // A* pathfinding on triangle mesh
        let mut open_set = BTreeSet::new();
        let mut came_from: BTreeMap<usize, usize> = BTreeMap::new();
        let mut g_score: BTreeMap<usize, f64> = BTreeMap::new();
        let mut f_score: BTreeMap<usize, f64> = BTreeMap::new();

        let h_start = euclidean_distance(
            self.triangles[start_tri].center,
            self.triangles[end_tri].center,
        );

        g_score.insert(start_tri, 0.0);
        f_score.insert(start_tri, h_start);
        open_set.insert((OrderedFloat(h_start), start_tri));

        while !open_set.is_empty() {
            // Get node with lowest f_score
            let (_, current) = open_set.iter().next().copied().unwrap();
            open_set.remove(&(OrderedFloat(f_score[&current]), current));

            if current == end_tri {
                // Reconstruct path
                let mut path = vec![end];
                let mut curr = current;

                while let Some(&prev) = came_from.get(&curr) {
                    path.push(self.triangles[prev].center);
                    curr = prev;
                }

                path.push(start);
                path.reverse();
                return Some(path);
            }

            let current_g = g_score[&current];

            // Check all adjacent triangles
            for &neighbor in &self.adjacency[current] {
                let edge_cost = euclidean_distance(
                    self.triangles[current].center,
                    self.triangles[neighbor].center,
                );

                let tentative_g = current_g + edge_cost;
                let neighbor_g = g_score.get(&neighbor).copied().unwrap_or(f64::INFINITY);

                if tentative_g < neighbor_g {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g);

                    let h = euclidean_distance(
                        self.triangles[neighbor].center,
                        self.triangles[end_tri].center,
                    );
                    let f = tentative_g + h;
                    f_score.insert(neighbor, f);

                    open_set.insert((OrderedFloat(f), neighbor));
                }
            }
        }

        None
    }
}

/// Triangle in a triangulation mesh
#[derive(Debug, Clone, Copy)]
#[cfg(feature = "geo")]
pub struct Triangle {
    /// The three vertices of the triangle
    pub vertices: [(f64, f64); 3],
    /// Center (centroid) of the triangle
    pub center: (f64, f64),
    /// Indices of the vertices in the original polygon
    pub indices: [usize; 3],
}

#[cfg(feature = "geo")]
impl Triangle {
    /// Check if a point is inside the triangle using barycentric coordinates
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let (x1, y1) = self.vertices[0];
        let (x2, y2) = self.vertices[1];
        let (x3, y3) = self.vertices[2];

        let denom = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);
        if denom.abs() < 1e-10 {
            return false; // Degenerate triangle
        }

        let a = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / denom;
        let b = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / denom;
        let c = 1.0 - a - b;

        (0.0..=1.0).contains(&a) && (0.0..=1.0).contains(&b) && (0.0..=1.0).contains(&c)
    }

    /// Check if this triangle shares an edge with another triangle
    pub fn shares_edge(&self, other: &Triangle) -> bool {
        let mut shared_vertices = 0;
        for &idx1 in &self.indices {
            for &idx2 in &other.indices {
                if idx1 == idx2 {
                    shared_vertices += 1;
                }
            }
        }
        shared_vertices >= 2
    }
}

/// Build adjacency graph for triangles
#[cfg(feature = "geo")]
fn build_triangle_adjacency(triangles: &[Triangle]) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); triangles.len()];

    for i in 0..triangles.len() {
        for j in (i + 1)..triangles.len() {
            if triangles[i].shares_edge(&triangles[j]) {
                adjacency[i].push(j);
                adjacency[j].push(i);
            }
        }
    }

    adjacency
}

/// Helper function to calculate Euclidean distance
#[cfg(feature = "geo")]
fn euclidean_distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
}

/// Wrapper for f64 to make it Ord for use in BTreeSet
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg(feature = "geo")]
struct OrderedFloat(f64);

#[cfg(feature = "geo")]
impl Eq for OrderedFloat {}

#[cfg(feature = "geo")]
impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "geo")]
impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(core::cmp::Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_block_center() {
        let block = BoundedBlock {
            x1: 0.0,
            y1: 0.0,
            x2: 2.0,
            y2: 2.0,
            is_bounded: true,
        };
        assert_eq!(block.center(), (1.0, 1.0));
    }

    #[test]
    fn test_bounded_block_contains() {
        let block = BoundedBlock {
            x1: 0.0,
            y1: 0.0,
            x2: 2.0,
            y2: 2.0,
            is_bounded: true,
        };
        assert!(block.contains(1.0, 1.0));
        assert!(!block.contains(3.0, 3.0));
    }

    #[test]
    fn test_polygon_from_wkt() {
        let wkt = "POLYGON((0 0, 4 0, 4 3, 0 3, 0 0))";
        let poly = Polygon::from_wkt(wkt).unwrap();
        assert_eq!(poly.vertices.len(), 5);
        assert_eq!(poly.vertices[0], (0.0, 0.0));
        assert_eq!(poly.vertices[2], (4.0, 3.0));
    }

    #[test]
    fn test_polygon_bounds() {
        let poly = Polygon::from_coords(vec![(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (0.0, 3.0)]);
        let (min_x, min_y, max_x, max_y) = poly.bounds();
        assert_eq!(min_x, 0.0);
        assert_eq!(min_y, 0.0);
        assert_eq!(max_x, 4.0);
        assert_eq!(max_y, 3.0);
    }

    #[test]
    fn test_polygon_contains() {
        let poly = Polygon::from_coords(vec![
            (0.0, 0.0),
            (4.0, 0.0),
            (4.0, 3.0),
            (0.0, 3.0),
            (0.0, 0.0),
        ]);
        assert!(poly.contains(2.0, 1.5));
        assert!(!poly.contains(5.0, 5.0));
    }

    #[test]
    fn test_polygon_to_blocks() {
        let poly = Polygon::from_coords(vec![
            (0.0, 0.0),
            (4.0, 0.0),
            (4.0, 4.0),
            (0.0, 4.0),
            (0.0, 0.0),
        ]);
        let blocks = poly.to_bounded_blocks(1.0);
        assert!(blocks.len() > 0);
        // Check that some blocks are bounded
        let bounded_count = blocks.iter().filter(|b| b.is_bounded).count();
        assert!(bounded_count > 0);
    }

    #[test]
    #[cfg(feature = "geo")]
    fn test_triangulation_simple_square() {
        let poly = Polygon::from_coords(vec![(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]);
        let mesh = poly.to_triangulation_mesh().unwrap();

        // A square should be triangulated into 2 triangles
        assert_eq!(mesh.triangles.len(), 2);

        // Check that triangles are adjacent
        assert_eq!(mesh.adjacency[0].len(), 1);
        assert_eq!(mesh.adjacency[1].len(), 1);
    }

    #[test]
    #[cfg(feature = "geo")]
    fn test_triangulation_irregular_polygon() {
        // L-shaped polygon
        let poly = Polygon::from_coords(vec![
            (0.0, 0.0),
            (2.0, 0.0),
            (2.0, 2.0),
            (4.0, 2.0),
            (4.0, 4.0),
            (0.0, 4.0),
        ]);
        let mesh = poly.to_triangulation_mesh().unwrap();

        // Should have multiple triangles
        assert!(mesh.triangles.len() >= 4);

        // Check that all triangles have adjacencies
        for adjacencies in &mesh.adjacency {
            assert!(adjacencies.len() > 0 || mesh.triangles.len() == 1);
        }
    }

    #[test]
    #[cfg(feature = "geo")]
    fn test_triangle_contains_point() {
        let triangle = Triangle {
            vertices: [(0.0, 0.0), (4.0, 0.0), (2.0, 3.0)],
            center: (2.0, 1.0),
            indices: [0, 1, 2],
        };

        // Point inside
        assert!(triangle.contains(2.0, 1.0));

        // Point outside
        assert!(!triangle.contains(0.0, 5.0));
        assert!(!triangle.contains(-1.0, 0.0));
    }

    #[test]
    #[cfg(feature = "geo")]
    fn test_triangle_shares_edge() {
        let tri1 = Triangle {
            vertices: [(0.0, 0.0), (2.0, 0.0), (1.0, 2.0)],
            center: (1.0, 0.67),
            indices: [0, 1, 2],
        };

        let tri2 = Triangle {
            vertices: [(2.0, 0.0), (4.0, 0.0), (1.0, 2.0)],
            center: (2.33, 0.67),
            indices: [1, 3, 2],
        };

        // They share vertices at indices 1 and 2
        assert!(tri1.shares_edge(&tri2));
    }

    #[test]
    #[cfg(feature = "geo")]
    fn test_triangulation_mesh_find_triangle() {
        let poly = Polygon::from_coords(vec![(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]);
        let mesh = poly.to_triangulation_mesh().unwrap();

        // Point in the center should be in one of the triangles
        let tri_index = mesh.find_triangle(2.0, 2.0);
        assert!(tri_index.is_some());

        // Point outside should not be found
        let tri_index = mesh.find_triangle(10.0, 10.0);
        assert!(tri_index.is_none());
    }

    #[test]
    #[cfg(feature = "geo")]
    fn test_triangulation_mesh_pathfinding() {
        let poly = Polygon::from_coords(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
        let mesh = poly.to_triangulation_mesh().unwrap();

        // Find path from one corner to another
        let path = mesh.find_path((1.0, 1.0), (9.0, 9.0));
        assert!(path.is_some());

        let path = path.unwrap();
        assert!(path.len() >= 2); // At least start and end
        assert_eq!(path[0], (1.0, 1.0)); // Start point
        assert_eq!(path[path.len() - 1], (9.0, 9.0)); // End point
    }

    #[test]
    #[cfg(feature = "geo")]
    fn test_triangulation_mesh_pathfinding_complex() {
        // L-shaped polygon
        let poly = Polygon::from_coords(vec![
            (0.0, 0.0),
            (5.0, 0.0),
            (5.0, 5.0),
            (10.0, 5.0),
            (10.0, 10.0),
            (0.0, 10.0),
        ]);
        let mesh = poly.to_triangulation_mesh().unwrap();

        // Find path through the L-shape
        let path = mesh.find_path((2.0, 2.0), (8.0, 8.0));
        assert!(path.is_some());

        let path = path.unwrap();
        assert!(path.len() >= 2);
    }

    #[test]
    #[cfg(feature = "geo")]
    fn test_triangulation_error_cases() {
        // Too few vertices
        let poly = Polygon::from_coords(vec![(0.0, 0.0), (1.0, 0.0)]);
        let result = poly.to_triangulation_mesh();
        assert!(result.is_err());
    }
}
