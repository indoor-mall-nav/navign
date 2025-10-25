use crate::Packetize;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_big_array::BigArray;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Proof {
    pub nonce: [u8; 16],
    pub device_bytes: [u8; 8],
    pub verify_bytes: [u8; 8],
    pub timestamp: u64,
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
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
        Proof {
            nonce,
            device_bytes,
            verify_bytes,
            timestamp,
            server_signature,
        }
    }
}

#[cfg(feature = "heapless")]
impl Packetize<104> for Proof {
    fn packetize(&self) -> heapless::Vec<u8, 104> {
        let mut vec = heapless::Vec::<u8, 104>::new();
        vec.extend_from_slice(&self.nonce).unwrap();
        vec.extend_from_slice(&self.device_bytes).unwrap();
        vec.extend_from_slice(&self.verify_bytes).unwrap();
        vec.extend_from_slice(&self.timestamp.to_be_bytes())
            .unwrap();
        vec.extend_from_slice(&self.server_signature).unwrap();
        vec
    }
}

#[cfg(feature = "alloc")]
impl Packetize for Proof {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        let mut vec = alloc::vec::Vec::with_capacity(16 + 8 + 8 + 8 + 64);
        vec.extend_from_slice(&self.nonce);
        vec.extend_from_slice(&self.device_bytes);
        vec.extend_from_slice(&self.verify_bytes);
        vec.extend_from_slice(&self.timestamp.to_be_bytes());
        vec.extend_from_slice(&self.server_signature);
        vec
    }
}
