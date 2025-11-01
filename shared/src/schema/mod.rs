#[cfg(all(feature = "serde", feature = "alloc"))]
pub mod read_query;

#[cfg(all(feature = "serde", feature = "alloc"))]
pub use read_query::ReadQuery;
