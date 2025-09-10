use super::super::crypto::{Nonce, Proof};
use super::{BleError, BleMessage};
use heapless::Vec;

const MAX_PACKET_SIZE: usize = 512;

pub struct BleProtocolHandler {
    receive_buffer: Vec<u8, MAX_PACKET_SIZE>,
    send_buffer: Vec<u8, MAX_PACKET_SIZE>,
}

impl BleProtocolHandler {
    pub fn new() -> Self {
        Self {
            receive_buffer: Vec::new(),
            send_buffer: Vec::new(),
        }
    }

    pub fn serialize_message(&mut self, message: &BleMessage) -> Result<&[u8], BleError> {
        self.send_buffer.clear();

        match message {
            BleMessage::NonceRequest => {
                self.send_buffer
                    .push(0x01)
                    .map_err(|_| BleError::BufferFull)?;
            }

            BleMessage::NonceResponse(nonce) => {
                self.send_buffer
                    .push(0x02)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer
                    .extend_from_slice(nonce.as_bytes())
                    .map_err(|_| BleError::BufferFull)?;
            }

            BleMessage::ProofSubmission(proof) => {
                self.send_buffer
                    .push(0x03)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer
                    .extend_from_slice(&proof.challenge_hash)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer
                    .extend_from_slice(&proof.device_signature)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer
                    .extend_from_slice(&proof.timestamp.to_be_bytes())
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer
                    .extend_from_slice(&proof.counter.to_be_bytes())
                    .map_err(|_| BleError::BufferFull)?;
            }

            BleMessage::UnlockResult(success) => {
                self.send_buffer
                    .push(0x04)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer
                    .push(if *success { 0x01 } else { 0x00 })
                    .map_err(|_| BleError::BufferFull)?;
            }
        }

        Ok(&self.send_buffer)
    }

    pub fn deserialize_message(&mut self, data: &[u8]) -> Result<BleMessage, BleError> {
        self.receive_buffer.clear();
        self.receive_buffer
            .extend_from_slice(data)
            .map_err(|_| BleError::BufferFull)?;

        if self.receive_buffer.is_empty() {
            return Err(BleError::ParseError);
        }

        match self.receive_buffer[0] {
            0x01 => Ok(BleMessage::NonceRequest),

            0x02 => {
                if self.receive_buffer.len() != 1 + 16 {
                    return Err(BleError::ParseError);
                }
                let mut nonce_bytes = [0u8; 16];
                nonce_bytes.copy_from_slice(&self.receive_buffer[1..17]);
                Ok(BleMessage::NonceResponse(Nonce::from_bytes(&nonce_bytes)))
            }

            0x03 => {
                if self.receive_buffer.len() != 1 + 32 + 64 + 8 + 8 {
                    return Err(BleError::ParseError);
                }
                let mut challenge_hash = [0u8; 32];
                challenge_hash.copy_from_slice(&self.receive_buffer[1..33]);
                let mut device_signature = [0u8; 64];
                device_signature.copy_from_slice(&self.receive_buffer[33..97]);
                let timestamp =
                    u64::from_be_bytes(self.receive_buffer[97..105].try_into().unwrap());
                let counter = u64::from_be_bytes(self.receive_buffer[105..113].try_into().unwrap());
                Ok(BleMessage::ProofSubmission(Proof {
                    challenge_hash,
                    device_signature,
                    timestamp,
                    counter,
                }))
            }

            0x04 => {
                if self.receive_buffer.len() != 2 {
                    return Err(BleError::ParseError);
                }
                let success = match self.receive_buffer[1] {
                    0x00 => false,
                    0x01 => true,
                    _ => return Err(BleError::ParseError),
                };
                Ok(BleMessage::UnlockResult(success))
            }

            _ => Err(BleError::ParseError),
        }
    }
}
