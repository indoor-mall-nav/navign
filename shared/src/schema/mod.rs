#[cfg(all(feature = "serde", feature = "alloc"))]
pub mod read_query;

#[cfg(all(feature = "serde", feature = "alloc"))]
pub use read_query::ReadQuery;

// Core schema modules
#[cfg(feature = "alloc")]
pub mod area;
#[cfg(feature = "alloc")]
pub mod beacon;

// Re-export core types
#[cfg(feature = "alloc")]
pub use area::{Area, Floor, FloorType};
#[cfg(feature = "alloc")]
pub use beacon::{Beacon, BeaconDevice, BeaconType};

// Mobile-specific exports
#[cfg(feature = "sql")]
pub use area::mobile::AreaMobile;
#[cfg(feature = "sql")]
pub use beacon::mobile::BeaconMobile;
