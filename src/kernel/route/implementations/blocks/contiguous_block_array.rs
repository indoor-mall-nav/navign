#![allow(unused)]
use std::ops::Index;
use super::{BoundedBlock, BoundedBlockArray};

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
