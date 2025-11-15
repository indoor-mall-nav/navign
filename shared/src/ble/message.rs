#[cfg(feature = "postcard")]
use crate::Packetize;
use crate::{DeviceCapabilities, DeviceTypes, Proof, errors::CryptoError};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

#[cfg(all(feature = "heapless", feature = "postcard"))]
impl Packetize<128> for BleMessage {
    fn packetize(&self) -> heapless::Vec<u8, 128> {
        let mut buf = [0u8; 128];
        let used = postcard::to_slice(self, &mut buf).unwrap();
        let mut vec = heapless::Vec::<u8, 128>::new();
        vec.extend_from_slice(used).unwrap();
        vec
    }

    fn try_packetize(&self) -> Result<heapless::Vec<u8, 128>, crate::PacketizeError> {
        let mut buf = [0u8; 128];
        let used = postcard::to_slice(self, &mut buf)
            .map_err(|_| crate::PacketizeError::BufferOverflow)?;
        let mut vec = heapless::Vec::<u8, 128>::new();
        vec.extend_from_slice(used)
            .map_err(|_| crate::PacketizeError::BufferOverflow)?;
        Ok(vec)
    }
}

#[cfg(all(feature = "alloc", feature = "postcard"))]
impl Packetize for BleMessage {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        postcard::to_allocvec(self).unwrap()
    }
}

#[cfg(feature = "postcard")]
impl crate::Depacketize for BleMessage {
    fn depacketize(data: &[u8]) -> Option<Self> {
        postcard::from_bytes(data).ok()
    }
}
