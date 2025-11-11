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

#[cfg(feature = "alloc")]
impl Packetize for ServerChallenge {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        let mut vec = alloc::vec::Vec::with_capacity(16 + 24 + 8 + 24);
        vec.extend_from_slice(&self.nonce);
        vec.extend_from_slice(&self.instance_id);
        vec.extend_from_slice(&self.timestamp.to_be_bytes());
        vec.extend_from_slice(&self.user_id);
        vec
    }
}

#[cfg(feature = "heapless")]
impl Packetize<72> for ServerChallenge {
    fn packetize(&self) -> heapless::Vec<u8, 72> {
        let mut vec = heapless::Vec::<u8, 72>::new();
        vec.extend_from_slice(&self.nonce).unwrap();
        vec.extend_from_slice(&self.instance_id).unwrap();
        vec.extend_from_slice(&self.timestamp.to_be_bytes())
            .unwrap();
        vec.extend_from_slice(&self.user_id).unwrap();
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_server_challenge_packetize() {
        let challenge = ServerChallenge::new([0u8; 16], [1u8; 24], 1234567890, [2u8; 24]);
        #[cfg(feature = "heapless")]
        {
            let packet = challenge.packetize();
            assert_eq!(packet.len(), 72);
            assert_eq!(&packet[0..16], &[0u8; 16]);
            assert_eq!(&packet[16..40], &[1u8; 24]);
            assert_eq!(&packet[40..48], &1234567890u64.to_be_bytes());
            assert_eq!(&packet[48..72], &[2u8; 24]);
        }
        #[cfg(feature = "alloc")]
        {
            let packet = challenge.packetize();
            assert_eq!(packet.len(), 72);
            assert_eq!(&packet[0..16], &[0u8; 16]);
            assert_eq!(&packet[16..40], &[1u8; 24]);
            assert_eq!(&packet[40..48], &1234567890u64.to_be_bytes());
            assert_eq!(&packet[48..72], &[2u8; 24]);
        }
    }
}
