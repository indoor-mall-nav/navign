use super::{BoundedBlock, BoundedBlockArray};
#[cfg(test)]
use super::ContiguousBlockArray;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation as _};

/// Represents a triangle in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    pub p0: (f64, f64),
    pub p1: (f64, f64),
    pub p2: (f64, f64),
}

impl Triangle {
    /// Check if a point is inside this triangle using barycentric coordinates
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        let (x0, y0) = self.p0;
        let (x1, y1) = self.p1;
        let (x2, y2) = self.p2;

        let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);
        if denom.abs() < 1e-10 {
            return false; // Degenerate triangle
        }

        let a = ((y1 - y2) * (x - x2) + (x2 - x1) * (y - y2)) / denom;
        let b = ((y2 - y0) * (x - x2) + (x0 - x2) * (y - y2)) / denom;
        let c = 1.0 - a - b;

        a >= 0.0 && b >= 0.0 && c >= 0.0
    }

    /// Get the centroid of the triangle
    pub fn centroid(&self) -> (f64, f64) {
        (
            (self.p0.0 + self.p1.0 + self.p2.0) / 3.0,
            (self.p0.1 + self.p1.1 + self.p2.1) / 3.0,
        )
    }

    /// Get the bounding box of the triangle
    pub fn bounding_box(&self) -> (f64, f64, f64, f64) {
        let min_x = self.p0.0.min(self.p1.0).min(self.p2.0);
        let max_x = self.p0.0.max(self.p1.0).max(self.p2.0);
        let min_y = self.p0.1.min(self.p1.1).min(self.p2.1);
        let max_y = self.p0.1.max(self.p1.1).max(self.p2.1);
        (min_x, min_y, max_x, max_y)
    }

    /// Calculate the area of the triangle
    pub fn area(&self) -> f64 {
        let (x0, y0) = self.p0;
        let (x1, y1) = self.p1;
        let (x2, y2) = self.p2;
        ((x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0)).abs() / 2.0
    }
}

/// Triangulate a polygon using Constrained Delaunay Triangulation from the spade crate
/// Returns a list of triangles that decompose the polygon
pub fn triangulate_polygon(points: &[(f64, f64)]) -> Vec<Triangle> {
    if points.len() < 3 {
        return vec![];
    }

    // Remove duplicate last point if it's the same as the first
    let mut vertices: Vec<(f64, f64)> = points.to_vec();
    if vertices.len() > 1 && vertices[0] == vertices[vertices.len() - 1] {
        vertices.pop();
    }

    if vertices.len() < 3 {
        return vec![];
    }

    // Create a Constrained Delaunay Triangulation
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f64>>::new();
    
    // Insert all vertices
    let mut vertex_handles = Vec::new();
    for &(x, y) in &vertices {
        match cdt.insert(Point2::new(x, y)) {
            Ok(handle) => vertex_handles.push(handle),
            Err(_) => {
                // If insertion fails (duplicate point), skip it
                continue;
            }
        }
    }

    // Add constraints (edges of the polygon)
    for i in 0..vertex_handles.len() {
        let v1 = vertex_handles[i];
        let v2 = vertex_handles[(i + 1) % vertex_handles.len()];
        
        // Add edge as constraint
        let _ = cdt.add_constraint(v1, v2);
    }

    // Extract triangles from the triangulation
    let mut triangles = Vec::new();
    
    for face in cdt.inner_faces() {
        let [v0, v1, v2] = face.vertices();
        let p0_pos = v0.position();
        let p1_pos = v1.position();
        let p2_pos = v2.position();
        
        let tri = Triangle {
            p0: (p0_pos.x, p0_pos.y),
            p1: (p1_pos.x, p1_pos.y),
            p2: (p2_pos.x, p2_pos.y),
        };

        // Check if this triangle's centroid is inside the polygon
        // This filters out triangles that are outside the constrained region
        let centroid = tri.centroid();
        if is_point_in_polygon(&vertices, centroid.0, centroid.1) {
            triangles.push(tri);
        }
    }

    triangles
}

/// Check if a point is inside a polygon using ray casting algorithm
fn is_point_in_polygon(polygon: &[(f64, f64)], x: f64, y: f64) -> bool {
    let mut inside = false;
    let n = polygon.len();
    let mut j = n - 1;
    
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        
        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    
    inside
}

