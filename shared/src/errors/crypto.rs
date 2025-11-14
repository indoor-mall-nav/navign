use crate::{Depacketize, Packetize};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

#[cfg(all(feature = "heapless", feature = "postcard"))]
impl Packetize<8> for CryptoError {
    fn packetize(&self) -> heapless::Vec<u8, 8> {
        let mut buf = [0u8; 8];
        let used = postcard::to_slice(self, &mut buf).unwrap();
        let mut vec = heapless::Vec::<u8, 8>::new();
        vec.extend_from_slice(used).unwrap();
        vec
    }
}

#[cfg(all(feature = "alloc", feature = "postcard"))]
impl Packetize for CryptoError {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        postcard::to_allocvec(self).unwrap()
    }
}

#[cfg(feature = "postcard")]
impl Depacketize for CryptoError {
    fn depacketize(packet: &[u8]) -> Option<Self> {
        postcard::from_bytes(packet).ok()
    }
}
