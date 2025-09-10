#[derive(Debug)]
pub enum CryptoError {
    InvalidSignature,
    VerificationFailed,
    BufferFull,
    InvalidKey,
    ServerPublicKeyNotSet,
}

impl CryptoError {
    pub fn message(&self) -> &str {
        match self {
            CryptoError::InvalidSignature => "Invalid signature",
            CryptoError::VerificationFailed => "Verification failed",
            CryptoError::BufferFull => "Buffer is full",
            CryptoError::InvalidKey => "Invalid key",
            CryptoError::ServerPublicKeyNotSet => "Server public key not set",
        }
    }

    pub fn serialize(&self) -> u8 {
        match self {
            CryptoError::InvalidSignature => 0x01,
            CryptoError::VerificationFailed => 0x02,
            CryptoError::BufferFull => 0x03,
            CryptoError::InvalidKey => 0x04,
            CryptoError::ServerPublicKeyNotSet => 0x05,
        }
    }
}