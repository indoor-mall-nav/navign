use crate::{Depacketize, Packetize};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Nonce([u8; 16]);

impl Nonce {
    pub fn new(bytes: [u8; 16]) -> Self {
        Nonce(bytes)
    }
}

impl AsRef<[u8]> for Nonce {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::ops::Deref for Nonce {
    type Target = [u8; 16];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Debug for Nonce {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Nonce(")?;
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        write!(f, ")")
    }
}

#[cfg(feature = "alloc")]
impl Packetize for Nonce {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        self.0.to_vec()
    }
}

#[cfg(feature = "heapless")]
impl Packetize<16> for Nonce {
    fn packetize(&self) -> heapless::Vec<u8, 16> {
        let mut vec = heapless::Vec::<u8, 16>::new();
        vec.extend_from_slice(&self.0).unwrap();
        vec
    }
}

impl Depacketize for Nonce {
    fn depacketize(packet: &[u8]) -> Option<Self> {
        if packet.len() != 16 {
            return None;
        }
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&packet[0..16]);
        Some(Nonce(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nonce_packetize_depacketize() {
        let nonce = Nonce::new([1u8; 16]);
        #[cfg(feature = "heapless")]
        {
            let packet = nonce.packetize();
            assert_eq!(packet.len(), 16);
            let depacketized = Nonce::depacketize(&packet).unwrap();
            assert_eq!(nonce, depacketized);
        }
        #[cfg(feature = "alloc")]
        {
            let packet = nonce.packetize();
            assert_eq!(packet.len(), 16);
            let depacketized = Nonce::depacketize(&packet).unwrap();
            assert_eq!(nonce, depacketized);
        }
    }
}
