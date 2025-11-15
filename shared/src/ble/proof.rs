#[cfg(feature = "postcard")]
use crate::{Depacketize, Packetize};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_big_array::BigArray;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

#[cfg(all(feature = "heapless", feature = "postcard"))]
impl Packetize<128> for Proof {
    fn packetize(&self) -> heapless::Vec<u8, 128> {
        let mut buf = [0u8; 128];
        let used = postcard::to_slice(self, &mut buf).unwrap();
        let mut vec = heapless::Vec::<u8, 128>::new();
        vec.extend_from_slice(used).unwrap();
        vec
    }
}

#[cfg(all(feature = "alloc", feature = "postcard"))]
impl Packetize for Proof {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        postcard::to_allocvec(self).unwrap()
    }
}

#[cfg(feature = "postcard")]
impl Depacketize for Proof {
    fn depacketize(data: &[u8]) -> Option<Self> {
        postcard::from_bytes(data).ok()
    }
}
