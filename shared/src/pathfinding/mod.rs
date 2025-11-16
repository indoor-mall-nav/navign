//! Pathfinding module for indoor navigation
//!
//! This module provides pathfinding algorithms for both:
//! - Inner-area routing: A* pathfinding within a polygon area
//! - Inter-area routing: Dijkstra pathfinding between connected areas
//!
//! All algorithms work without arena allocation for better portability.

mod inner_area;
mod inter_area;
mod polygon;

pub use inner_area::{InnerPathError, find_path_in_area};
pub use inter_area::{
    AreaData, ConnectionData, ConnectivityLimits, InterPathError, RouteInstruction,
    find_path_between_areas,
};
pub use polygon::{BoundedBlock, Polygon};

#[cfg(feature = "geo")]
pub use polygon::{Triangle, TriangulationMesh};
