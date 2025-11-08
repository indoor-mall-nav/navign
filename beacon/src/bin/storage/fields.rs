#![allow(unused)]
use embedded_storage::{ReadStorage, Storage};
use esp_storage::FlashStorage;
use heapless::Vec;
use navign_shared::{Depacketize, DeviceCapabilities, DeviceTypes, Packetize};

#[derive(Debug, Clone)]
pub struct BeaconFields {
    r#type: DeviceTypes,
    capabilities: DeviceCapabilities,
    id: Vec<u8, 24>,
    major: u16,
    minor: u16,
    fix: u8,
    battery: u8,
}

pub enum ReadError {
    StorageError,
    DepacketizeError,
}

impl core::fmt::Display for ReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ReadError::StorageError => write!(f, "Storage error"),
            ReadError::DepacketizeError => write!(f, "Depacketize error"),
        }
    }
}

impl Packetize<32> for BeaconFields {
    fn packetize(&self) -> Vec<u8, 32> {
        let mut packet: Vec<u8, 32> = Vec::new();
        packet.push(self.r#type.bits()).ok();
        packet.push(self.capabilities.bits()).ok();
        packet.extend_from_slice(&self.id.as_slice()).ok();
        packet.push(0).ok(); // Null terminator for the string
        packet.extend_from_slice(&self.major.to_be_bytes()).ok();
        packet.extend_from_slice(&self.minor.to_be_bytes()).ok();
        packet.push(self.fix).ok();
        packet.push(self.battery).ok();
        packet
    }
}

impl Depacketize for BeaconFields {
    fn depacketize(packet: &[u8]) -> Option<Self> {
        if packet.len() < 32 {
            return None;
        }
        let r#type = DeviceTypes::from_bits(packet[0])?;
        let capabilities = DeviceCapabilities::from_bits(packet[1])?;
        let id_end = packet.iter().position(|&b| b == 0).unwrap_or(25);
        let id = Vec::from_slice(&packet[2..id_end]).ok()?;
        let major = u16::from_be_bytes([packet[id_end + 1], packet[id_end + 2]]);
        let minor = u16::from_be_bytes([packet[id_end + 3], packet[id_end + 4]]);
        let fix = packet[id_end + 5];
        let battery = packet[id_end + 6];
        Some(BeaconFields {
            r#type,
            capabilities,
            id,
            major,
            minor,
            fix,
            battery,
        })
    }
}

impl BeaconFields {
    pub fn read(storage: &mut FlashStorage) -> Result<Self, ReadError> {
        let mut buffer = [0u8; 32];
        storage
            .read(0, &mut buffer)
            .map_err(|_| ReadError::StorageError)?;
        Self::depacketize(&buffer).ok_or(ReadError::DepacketizeError)
    }

    pub fn write(&self, storage: &mut FlashStorage) -> Result<(), esp_storage::FlashStorageError> {
        let packet = self.packetize();
        storage.write(0, &packet)
    }
}
