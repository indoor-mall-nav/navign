// Library interface for navign-maintenance
// Exposes types and functions for testing

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct KeyMetadata {
    pub key_name: String,
    pub private_key_file: String,
    pub public_key_hex: String,
    pub generated_at: String,
    pub fused: bool,
    pub chip_info: Option<String>,
}

// Re-export main protobuf types for testing
pub mod proto {
    pub mod navign {
        pub mod orchestrator {
            pub mod sync {
                tonic::include_proto!("navign.orchestrator.sync");
            }
        }
    }
}
