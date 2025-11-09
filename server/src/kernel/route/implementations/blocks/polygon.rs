use crate::kernel::route::implementations::{BoundedBlock, BoundedBlockArray};
use crate::kernel::route::types::CloneIn;
use std::collections::BTreeSet;

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

        // Handle edge case: polygons must have at least 2 unique coordinates
        let width = xs.last().copied().unwrap_or(0.0) - xs.first().copied().unwrap_or(0.0);
        let height = ys.last().copied().unwrap_or(0.0) - ys.first().copied().unwrap_or(0.0);

        let memory_width = xs.len().saturating_sub(1);
        let memory_height = ys.len().saturating_sub(1);
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
            width,
            height,
        }
    }
}
