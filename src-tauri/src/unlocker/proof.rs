use crate::unlocker::constants::{CHALLENGE_HASH_LENGTH, DEVICE_BYTES_LENGTH, NONCE_LENGTH, SERVER_SIGNATURE_LENGTH, TIMESTAMP_LENGTH, UNLOCK_REQUEST_LENGTH, VERIFY_BYTES_LENGTH};
use p256::ecdsa::SigningKey;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub nonce: [u8; 16],
    pub device_bytes: [u8; 8],
    pub verify_bytes: [u8; 8],
    pub timestamp: u64,
    #[serde(with = "BigArray")]
    pub server_signature: [u8; 64],
}

impl Proof {
    pub fn new(
        nonce: [u8; 16],
        device_bytes: [u8; 8],
        verify_bytes: [u8; 8],
        timestamp: u64,
        server_signature: [u8; 64],
    ) -> Self {
        Self {
            nonce,
            device_bytes,
            verify_bytes,
            timestamp,
            server_signature,
        }
    }
    
    pub fn packetize(&self) -> Vec<u8> {
        let mut packet = Vec::with_capacity(16 + 8 + 8 + 8 + 64);
        packet.extend_from_slice(&self.nonce);
        packet.extend_from_slice(&self.device_bytes);
        packet.extend_from_slice(&self.verify_bytes);
        packet.extend_from_slice(&self.timestamp.to_be_bytes());
        packet.extend_from_slice(&self.server_signature);
        packet
    }

    pub fn depacketize(data: &[u8]) -> Option<Self> {
        if data.len() != UNLOCK_REQUEST_LENGTH - 1 {
            return None;
        }
        let device_bytes_offset = NONCE_LENGTH;
        let verify_bytes_offset = device_bytes_offset + DEVICE_BYTES_LENGTH;
        let timestamp_offset = verify_bytes_offset + VERIFY_BYTES_LENGTH;
        let server_signature_offset = timestamp_offset + TIMESTAMP_LENGTH;
        let mut nonce = [0u8; NONCE_LENGTH];
        nonce.copy_from_slice(&data[0..device_bytes_offset]);
        let mut device_bytes = [0u8; 8];
        device_bytes.copy_from_slice(&data[device_bytes_offset..verify_bytes_offset]);
        let mut verify_bytes = [0u8; 8];
        verify_bytes.copy_from_slice(&data[verify_bytes_offset..timestamp_offset]);
        let timestamp = u64::from_be_bytes(data[timestamp_offset..server_signature_offset].try_into().ok()?);
        let mut server_signature = [0u8; 64];
        server_signature.copy_from_slice(
            &data[server_signature_offset..server_signature_offset + SERVER_SIGNATURE_LENGTH],
        );
        Some(Self {
            nonce,
            device_bytes,
            verify_bytes,
            timestamp,
            server_signature,
        })
    }
}
