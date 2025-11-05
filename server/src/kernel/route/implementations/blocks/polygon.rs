use crate::kernel::route::implementations::{BoundedBlock, BoundedBlockArray};
use crate::kernel::route::types::CloneIn;
use std::collections::BTreeSet;
use super::triangulation::{triangulate_polygon, triangulated_to_bounded_blocks};

#[derive(Debug, Clone, Copy, Default)]
pub struct Polygon<'a> {
    pub points: &'a [(f64, f64)],
    /// Sometimes the polygon indicates areas _outside_ the polygon are bounded.
    pub bounding: bool,
}

impl<'a, 'b: 'a> CloneIn<'b> for Polygon<'a> {
    type Cloned = Polygon<'b>;
    fn clone_in(&self, allocator: &'b bumpalo::Bump) -> Polygon<'b> {
        Polygon {
            points: allocator.alloc_slice_copy(self.points),
            bounding: self.bounding,
        }
    }
}

impl<'a> From<Vec<(f64, f64)>> for Polygon<'a> {
    fn from(points: Vec<(f64, f64)>) -> Self {
        Self {
            points: Box::leak(points.into_boxed_slice()),
            bounding: true,
        }
    }
}

impl<'a> From<&'a [(f64, f64)]> for Polygon<'a> {
    fn from(points: &'a [(f64, f64)]) -> Self {
        Self {
            points,
            bounding: true,
        }
    }
}

impl<'a> Polygon<'a> {
    #[allow(unused)]
    pub fn into_unbound(self) -> Self {
        Self {
            bounding: false,
            ..self
        }
    }

    /// Ray-casting algorithm to determine if a point is inside the polygon
    pub fn is_point_inside(&self, x: f64, y: f64) -> bool {
        let mut inside = false;
        let n = self.points.len();
        let mut j = n - 1;
        for i in 0..n {
            let (xi, yi) = self.points[i];
            let (xj, yj) = self.points[j];
            if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
                inside = !inside;
            }
            j = i;
        }
        if self.bounding { inside } else { !inside }
    }

    pub fn get_sorted_coords(&self) -> (Vec<f64>, Vec<f64>) {
        let xs: BTreeSet<u64> = self.points.iter().map(|(x, _)| *x as u64).collect();
        let ys: BTreeSet<u64> = self.points.iter().map(|(_, y)| *y as u64).collect();
        let xs: Vec<f64> = xs.into_iter().map(|x| x as f64).collect();
        let ys: Vec<f64> = ys.into_iter().map(|y| y as f64).collect();
        (xs, ys)
    }

    pub fn as_bounded_block_array(&self) -> BoundedBlockArray<'a> {
        let (xs, ys) = self.get_sorted_coords();
        let memory_width = xs.len() - 1;
        let memory_height = ys.len() - 1;
        let mut blocks = Vec::with_capacity(memory_width * memory_height);
        for y in 0..memory_height {
            for x in 0..memory_width {
                let block = BoundedBlock {
                    x1: xs[x],
                    y1: ys[y],
                    x2: xs[x + 1],
                    y2: ys[y + 1],
                    is_bounded: self
                        .is_point_inside((xs[x] + xs[x + 1]) / 2.0, (ys[y] + ys[y + 1]) / 2.0),
                };
                blocks.push(block);
            }
        }
        BoundedBlockArray {
            blocks: Box::leak(blocks.into_boxed_slice()),
            memory_width,
            memory_height,
            width: *xs.last().unwrap() - xs[0],
            height: *ys.last().unwrap() - ys[0],
        }
    }

    /// Convert polygon to bounded block array using triangulation
    /// This is more efficient for non-axis-aligned polygons
    pub fn as_bounded_block_array_triangulated(&self) -> BoundedBlockArray<'static> {
        let triangles = triangulate_polygon(self.points);
        triangulated_to_bounded_blocks(&triangles)
    }

    /// Automatically choose the best representation method
    /// Uses triangulation for non-axis-aligned polygons, grid for axis-aligned ones
    pub fn as_bounded_block_array_auto(&self) -> BoundedBlockArray<'static> {
        if self.is_axis_aligned() {
            // Convert to static lifetime by cloning
            let (xs, ys) = self.get_sorted_coords();
            let memory_width = xs.len() - 1;
            let memory_height = ys.len() - 1;
            let mut blocks = Vec::with_capacity(memory_width * memory_height);
            for y in 0..memory_height {
                for x in 0..memory_width {
                    let block = BoundedBlock {
                        x1: xs[x],
                        y1: ys[y],
                        x2: xs[x + 1],
                        y2: ys[y + 1],
                        is_bounded: self
                            .is_point_inside((xs[x] + xs[x + 1]) / 2.0, (ys[y] + ys[y + 1]) / 2.0),
                    };
                    blocks.push(block);
                }
            }
            BoundedBlockArray {
                blocks: Box::leak(blocks.into_boxed_slice()),
                memory_width,
                memory_height,
                width: *xs.last().unwrap() - xs[0],
                height: *ys.last().unwrap() - ys[0],
            }
        } else {
            self.as_bounded_block_array_triangulated()
        }
    }

    /// Check if the polygon is axis-aligned (all edges are horizontal or vertical)
    pub fn is_axis_aligned(&self) -> bool {
        if self.points.len() < 2 {
            return true;
        }

        for i in 0..self.points.len() {
            let (x1, y1) = self.points[i];
            let (x2, y2) = self.points[(i + 1) % self.points.len()];

            // Edge is neither horizontal nor vertical
            if (x1 - x2).abs() > 1e-10 && (y1 - y2).abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}
