// Re-export constants from navign-shared
pub mod constants {
    pub use navign_shared::constants::*;
}

// Re-export types from navign-shared
pub use navign_shared::{DeviceCapabilities, DeviceTypes};
pub use navign_shared::errors::CryptoError;

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum BleError {
    SetupFailed,
    NotConnected,
    SendFailed,
    ReceiveFailed,
    ParseError,
    BufferFull,
}
