use crate::{Depacketize, Packetize};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl core::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryptoError::InvalidSignature => write!(f, "Invalid Signature"),
            CryptoError::InvalidKey => write!(f, "Invalid Key"),
            CryptoError::InvalidNonce => write!(f, "Invalid Nonce"),
            CryptoError::VerificationFailed => write!(f, "Verification Failed"),
            CryptoError::BufferFull => write!(f, "Buffer Full"),
            CryptoError::RateLimited => write!(f, "Rate Limited"),
            CryptoError::ReplayDetected => write!(f, "Replay Detected"),
            CryptoError::ServerPublicKeyNotSet => write!(f, "Server Public Key Not Set"),
        }
    }
}

impl From<CryptoError> for u8 {
    fn from(error: CryptoError) -> Self {
        match error {
            CryptoError::InvalidSignature => 0x01,
            CryptoError::InvalidKey => 0x02,
            CryptoError::InvalidNonce => 0x03,
            CryptoError::VerificationFailed => 0x04,
            CryptoError::BufferFull => 0x05,
            CryptoError::RateLimited => 0x06,
            CryptoError::ReplayDetected => 0x07,
            CryptoError::ServerPublicKeyNotSet => 0x08,
        }
    }
}

#[cfg(feature = "heapless")]
impl Packetize<1> for CryptoError {
    fn packetize(&self) -> heapless::Vec<u8, 1> {
        let mut vec = heapless::Vec::<u8, 1>::new();
        vec.push((*self).into()).unwrap();
        vec
    }
}

#[cfg(feature = "alloc")]
impl Packetize for CryptoError {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        alloc::vec![(*self).into()]
    }
}

impl Depacketize for CryptoError {
    fn depacketize(packet: &[u8]) -> Option<Self> {
        if packet.len() != 1 {
            return None;
        }
        match packet[0] {
            0x01 => Some(CryptoError::InvalidSignature),
            0x02 => Some(CryptoError::InvalidKey),
            0x03 => Some(CryptoError::InvalidNonce),
            0x04 => Some(CryptoError::VerificationFailed),
            0x05 => Some(CryptoError::BufferFull),
            0x06 => Some(CryptoError::RateLimited),
            0x07 => Some(CryptoError::ReplayDetected),
            0x08 => Some(CryptoError::ServerPublicKeyNotSet),
            _ => None,
        }
    }
}
