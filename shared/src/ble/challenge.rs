#[cfg(feature = "postcard")]
use crate::Packetize;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ServerChallenge {
    pub nonce: [u8; 16],
    pub instance_id: [u8; 24],
    pub timestamp: u64,
    pub user_id: [u8; 24],
}

impl ServerChallenge {
    pub fn new(nonce: [u8; 16], instance_id: [u8; 24], timestamp: u64, user_id: [u8; 24]) -> Self {
        ServerChallenge {
            nonce,
            instance_id,
            timestamp,
            user_id,
        }
    }
}

#[cfg(all(feature = "alloc", feature = "postcard"))]
impl Packetize for ServerChallenge {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        postcard::to_allocvec(self).unwrap()
    }
}

#[cfg(all(feature = "heapless", feature = "postcard"))]
impl Packetize<128> for ServerChallenge {
    fn packetize(&self) -> heapless::Vec<u8, 128> {
        let mut buf = [0u8; 128];
        let used = postcard::to_slice(self, &mut buf).unwrap();
        let mut vec = heapless::Vec::<u8, 128>::new();
        vec.extend_from_slice(used).unwrap();
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "postcard")]
    fn test_server_challenge_packetize() {
        let challenge = ServerChallenge::new([0u8; 16], [1u8; 24], 1234567890, [2u8; 24]);
        #[cfg(feature = "heapless")]
        {
            let packet = challenge.packetize();
            // Postcard format is more compact than manual serialization
            assert!(packet.len() > 0);
            assert!(packet.len() <= 128); // Within buffer size
        }
        #[cfg(feature = "alloc")]
        {
            let packet = challenge.packetize();
            // Postcard format is more compact than manual serialization
            assert!(packet.len() > 0);
        }
    }
}
