pub mod constants;

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
