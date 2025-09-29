use super::super::crypto::{Nonce, Proof};
pub(crate) use super::BleMessage;
use crate::shared::constants::*;
use crate::shared::{BleError, CryptoError, DeviceCapability, DeviceType};
use esp_println::println;
use heapless::Vec;

#[derive(Debug)]
pub struct BleProtocolHandler {
    send_buffer: Vec<u8, MAX_PACKET_SIZE>,
    receive_buffer: Vec<u8, MAX_PACKET_SIZE>,
    send_buffer_length: usize,
    receive_buffer_length: usize,
    pub processing: bool,
}

fn expect_length(data: &[u8]) -> bool {
    println!("Expecting length for data: {:?}", data);
    match data.first().copied().unwrap_or(0x00) {
        DEVICE_REQUEST => data.len() == DEVICE_REQUEST_LENGTH,
        DEVICE_RESPONSE => data.len() == DEVICE_RESPONSE_LENGTH,
        NONCE_REQUEST => data.len() == NONCE_REQUEST_LENGTH,
        NONCE_RESPONSE => data.len() == NONCE_RESPONSE_LENGTH,
        UNLOCK_REQUEST => data.len() == UNLOCK_REQUEST_LENGTH,
        UNLOCK_RESPONSE => data.len() == UNLOCK_RESPONSE_LENGTH,
        _ => false,
    }
}

impl BleProtocolHandler {
    pub fn new() -> Self {
        Self {
            send_buffer: Vec::<u8, MAX_PACKET_SIZE>::new(),
            receive_buffer: Vec::<u8, MAX_PACKET_SIZE>::new(),
            send_buffer_length: 0,
            receive_buffer_length: 0,
            processing: false,
        }
    }

    pub fn has_message(&self) -> bool {
        if self.receive_buffer.is_empty() {
            return false;
        }
        println!("Checking message with length {}", self.receive_buffer.len());
        expect_length(&self.receive_buffer)
    }

    pub fn store_message(&mut self, data: &[u8], offset: usize) -> Result<(), BleError> {
        if offset == 0 {
            self.receive_buffer.clear();
        }
        if data.is_empty() {
            return Ok(());
        }
        self.processing = false;
        self.receive_buffer
            .extend_from_slice(data)
            .map_err(|_| BleError::BufferFull)?;
        Ok(())
    }

    pub fn extract_message(&mut self, offset: usize) -> [u8; MAX_PACKET_SIZE] {
        let mut output = [0u8; MAX_PACKET_SIZE];
        let terminal = if self.send_buffer_length > offset {
            self.send_buffer_length - offset
        } else {
            return output;
        };
        output[..terminal].copy_from_slice(&self.send_buffer[offset..self.send_buffer_length]);
        // Remove those data from the vec
        if terminal > 0 {
            self.send_buffer.drain(0..terminal);
            self.send_buffer_length -= terminal;
        }
        output
    }