/// Convert triangulated polygon to BoundedBlockArray for pathfinding
/// This creates a hybrid representation using both triangles and grid
pub fn triangulated_to_bounded_blocks(triangles: &[Triangle]) -> BoundedBlockArray<'static> {
    if triangles.is_empty() {
        return BoundedBlockArray {
            blocks: &[],
            memory_width: 0,
            memory_height: 0,
            width: 0.0,
            height: 0.0,
        };
    }

    // Find the overall bounding box
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for triangle in triangles {
        let (bbox_min_x, bbox_min_y, bbox_max_x, bbox_max_y) = triangle.bounding_box();
        min_x = min_x.min(bbox_min_x);
        max_x = max_x.max(bbox_max_x);
        min_y = min_y.min(bbox_min_y);
        max_y = max_y.max(bbox_max_y);
    }

    // Create a grid resolution based on the number of triangles
    // More triangles = finer grid for better representation
    let grid_size = (triangles.len() as f64).sqrt().ceil() as usize * 2;
    let grid_size = grid_size.max(3).min(50); // Min 3x3, max 50x50 to prevent excessive memory

    let cell_width = (max_x - min_x) / grid_size as f64;
    let cell_height = (max_y - min_y) / grid_size as f64;

    let mut blocks = Vec::new();

    for y in 0..grid_size {
        for x in 0..grid_size {
            let x1 = min_x + x as f64 * cell_width;
            let y1 = min_y + y as f64 * cell_height;
            let x2 = x1 + cell_width;
            let y2 = y1 + cell_height;

            let center_x = (x1 + x2) / 2.0;
            let center_y = (y1 + y2) / 2.0;

            // Check if the center of this cell is inside any triangle
            let is_bounded = triangles
                .iter()
                .any(|tri| tri.contains_point(center_x, center_y));

            blocks.push(BoundedBlock {
                x1,
                y1,
                x2,
                y2,
                is_bounded,
            });
        }
    }

    BoundedBlockArray {
        blocks: Box::leak(blocks.into_boxed_slice()),
        memory_width: grid_size,
        memory_height: grid_size,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_contains_point() {
        let triangle = Triangle {
            p0: (0.0, 0.0),
            p1: (4.0, 0.0),
            p2: (2.0, 3.0),
        };

        assert!(triangle.contains_point(2.0, 1.0));
        assert!(triangle.contains_point(1.0, 0.5));
        assert!(!triangle.contains_point(0.0, 3.0));
        assert!(!triangle.contains_point(5.0, 0.0));
    }

    #[test]
    fn test_triangle_centroid() {
        let triangle = Triangle {
            p0: (0.0, 0.0),
            p1: (3.0, 0.0),
            p2: (0.0, 3.0),
        };
        let (cx, cy) = triangle.centroid();
        assert!((cx - 1.0).abs() < 1e-10);
        assert!((cy - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_triangle_area() {
        let triangle = Triangle {
            p0: (0.0, 0.0),
            p1: (4.0, 0.0),
            p2: (0.0, 3.0),
        };
        assert!((triangle.area() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_triangulate_simple_square() {
        let square = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let triangles = triangulate_polygon(&square);
        assert_eq!(triangles.len(), 2);

        // Check that triangles cover the square
        let total_area: f64 = triangles.iter().map(|t| t.area()).sum();
        assert!((total_area - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_triangulate_with_closing_point() {
        let square = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)];
        let triangles = triangulate_polygon(&square);
        assert_eq!(triangles.len(), 2);
    }

    #[test]
    fn test_triangulate_rotated_rectangle() {
        // A rotated rectangle (45 degrees)
        let rect = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0)];
        let triangles = triangulate_polygon(&rect);
        assert_eq!(triangles.len(), 2);

        // Verify we can find points inside
        let blocks = triangulated_to_bounded_blocks(&triangles);
        assert!(blocks.memory_width > 0);
        assert!(blocks.memory_height > 0);

        // The center should be covered
        let center_block = blocks.fit(1.0, 1.0);
        assert!(center_block.is_some());
        if let Some(block) = center_block {
            assert!(block.is_bounded);
        }
    }

    #[test]
    fn test_triangulate_pentagon() {
        // Regular pentagon approximation
        let pentagon = vec![
            (1.0, 0.0),
            (1.951, 0.309),
            (1.588, 1.118),
            (0.412, 1.118),
            (0.049, 0.309),
        ];
        let triangles = triangulate_polygon(&pentagon);
        assert_eq!(triangles.len(), 3);

        let total_area: f64 = triangles.iter().map(|t| t.area()).sum();
        assert!(total_area > 1.0); // Pentagon should have positive area
    }

    #[test]
    fn test_triangulated_to_blocks() {
        let square = vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)];
        let triangles = triangulate_polygon(&square);
        let blocks = triangulated_to_bounded_blocks(&triangles);

        assert!(blocks.memory_width >= 3);
        assert!(blocks.memory_height >= 3);

        // Check that the center is properly marked as bounded
        let center_block = blocks.fit(1.0, 1.0);
        assert!(center_block.is_some());
        assert!(center_block.unwrap().is_bounded);
    }
}
