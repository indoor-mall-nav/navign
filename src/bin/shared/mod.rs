use bleps::att::Uuid;
use heapless::Vec;

pub mod constants;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
/// Capabilities that the device can report to the client.
/// The sum of all capabilities is sent as a single byte.
pub enum DeviceCapability {
    UnlockGate = 0x01,
    EnvironmentalData = 0x02,
    RssiCalibration = 0x04,
}

impl DeviceCapability {
    pub fn serialize(capabilities: &[DeviceCapability]) -> u8 {
        capabilities.iter().fold(0u8, |acc, cap| acc | (*cap as u8))
    }

    pub fn deserialize(byte: u8) -> Vec<DeviceCapability, 3> {
        let mut capabilities = Vec::new();
        if byte & (DeviceCapability::UnlockGate as u8) != 0 {
            capabilities.push(DeviceCapability::UnlockGate).unwrap();
        }
        if byte & (DeviceCapability::EnvironmentalData as u8) != 0 {
            capabilities
                .push(DeviceCapability::EnvironmentalData)
                .unwrap();
        }
        if byte & (DeviceCapability::RssiCalibration as u8) != 0 {
            capabilities
                .push(DeviceCapability::RssiCalibration)
                .unwrap();
        }
        capabilities
    }
}

#[derive(Debug, Clone, Copy)]
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

    pub fn deserialize(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::Merchant),
            0x02 => Some(Self::Pathway),
            0x03 => Some(Self::Connection),
            0x04 => Some(Self::Turnstile),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CryptoError {
    InvalidSignature,
    InvalidKey,
    InvalidNonce,
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
            Self::InvalidNonce => 0x03,
            Self::VerificationFailed => 0x04,
            Self::BufferFull => 0x05,
            Self::RateLimited => 0x06,
            Self::ReplayDetected => 0x07,
            Self::ServerPublicKeyNotSet => 0x08,
        }
    }

    pub fn deserialize(code: u8) -> Option<Self> {
        match code {
            0x01 => Some(Self::InvalidSignature),
            0x02 => Some(Self::InvalidKey),
            0x03 => Some(Self::InvalidNonce),
            0x04 => Some(Self::VerificationFailed),
            0x05 => Some(Self::BufferFull),
            0x06 => Some(Self::RateLimited),
            0x07 => Some(Self::ReplayDetected),
            0x08 => Some(Self::ServerPublicKeyNotSet),
            _ => None,
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
