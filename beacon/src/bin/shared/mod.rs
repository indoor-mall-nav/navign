pub mod constants;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
/// Capabilities that the device can report to the client.
/// The sum of all capabilities is sent as a single byte.
#[allow(dead_code)]
pub enum DeviceCapability {
    UnlockGate = 0x01,
    EnvironmentalData = 0x02,
    RssiCalibration = 0x04,
}

impl DeviceCapability {
    pub fn serialize(capabilities: &[DeviceCapability]) -> u8 {
        capabilities.iter().fold(0u8, |acc, cap| acc | (*cap as u8))
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum DeviceType {
    Merchant = 0x01,
    Pathway = 0x02,
    Connection = 0x03,
    Turnstile = 0x04,
}

impl DeviceType {
    pub fn serialize(&self) -> u8 {
        match self {
            Self::Merchant => 0x01,
            Self::Pathway => 0x02,
            Self::Connection => 0x03,
            Self::Turnstile => 0x04,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CryptoError {
    InvalidSignature,
    InvalidKey,
    VerificationFailed,
    BufferFull,
    RateLimited,
    ReplayDetected,
    ServerPublicKeyNotSet,
}

impl CryptoError {
    pub fn serialize(&self) -> u8 {
        match self {
            Self::InvalidSignature => 0x01,
            Self::InvalidKey => 0x02,
            Self::ServerPublicKeyNotSet => 0x03,
            Self::VerificationFailed => 0x04,
            Self::BufferFull => 0x05,
            Self::RateLimited => 0x06,
            Self::ReplayDetected => 0x07,
        }
    }
}

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
