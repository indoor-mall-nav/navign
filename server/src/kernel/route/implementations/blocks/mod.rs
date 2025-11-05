pub mod bounded_block;
pub mod bounded_block_array;
pub mod contiguous_block_array;
pub mod polygon;
pub mod triangulation;
mod test;

pub use bounded_block::BoundedBlock;
pub use bounded_block_array::BoundedBlockArray;
pub use contiguous_block_array::ContiguousBlockArray;
pub use polygon::Polygon;
// Re-export triangulation types for external use if needed
#[allow(unused_imports)]
pub use triangulation::{triangulate_polygon, triangulated_to_bounded_blocks, Triangle};
