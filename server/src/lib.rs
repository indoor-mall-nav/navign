//! Navign Server Library
//!
//! This library exposes server functionality for use by binaries (main server, migration tool, etc.)

pub mod error;
pub mod pg;
pub mod schema;
pub mod state;

// Re-export commonly used types
pub use error::{Result, ServerError};
pub use state::AppState;
