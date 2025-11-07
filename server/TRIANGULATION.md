# Triangulation-Based Pathfinding

## Overview

This document explains the triangulation-based pathfinding feature added to the Navign server. This feature improves pathfinding performance and accuracy for polygons that are not axis-aligned (i.e., polygons with edges that are not strictly horizontal or vertical).

## Problem

The original grid-based pathfinding approach works well for axis-aligned polygons (rectangles with horizontal and vertical edges). However, for rotated or irregular polygons, the grid-based approach has limitations:

1. **Inefficient grid cells**: Many grid cells are created that don't align well with the polygon shape
2. **Poor representation**: Non-axis-aligned edges are approximated poorly by rectangular grids
3. **Suboptimal paths**: The resulting paths may be unnecessarily long or awkward

## Solution

The triangulation-based approach uses the **spade** crate's Constrained Delaunay Triangulation (CDT) to decompose any polygon into triangles, then creates a grid overlay that better represents the polygon's actual shape.

### Key Components

#### 1. Triangle Struct
```rust
pub struct Triangle {
    pub p0: (f64, f64),
    pub p1: (f64, f64),
    pub p2: (f64, f64),
}
```

Represents a triangle with three vertices and provides methods for:
- Point-in-triangle testing using barycentric coordinates
- Calculating centroid
- Computing bounding box
- Calculating area

#### 2. Triangulation Function
```rust
pub fn triangulate_polygon(points: &[(f64, f64)]) -> Vec<Triangle>
```

Uses the **spade** crate's Constrained Delaunay Triangulation to decompose a simple polygon into triangles:
- Handles polygons with or without closing points
- Robust to degenerate cases
- Returns an empty vector for invalid input
- Filters triangles to only include those inside the polygon boundary

#### 3. Conversion to Bounded Blocks
```rust
pub fn triangulated_to_bounded_blocks(triangles: &[Triangle]) -> BoundedBlockArray<'static>
```

Converts triangulated representation to a grid of bounded blocks:
- Creates a uniform grid based on the number of triangles
- Tests each grid cell center against all triangles
- Marks cells as bounded/unbounded appropriately

## Usage

### Method 1: Explicit Triangulation
```rust
let polygon = &[
    (25.0, 0.0),
    (50.0, 25.0),
    (25.0, 50.0),
    (0.0, 25.0),
];
let poly = Polygon::from(polygon.as_slice());
let array = poly.as_bounded_block_array_triangulated();

// Use for pathfinding
let path = array.find_displacement((10.0, 25.0), (40.0, 25.0));
```

### Method 2: Automatic Selection (Recommended)
```rust
let poly = Polygon::from(polygon.as_slice());
let array = poly.as_bounded_block_array_auto();

// Automatically chooses:
// - Grid-based for axis-aligned polygons
// - Triangulation for non-axis-aligned polygons
```

### Method 3: Manual Grid-Based (Legacy)
```rust
let poly = Polygon::from(polygon.as_slice());
let array = poly.as_bounded_block_array();

// Uses the original grid-based approach
```

## API Reference

### Polygon Methods

#### `as_bounded_block_array_triangulated(&self) -> BoundedBlockArray<'static>`
Explicitly uses triangulation to convert the polygon to a bounded block array.

**When to use**: For non-axis-aligned polygons where you want guaranteed triangulation.

#### `as_bounded_block_array_auto(&self) -> BoundedBlockArray<'static>`
Automatically chooses the best method based on polygon shape.

**When to use**: Default choice for most use cases. Provides optimal performance.

#### `is_axis_aligned(&self) -> bool`
Checks if all polygon edges are horizontal or vertical.

**Returns**: `true` if axis-aligned, `false` otherwise.

## Algorithm Details

### Constrained Delaunay Triangulation (CDT)

The implementation uses the **spade** crate's Constrained Delaunay Triangulation algorithm:

1. **Insert vertices**: All polygon vertices are inserted into the triangulation
2. **Add constraints**: Polygon edges are added as constraints to maintain polygon boundaries
3. **Extract triangles**: Inner faces of the triangulation are extracted
4. **Filter triangles**: Only triangles whose centroids lie inside the polygon are kept

**Advantages of CDT**:
- More robust than ear clipping for complex polygons
- Better quality triangles (closer to equilateral)
- Industry-standard algorithm used in many applications
- Handles edge cases and degenerate geometries gracefully

**Time Complexity**: O(n log n) for a polygon with n vertices
**Space Complexity**: O(n) for storing the triangles

### Grid Overlay

After triangulation:
1. Compute the bounding box of all triangles
2. Create a uniform grid with resolution based on triangle count
3. For each grid cell, test if its center is inside any triangle
4. Mark cells as bounded or unbounded

**Grid Resolution**: `sqrt(num_triangles) * 2`, minimum 3×3

## Performance Considerations

### Grid-Based Approach
- **Pros**: Fast for axis-aligned polygons, simple implementation
- **Cons**: Poor for rotated polygons, may create many unnecessary blocks
- **Best for**: Rectangular rooms, corridors, axis-aligned spaces

### Triangulation-Based Approach
- **Pros**: Accurate for any polygon shape, better path quality
- **Cons**: Slight overhead for triangulation computation
- **Best for**: Rotated polygons, irregular spaces, diagonal corridors

## Examples

### Example 1: Rotated Diamond
```rust
let diamond = [
    (50.0, 0.0),
    (100.0, 50.0),
    (50.0, 100.0),
    (0.0, 50.0),
];
let poly = Polygon::from(diamond.as_slice());
let array = poly.as_bounded_block_array_triangulated();
let path = array.find_displacement((50.0, 25.0), (50.0, 75.0));
```

### Example 2: Irregular Pentagon
```rust
let pentagon = [
    (1.0, 0.0),
    (1.951, 0.309),
    (1.588, 1.118),
    (0.412, 1.118),
    (0.049, 0.309),
];
let poly = Polygon::from(pentagon.as_slice());
let array = poly.as_bounded_block_array_auto();
```

## Testing

The implementation includes comprehensive tests:

- **Triangle operations**: Point containment, centroid, area calculation
- **Triangulation**: Simple shapes (square, pentagon), rotated polygons
- **Pathfinding**: End-to-end tests with various polygon shapes
- **Auto-detection**: Verification of axis-alignment detection

Run tests with:
```bash
cargo test triangulation
cargo test displacement_route
```

## Future Enhancements

Potential improvements for future versions:

1. **Constrained Delaunay Triangulation**: Higher quality triangulation for complex polygons
2. **Adaptive Grid Resolution**: Dynamic grid sizing based on polygon complexity
3. **Triangle Graph**: Direct pathfinding on triangle adjacency graph
4. **Caching**: Cache triangulation results for frequently used polygons
5. **Parallel Triangulation**: Multi-threaded processing for large polygons

## References

- Constrained Delaunay Triangulation: Chew, L. Paul (1987). "Constrained Delaunay triangulations"
- Spade Crate: https://github.com/Stoeoef/spade
- Barycentric Coordinates: Christer Ericson (2004). Real-Time Collision Detection
- A* Pathfinding: Hart, P. E.; Nilsson, N. J.; Raphael, B. (1968)

## License

This implementation is part of the Navign project and is licensed under the MIT License.
