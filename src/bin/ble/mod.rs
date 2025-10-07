mod manager;
pub(crate) mod protocol;

use super::crypto::{Nonce, Proof};
use crate::shared::{CryptoError, DeviceCapability, DeviceType};
use heapless::Vec;

#[derive(Debug, Clone)]
pub enum BleMessage {
    DeviceRequest(u8),
    DeviceResponse(DeviceType, Vec<DeviceCapability, 3>, [u8; 12]), // 24-byte MongoDB ObjectId
    NonceRequest,
    NonceResponse(Nonce, [u8; 8]), // 8-byte signature tail
    UnlockRequest(Proof),
    UnlockResponse(bool, Option<CryptoError>),
}

impl From<(bool, Option<CryptoError>)> for BleMessage {
    fn from(value: (bool, Option<CryptoError>)) -> Self {
        BleMessage::UnlockResponse(value.0, value.1)
    }
}

impl From<(Nonce, [u8; 8])> for BleMessage {
    fn from((nonce, tail): (Nonce, [u8; 8])) -> Self {
        BleMessage::NonceResponse(nonce, tail)
    }
}

impl From<(DeviceType, Vec<DeviceCapability, 3>, [u8; 12])> for BleMessage {
    fn from(value: (DeviceType, Vec<DeviceCapability, 3>, [u8; 12])) -> Self {
        BleMessage::DeviceResponse(value.0, value.1, value.2)
    }
}
