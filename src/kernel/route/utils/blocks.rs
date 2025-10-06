#![allow(unused)]
use crate::kernel::route::types::CloneIn;
use std::collections::BTreeSet;
use std::ops::Index;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundedBlock {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub is_bounded: bool,
}

impl BoundedBlock {
    pub fn width(&self) -> f64 {
        self.x2 - self.x1
    }
    pub fn height(&self) -> f64 {
        self.y2 - self.y1
    }

    pub fn is_point_inside(&self, x: f64, y: f64) -> bool {
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }

    pub fn center(&self) -> (f64, f64) {
        ((self.x1 + self.x2) / 2.0, (self.y1 + self.y2) / 2.0)
    }
}

#[derive(Debug)]
pub struct BoundedBlockArray<'a> {
    /// Please be aware that the blocks are probably not share a same size.
    pub blocks: &'a [BoundedBlock],
    /// The width of the memory buffer that the blocks are mapped to, i.e., how many blocks fit in one row.
    pub memory_width: usize,
    /// The height of the memory buffer that the blocks are mapped to, i.e., how many blocks fit in one column.
    pub memory_height: usize,
    /// The actual width of the area covered by the blocks.
    pub width: f64,
    /// The actual height of the area covered by the blocks.
    pub height: f64,
}

impl<'a> BoundedBlockArray<'a> {
    pub fn coords_x(&self) -> Vec<f64> {
        let mut xs = Vec::with_capacity(self.memory_width + 1);
        for i in 0..self.memory_width {
            xs.push(self.blocks[i].x1);
        }
        if let Some(last) = self.blocks.get(self.memory_width * self.memory_height - 1) {
            xs.push(last.x2);
        }
        xs
    }

    pub fn coords_y(&self) -> Vec<f64> {
        let mut ys = Vec::with_capacity(self.memory_height + 1);
        for i in 0..self.memory_height {
            ys.push(self.blocks[i * self.memory_width].y1);
        }
        if let Some(last) = self.blocks.get(self.memory_width * self.memory_height - 1) {
            ys.push(last.y2);
        }
        ys
    }
}

/// A trait for blocks that can be part of a contiguous grid, allowing navigation to adjacent blocks.
pub trait ContiguousBlockArray<T: Sized + Clone + Copy>: Index<usize, Output = T> {
    fn fit(&self, x: f64, y: f64) -> Option<T>;
    fn memory_width(&self) -> usize;
    fn memory_height(&self) -> usize;
    fn access(&self, x: usize, y: usize) -> Option<T> {
        if x < self.memory_width() && y < self.memory_height() {
            let index = y * self.memory_width() + x;
            Some(*self.index(index))
        } else {
            None
        }
    }
    fn get_index(&self, block: T) -> Option<usize>;
    fn deaccess(&self, block: T) -> Option<(usize, usize)> {
        self.get_index(block).and_then(|index| {
            if index < self.memory_width() * self.memory_height() {
                let x = index % self.memory_width();
                let y = index / self.memory_width();
                Some((x, y))
            } else {
                None
            }
        })
    }
    fn up(&self, block: T) -> Option<T> {
        self.deaccess(block).and_then(|(x, y)| self.access_up(x, y))
    }
    fn down(&self, block: T) -> Option<T> {
        self.deaccess(block)
            .and_then(|(x, y)| self.access_down(x, y))
    }
    fn left(&self, block: T) -> Option<T> {
        self.deaccess(block)
            .and_then(|(x, y)| self.access_left(x, y))
    }
    fn right(&self, block: T) -> Option<T> {
        self.deaccess(block)
            .and_then(|(x, y)| self.access_right(x, y))
    }

    fn access_up(&self, x: usize, y: usize) -> Option<T> {
        (y > 0).then(|| self.access(x, y - 1)).flatten()
    }
    fn access_down(&self, x: usize, y: usize) -> Option<T> {
        (y + 1 < self.memory_height())
            .then(|| self.access(x, y + 1))
            .flatten()
    }
    fn access_left(&self, x: usize, y: usize) -> Option<T> {
        (x > 0).then(|| self.access(x - 1, y)).flatten()
    }
    fn access_right(&self, x: usize, y: usize) -> Option<T> {
        (x + 1 < self.memory_width())
            .then(|| self.access(x + 1, y))
            .flatten()
    }

