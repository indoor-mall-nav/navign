//! Polygon and block structures for pathfinding

use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
            let parts: Vec<&str> = coord.trim().split_whitespace().collect();
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

    /// Check if a point is inside the polygon using ray casting
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let n = self.vertices.len();
        if n < 3 {
            return false;
        }

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

    /// TODO: Future implementation for triangulation-based pathfinding
    /// This will use Delaunay triangulation to create a navigation mesh
    /// that follows the polygon shape more precisely than axis-aligned blocks.
    ///
    /// Advantages over grid-based:
    /// - More natural paths following polygon boundaries
    /// - Better handling of irregular polygon shapes
    /// - Reduced memory usage for large areas
    ///
    /// Implementation will use:
    /// - Constrained Delaunay Triangulation (CDT)
    /// - Triangle adjacency graph
    /// - Funnel algorithm for path smoothing
    #[allow(dead_code)]
    fn to_triangulation_mesh(&self) -> Result<TriangulationMesh, &'static str> {
        // Placeholder for future implementation
        Err("Triangulation-based pathfinding not yet implemented")
    }
}

/// Placeholder for future triangulation mesh structure
#[allow(dead_code)]
struct TriangulationMesh {
    triangles: Vec<Triangle>,
    adjacency: Vec<Vec<usize>>,
}

/// Placeholder for triangle structure
#[allow(dead_code)]
struct Triangle {
    vertices: [(f64, f64); 3],
    center: (f64, f64),
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
}
