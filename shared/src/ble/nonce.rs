#[cfg(feature = "postcard")]
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

#[cfg(feature = "defmt")]
impl defmt::Format for Nonce {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Nonce({:02x})", self.0);
    }
}

#[cfg(all(feature = "alloc", feature = "postcard"))]
impl Packetize for Nonce {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        postcard::to_allocvec(self).unwrap()
    }
}

#[cfg(all(feature = "heapless", feature = "postcard"))]
impl Packetize<32> for Nonce {
    fn packetize(&self) -> heapless::Vec<u8, 32> {
        let mut buf = [0u8; 32];
        let used = postcard::to_slice(self, &mut buf).unwrap();
        let mut vec = heapless::Vec::<u8, 32>::new();
        vec.extend_from_slice(used).unwrap();
        vec
    }

    fn try_packetize(&self) -> Result<heapless::Vec<u8, 32>, crate::PacketizeError> {
        let mut buf = [0u8; 32];
        let used = postcard::to_slice(self, &mut buf)
            .map_err(|_| crate::PacketizeError::BufferOverflow)?;
        let mut vec = heapless::Vec::<u8, 32>::new();
        vec.extend_from_slice(used)
            .map_err(|_| crate::PacketizeError::BufferOverflow)?;
        Ok(vec)
    }
}

#[cfg(feature = "postcard")]
impl Depacketize for Nonce {
    fn depacketize(packet: &[u8]) -> Option<Self> {
        postcard::from_bytes(packet).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "postcard")]
    fn test_nonce_packetize_depacketize() {
        let nonce = Nonce::new([1u8; 16]);
        #[cfg(feature = "heapless")]
        {
            let packet = nonce.packetize();
            assert!(packet.len() > 0);
            let depacketized = Nonce::depacketize(&packet).unwrap();
            assert_eq!(nonce, depacketized);
        }
        #[cfg(feature = "alloc")]
        {
            let packet = nonce.packetize();
            assert!(packet.len() > 0);
            let depacketized = Nonce::depacketize(&packet).unwrap();
            assert_eq!(nonce, depacketized);
        }
    }
}
