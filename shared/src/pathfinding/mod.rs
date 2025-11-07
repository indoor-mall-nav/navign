//! Pathfinding module for indoor navigation
//!
//! This module provides pathfinding algorithms for both:
//! - Inner-area routing: A* pathfinding within a polygon area
//! - Inter-area routing: Dijkstra pathfinding between connected areas
//!
//! All algorithms work without arena allocation for better portability.

#[cfg(feature = "alloc")]
mod inner_area;
#[cfg(feature = "alloc")]
mod inter_area;
#[cfg(feature = "alloc")]
mod polygon;

#[cfg(feature = "alloc")]
pub use inner_area::{InnerPathError, find_path_in_area};
#[cfg(feature = "alloc")]
pub use inter_area::{InterPathError, RouteInstruction, find_path_between_areas};
#[cfg(feature = "alloc")]
pub use polygon::{BoundedBlock, Polygon};
