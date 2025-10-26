use crate::constants::*;
use crate::{DeviceCapabilities, DeviceTypes, Packetize, Proof, errors::CryptoError};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BleMessage {
    DeviceRequest,
    DeviceResponse(DeviceTypes, DeviceCapabilities, [u8; 24]), // 24-byte MongoDB ObjectId segment
    NonceRequest,
    NonceResponse([u8; 16], [u8; 8]),
    UnlockRequest(Proof),
    UnlockResponse(bool, CryptoError),
}

impl TryFrom<u8> for BleMessage {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(BleMessage::DeviceRequest),
            0x03 => Ok(BleMessage::NonceRequest),
            _ => Err(()),
        }
    }
}

impl From<(DeviceTypes, DeviceCapabilities, [u8; 24])> for BleMessage {
    fn from(value: (DeviceTypes, DeviceCapabilities, [u8; 24])) -> Self {
        BleMessage::DeviceResponse(value.0, value.1, value.2)
    }
}

impl From<([u8; 16], [u8; 8])> for BleMessage {
    fn from(value: ([u8; 16], [u8; 8])) -> Self {
        BleMessage::NonceResponse(value.0, value.1)
    }
}

impl From<Proof> for BleMessage {
    fn from(value: Proof) -> Self {
        BleMessage::UnlockRequest(value)
    }
}

#[cfg(feature = "heapless")]
impl Packetize<128> for BleMessage {
    fn packetize(&self) -> heapless::Vec<u8, 128> {
        let mut vec = heapless::Vec::<u8, 128>::new();
        match self {
            BleMessage::DeviceRequest => {
                vec.push(DEVICE_REQUEST).unwrap();
            }
            BleMessage::DeviceResponse(device_types, device_capabilities, object_id_segment) => {
                vec.push(DEVICE_RESPONSE).unwrap();
                vec.extend_from_slice(&device_types.packetize()).unwrap();
                vec.extend_from_slice(&device_capabilities.packetize())
                    .unwrap();
                vec.extend_from_slice(object_id_segment).unwrap();
            }
            BleMessage::NonceRequest => {
                vec.push(NONCE_REQUEST).unwrap();
            }
            BleMessage::NonceResponse(nonce, verify_bytes) => {
                vec.push(NONCE_RESPONSE).unwrap();
                vec.extend_from_slice(nonce).unwrap();
                vec.extend_from_slice(verify_bytes).unwrap();
            }
            BleMessage::UnlockRequest(proof) => {
                vec.push(UNLOCK_REQUEST).unwrap();
                let proof_packet = proof.packetize();
                vec.extend_from_slice(&proof_packet).unwrap();
            }
            BleMessage::UnlockResponse(success, error) => {
                vec.push(UNLOCK_RESPONSE).unwrap();
                vec.push(if *success {
                    UNLOCK_SUCCESS
                } else {
                    UNLOCK_FAILURE
                })
                .unwrap();
                vec.extend_from_slice(&error.packetize()).unwrap();
            }
        }
        vec
    }
}

#[cfg(feature = "alloc")]
impl Packetize for BleMessage {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        let mut vec = alloc::vec::Vec::new();
        match self {
            BleMessage::DeviceRequest => {
                vec.push(DEVICE_REQUEST);
            }
            BleMessage::DeviceResponse(device_types, device_capabilities, object_id_segment) => {
                vec.push(DEVICE_RESPONSE);
                vec.extend_from_slice(&device_types.packetize());
                vec.extend_from_slice(&device_capabilities.packetize());
                vec.extend_from_slice(object_id_segment);
            }
            BleMessage::NonceRequest => {
                vec.push(NONCE_REQUEST);
            }
            BleMessage::NonceResponse(nonce, verify_bytes) => {
                vec.push(NONCE_RESPONSE);
                vec.extend_from_slice(nonce);
                vec.extend_from_slice(verify_bytes);
            }
            BleMessage::UnlockRequest(proof) => {
                vec.push(UNLOCK_REQUEST);
                let proof_packet = proof.packetize();
                vec.extend_from_slice(&proof_packet);
            }
            BleMessage::UnlockResponse(success, error) => {
                vec.push(UNLOCK_RESPONSE);
                vec.push(if *success {
                    UNLOCK_SUCCESS
                } else {
                    UNLOCK_FAILURE
                });
                vec.extend_from_slice(&error.packetize());
            }
        }
        vec
    }
}