    pub fn serialize_message(
        &mut self,
        message: &BleMessage,
    ) -> Result<[u8; MAX_PACKET_SIZE], BleError> {
        self.send_buffer.clear();

        let buffer = &mut self.send_buffer;

        println!("The buffer is {:?}", buffer);

        match message {
            BleMessage::DeviceRequest(payload) => {
                buffer
                    .push(DEVICE_REQUEST)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer_length = DEVICE_REQUEST_LENGTH;
                buffer.push(*payload).map_err(|_| BleError::BufferFull)?;
            }

            BleMessage::DeviceResponse(device_type, capabilities, object_id) => {
                buffer
                    .push(DEVICE_RESPONSE)
                    .map_err(|_| BleError::BufferFull)?;
                buffer
                    .push(device_type.serialize())
                    .map_err(|_| BleError::BufferFull)?;
                buffer
                    .push(DeviceCapability::serialize(&capabilities))
                    .map_err(|_| BleError::BufferFull)?;
                buffer
                    .extend_from_slice(object_id)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer_length = DEVICE_RESPONSE_LENGTH;
                println!("The buffer is {:?} for device response.", buffer);
            }

            BleMessage::NonceRequest => {
                buffer
                    .push(NONCE_REQUEST)
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer_length = NONCE_REQUEST_LENGTH;
            }

            BleMessage::NonceResponse(nonce) => {
                buffer
                    .push(NONCE_RESPONSE)
                    .map_err(|_| BleError::BufferFull)?;
                buffer
                    .extend_from_slice(nonce.as_bytes())
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer_length = NONCE_RESPONSE_LENGTH;
            }

            BleMessage::UnlockRequest(proof) => {
                buffer
                    .push(UNLOCK_REQUEST)
                    .map_err(|_| BleError::BufferFull)?;
                buffer
                    .extend_from_slice(&proof.challenge_hash)
                    .map_err(|_| BleError::BufferFull)?;
                buffer
                    .extend_from_slice(&proof.device_signature)
                    .map_err(|_| BleError::BufferFull)?;
                buffer
                    .extend_from_slice(&proof.timestamp.to_be_bytes())
                    .map_err(|_| BleError::BufferFull)?;
                buffer
                    .extend_from_slice(&proof.counter.to_be_bytes())
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer_length = UNLOCK_REQUEST_LENGTH;
            }

            BleMessage::UnlockResponse(success, reason) => {
                buffer
                    .push(UNLOCK_RESPONSE)
                    .map_err(|_| BleError::BufferFull)?;
                buffer
                    .push(if *success {
                        UNLOCK_SUCCESS
                    } else {
                        UNLOCK_FAILURE
                    })
                    .map_err(|_| BleError::BufferFull)?;
                buffer
                    .push(reason.map(|x| x.serialize()).unwrap_or(0x00))
                    .map_err(|_| BleError::BufferFull)?;
                self.send_buffer_length = UNLOCK_RESPONSE_LENGTH;
            }
        }

        let mut output = [0u8; MAX_PACKET_SIZE];
        output[..buffer.len()].copy_from_slice(buffer);
        println!("The buffer is {:?}", output);
        Ok(output)
    }

    pub fn clear_receive_buffer(&mut self) {
        self.receive_buffer.clear();
    }

    pub fn clear_send_buffer(&mut self) {
        self.send_buffer.clear();
    }

    pub fn deserialize_message(&mut self, data: Option<&[u8]>) -> Result<BleMessage, BleError> {
        if data.is_some() {
            self.receive_buffer.clear();
            self.receive_buffer
                .extend_from_slice(data.unwrap())
                .map_err(|_| BleError::BufferFull)?;
        }

        if self.receive_buffer.is_empty() {
            return Err(BleError::ParseError);
        }

        let result = match self.receive_buffer[0] {
            DEVICE_REQUEST => Ok(BleMessage::DeviceRequest(self.receive_buffer[1])),

            DEVICE_RESPONSE => {
                if self.receive_buffer.len() != DEVICE_RESPONSE_LENGTH {
                    return Err(BleError::ParseError);
                }
                let device_type_byte =
                    &self.receive_buffer[IDENTIFIER_LENGTH..IDENTIFIER_LENGTH + DEVICE_TYPE_LENGTH];
                let device_type =
                    DeviceType::deserialize(device_type_byte[0]).ok_or(BleError::ParseError)?;
                let capability_byte = &self.receive_buffer
                    [DEVICE_CAPABILITY_OFFSET..DEVICE_CAPABILITY_OFFSET + DEVICE_CAPABILITY_LENGTH];
                let capabilities = DeviceCapability::deserialize(capability_byte[0]);
                let mut object_id = [0u8; DEVICE_ID_LENGTH];
                object_id.copy_from_slice(
                    &self.receive_buffer[DEVICE_ID_OFFSET..DEVICE_ID_OFFSET + DEVICE_ID_LENGTH],
                );
                Ok(BleMessage::DeviceResponse(
                    device_type,
                    capabilities,
                    object_id,
                ))
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

            UNLOCK_REQUEST => {
                if self.receive_buffer.len() != UNLOCK_REQUEST_LENGTH {
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
                Ok(BleMessage::UnlockRequest(Proof {
                    challenge_hash,
                    device_signature,
                    timestamp,
                    counter,
                }))
            }

            UNLOCK_RESPONSE => {
                if self.receive_buffer.len() != UNLOCK_RESPONSE_LENGTH {
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
                Ok(BleMessage::UnlockResponse(success, reason))
            }

            _ => Err(BleError::ParseError),
        };
        self.clear_receive_buffer();
        result
    }
}
