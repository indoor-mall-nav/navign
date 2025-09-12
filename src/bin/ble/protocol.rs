use super::super::crypto::{Nonce, Proof};
use crate::shared::constants::*;
pub(crate) use super::BleMessage;
use crate::shared::{BleError, CryptoError};
use heapless::Vec;

#[derive(Debug)]
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
            BleMessage::DeviceInquiry => {
                self.send_buffer
                    .push(DEVICE_INQUIRY)
                    .map_err(|_| BleError::BufferFull)?;
            }

            BleMessage::DeviceResponse(object_id) => {
                self.send_buffer
                    .push(DEVICE_RESPONSE)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer
                    .extend_from_slice(object_id)
                    .map_err(|_| BleError::BufferFull)?;
            }

            BleMessage::NonceRequest => {
                self.send_buffer
                    .push(NONCE_REQUEST)
                    .map_err(|_| BleError::BufferFull)?;
            }

            BleMessage::NonceResponse(nonce) => {
                self.send_buffer
                    .push(NONCE_RESPONSE)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer
                    .extend_from_slice(nonce.as_bytes())
                    .map_err(|_| BleError::BufferFull)?;
            }

            BleMessage::ProofSubmission(proof) => {
                self.send_buffer
                    .push(PROOF_SUBMISSION)
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

            BleMessage::UnlockResult(success, reason) => {
                self.send_buffer
                    .push(UNLOCK_RESULT)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer
                    .push(if *success {
                        UNLOCK_SUCCESS
                    } else {
                        UNLOCK_FAILURE
                    })
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer
                    .push(reason.map(|x| x.serialize()).unwrap_or(0x00))
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
            DEVICE_INQUIRY => Ok(BleMessage::DeviceInquiry),

            DEVICE_RESPONSE => {
                if self.receive_buffer.len() != DEVICE_RESPONSE_LENGTH {
                    return Err(BleError::ParseError);
                }
                let mut object_id = [0u8; DEVICE_LENGTH];
                object_id.copy_from_slice(
                    &self.receive_buffer[IDENTIFIER_LENGTH..IDENTIFIER_LENGTH + DEVICE_LENGTH],
                );
                Ok(BleMessage::DeviceResponse(object_id))
            }

            NONCE_REQUEST => Ok(BleMessage::NonceRequest),

            NONCE_RESPONSE => {
                if self.receive_buffer.len() != NONCE_RESPONSE_LENGTH {
                    return Err(BleError::ParseError);
                }
                let mut nonce_bytes = [0u8; NONCE_LENGTH];
                nonce_bytes.copy_from_slice(
                    &self.receive_buffer[IDENTIFIER_LENGTH..IDENTIFIER_LENGTH + NONCE_LENGTH],
                );
                Ok(BleMessage::NonceResponse(Nonce::from_bytes(&nonce_bytes)))
            }

            PROOF_SUBMISSION => {
                if self.receive_buffer.len() != PROOF_SUBMISSION_LENGTH {
                    return Err(BleError::ParseError);
                }
                let mut challenge_hash = [0u8; CHALLENGE_HASH_LENGTH];
                challenge_hash.copy_from_slice(
                    &self.receive_buffer
                        [CHALLENGE_HASH_OFFSET..CHALLENGE_HASH_OFFSET + CHALLENGE_HASH_LENGTH],
                );
                let mut device_signature = [0u8; DEVICE_SIGNATURE_LENGTH];
                device_signature.copy_from_slice(
                    &self.receive_buffer[DEVICE_SIGNATURE_OFFSET
                        ..DEVICE_SIGNATURE_OFFSET + DEVICE_SIGNATURE_LENGTH],
                );
                let timestamp = u64::from_be_bytes(
                    self.receive_buffer[TIMESTAMP_OFFSET..TIMESTAMP_OFFSET + TIMESTAMP_LENGTH]
                        .try_into()
                        .unwrap(),
                );
                let counter = u64::from_be_bytes(
                    self.receive_buffer[COUNTER_OFFSET..COUNTER_OFFSET + COUNTER_LENGTH]
                        .try_into()
                        .unwrap(),
                );
                Ok(BleMessage::ProofSubmission(Proof {
                    challenge_hash,
                    device_signature,
                    timestamp,
                    counter,
                }))
            }

            UNLOCK_RESULT => {
                if self.receive_buffer.len() != UNLOCK_RESULT_LENGTH {
                    return Err(BleError::ParseError);
                }
                let success = match self.receive_buffer[1] {
                    UNLOCK_FAILURE => false,
                    UNLOCK_SUCCESS => true,
                    _ => return Err(BleError::ParseError),
                };
                let reason = if self.receive_buffer[2] == 0x00 {
                    None
                } else {
                    CryptoError::deserialize(self.receive_buffer[2])
                };
                Ok(BleMessage::UnlockResult(success, reason))
            }

            _ => Err(BleError::ParseError),
        }
    }
}
