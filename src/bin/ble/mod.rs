mod manager;
pub(crate) mod protocol;

use heapless::Vec;
use super::crypto::{Nonce, Proof};
use crate::shared::{CryptoError, DeviceCapability, DeviceType};

#[derive(Debug, Clone)]
pub enum BleMessage {
    DeviceRequest,
    DeviceResponse(DeviceType, Vec<DeviceCapability, 3>, [u8; 24]), // 24-byte MongoDB ObjectId
    NonceRequest,
    NonceResponse(Nonce),
    UnlockRequest(Proof),
    UnlockResponse(bool, Option<CryptoError>),
}
