#![allow(unused)]
use super::BoundedBlock;

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
