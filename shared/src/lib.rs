#![no_std]

#[cfg(all(feature = "heapless", feature = "alloc"))]
compile_error!("Features 'heapless' and 'alloc' cannot be enabled at the same time.");

#[cfg(all(not(feature = "heapless"), not(feature = "alloc")))]
compile_error!("Either feature 'heapless' or 'alloc' must be enabled.");

#[cfg(feature = "alloc")]
extern crate alloc;

mod ble;
mod traits;

pub use ble::challenge::ServerChallenge;
pub use ble::device_caps::DeviceCapabilities;
pub use ble::device_type::DeviceTypes;
pub use ble::nonce::Nonce;
pub use ble::proof::Proof;
pub use traits::{depacketize::Depacketize, packetize::Packetize};