    fn contiguous_access_matrix(&self, x: usize, y: usize) -> Vec<T> {
        vec![
            self.access_up(x, y),
            self.access_down(x, y),
            self.access_left(x, y),
            self.access_right(x, y),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl Index<usize> for BoundedBlockArray<'_> {
    type Output = BoundedBlock;

    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}

impl<'a> ContiguousBlockArray<BoundedBlock> for BoundedBlockArray<'a> {
    fn fit(&self, x: f64, y: f64) -> Option<BoundedBlock> {
        for block in self.blocks.iter() {
            if block.is_bounded && block.is_point_inside(x, y) {
                return Some(*block);
            }
        }
        None
    }

    fn memory_width(&self) -> usize {
        self.memory_width
    }

    fn memory_height(&self) -> usize {
        self.memory_height
    }

    fn get_index(&self, block: BoundedBlock) -> Option<usize> {
        self.blocks.iter().position(|&b| b == block)
    }
}

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_block() {
        let block = BoundedBlock {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            is_bounded: true,
        };
        assert_eq!(block.width(), 10.0);
        assert_eq!(block.height(), 10.0);
        assert!(block.is_point_inside(5.0, 5.0));
        assert!(!block.is_point_inside(15.0, 5.0));
    }

    #[test]
    fn test_ray_cast() {
        let polygon = &[
            (0.0, 0.0),
            (0.0, 3.0),
            (1.0, 3.0),
            (1.0, 1.0),
            (2.0, 1.0),
            (2.0, 0.0),
            (0.0, 0.0),
        ];
        let poly = Polygon::from(polygon.as_slice());
        assert!(poly.is_point_inside(0.5, 0.5));
        assert!(poly.is_point_inside(1.5, 0.5));
        assert!(!poly.is_point_inside(4.0, 0.5));
    }

    #[test]
    fn test_ray_cast_unbound() {
        let polygon = &[
            (0.0, 0.0),
            (0.0, 3.0),
            (1.0, 3.0),
            (1.0, 1.0),
            (2.0, 1.0),
            (2.0, 0.0),
            (0.0, 0.0),
        ];
        let poly = Polygon::from(polygon.as_slice()).into_unbound();
        assert!(!poly.is_point_inside(0.5, 0.5));
        assert!(!poly.is_point_inside(1.5, 0.5));
        assert!(poly.is_point_inside(4.0, 0.5));
    }

    #[test]
    fn test_get_sorted_coords() {
        let polygon = &[(2.0, 3.0), (0.0, 1.0), (1.0, 2.0)];
        let poly = Polygon::from(polygon.as_slice());
        let (xs, ys) = poly.get_sorted_coords();
        assert_eq!(xs, vec![0.0, 1.0, 2.0]);
        assert_eq!(ys, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_bounded_block_array() {
        let blocks = vec![
            BoundedBlock {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 1.0,
                y1: 0.0,
                x2: 2.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 0.0,
                y1: 1.0,
                x2: 1.0,
                y2: 2.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 1.0,
                y1: 1.0,
                x2: 2.0,
                y2: 2.0,
                is_bounded: false,
            },
        ];
        let array = BoundedBlockArray {
            blocks: &blocks,
            memory_width: 2,
            memory_height: 2,
            width: 2.0,
            height: 2.0,
        };

        assert_eq!(array.memory_width(), 2);
        assert_eq!(array.memory_height(), 2);

        let block = array.fit(0.5, 0.5).unwrap();
        assert_eq!(block.x1, 0.0);
        assert_eq!(block.y1, 0.0);

        let up_block = array.up(block);
        assert!(up_block.is_none());

        let down_block = array.down(block).unwrap();
        assert_eq!(down_block.x1, 0.0);
        assert_eq!(down_block.y1, 1.0);

        let right_block = array.right(block).unwrap();
        assert_eq!(right_block.x1, 1.0);
        assert_eq!(right_block.y1, 0.0);

        let left_block = array.left(block);
        assert!(left_block.is_none());
    }

    #[test]
    fn test_bounded_block_array_coords() {
        let blocks = vec![
            BoundedBlock {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 1.0,
                y1: 0.0,
                x2: 2.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 0.0,
                y1: 1.0,
                x2: 1.0,
                y2: 2.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 1.0,
                y1: 1.0,
                x2: 2.0,
                y2: 2.0,
                is_bounded: false,
            },
        ];
        let array = BoundedBlockArray {
            blocks: &blocks,
            memory_width: 2,
            memory_height: 2,
            width: 2.0,
            height: 2.0,
        };

        let xs = array.coords_x();
        assert_eq!(xs, vec![0.0, 1.0, 2.0]);

        let ys = array.coords_y();
        assert_eq!(ys, vec![0.0, 1.0, 2.0]);
    }

    #[test]
    fn test_polygon_to_bounded_block_array() {
        let polygon = &[(0.0, 0.0), (0.0, 2.0), (2.0, 2.0), (2.0, 0.0), (0.0, 0.0)];
        let poly = Polygon::from(polygon.as_slice());
        let array = poly.as_bounded_block_array();

        assert_eq!(array.memory_width, 1);
        assert_eq!(array.memory_height, 1);
        assert_eq!(array.width, 2.0);
        assert_eq!(array.height, 2.0);

        for block in array.blocks.iter() {
            assert!(block.is_bounded);
        }
    }

    #[test]
    fn test_advanced_polygon_to_bounded_block_array() {
        let polygon = &[
            (0.0, 0.0),
            (0.0, 3.0),
            (1.0, 3.0),
            (1.0, 1.0),
            (2.0, 1.0),
            (2.0, 0.0),
            (0.0, 0.0),
        ];
        let poly = Polygon::from(polygon.as_slice());
        let array = poly.as_bounded_block_array();

        assert_eq!(array.memory_width, 2);
        assert_eq!(array.memory_height, 2);

        assert_eq!(array.width, 2.0);
        assert_eq!(array.height, 3.0);

        let mut bounded_count = 0;
        let mut unbounded_count = 0;

        for block in array.blocks.iter() {
            if block.is_bounded {
                bounded_count += 1;
            } else {
                unbounded_count += 1;
            }
        }

        assert_eq!(bounded_count, 3);
        assert_eq!(unbounded_count, 1);
    }
}
