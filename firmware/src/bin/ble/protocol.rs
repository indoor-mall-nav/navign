use crate::shared::BleError;
use crate::shared::constants::*;
use esp_println::println;
use heapless::Vec;
use navign_shared::{BleMessage, Depacketize, Packetize};

#[derive(Debug)]
pub struct BleProtocolHandler {
    send_buffer: Vec<u8, MAX_PACKET_SIZE>,
    receive_buffer: Vec<u8, MAX_PACKET_SIZE>,
    send_buffer_length: usize,
    #[allow(unused)]
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
        DEBUG_REQUEST => data.len() >= IDENTIFIER_LENGTH,
        DEBUG_RESPONSE => data.len() >= IDENTIFIER_LENGTH,
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
        let offset_parts = (offset + 1) / 128;
        let offset = offset_parts * 125 + 1;
        let terminal = if self.send_buffer_length > offset {
            self.send_buffer_length - offset
        } else {
            return output;
        };
        let max_terminal = if terminal > 125 { 125 } else { terminal };
        let max_terminal = if max_terminal + offset > self.send_buffer_length {
            self.send_buffer_length - offset
        } else {
            max_terminal
        };
        let max_terminal = if max_terminal > self.send_buffer.len() {
            self.send_buffer.len() - offset
        } else {
            max_terminal
        };
        output[..terminal].copy_from_slice(&self.send_buffer[offset..offset + max_terminal]);
        output
    }

    pub fn serialize_message(
        &mut self,
        message: &BleMessage,
    ) -> Result<[u8; MAX_PACKET_SIZE], BleError> {
        self.send_buffer.clear();

        let buffer = &mut self.send_buffer;

        println!("The buffer is {:?}", buffer);

        let msg = message.packetize();
        let result = msg.as_slice();

        let mut output = [0u8; MAX_PACKET_SIZE];
        output[..buffer.len()].copy_from_slice(result);
        println!("The buffer is {:?}", output);
        Ok(output)
    }

    pub fn clear_receive_buffer(&mut self) {
        self.receive_buffer.clear();
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

        let result =
            BleMessage::depacketize(self.receive_buffer.as_slice()).ok_or(BleError::ParseError)?;

        self.clear_receive_buffer();

        Ok(result)
    }
}
