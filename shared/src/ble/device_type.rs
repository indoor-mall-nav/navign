use crate::{Depacketize, Packetize};
use bitflags::bitflags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    /// Device Types
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct DeviceTypes: u8 {
        const MERCHANT = 0b0000_0001;
        const HOSPITAL = 0b0000_0010;
        const CAMPUS = 0b0000_0100;
        const TRANSPORT = 0b0000_1000;
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for DeviceTypes {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "DeviceTypes({:02x})", self.bits());
    }
}

impl Depacketize for DeviceTypes {
    fn depacketize(packet: &[u8]) -> Option<Self> {
        if packet.len() != 1 {
            return None;
        }
        Some(DeviceTypes::from_bits_truncate(packet[0]))
    }
}

#[cfg(feature = "heapless")]
impl Packetize<1> for DeviceTypes {
    fn packetize(&self) -> heapless::Vec<u8, 1> {
        self.try_packetize()
            .expect("DeviceTypes exceeds 1-byte buffer capacity")
    }

    fn try_packetize(&self) -> Result<heapless::Vec<u8, 1>, ()> {
        let mut vec = heapless::Vec::<u8, 1>::new();
        vec.push(self.bits()).map_err(|_| ())?;
        Ok(vec)
    }
}

#[cfg(feature = "alloc")]
impl Packetize for DeviceTypes {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        alloc::vec![self.bits()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_device_types_packetize_depacketize() {
        let device_types = DeviceTypes::MERCHANT;
        #[cfg(feature = "heapless")]
        {
            let packet = device_types.packetize();
            assert_eq!(packet.len(), 1);
            let depacketized = DeviceTypes::depacketize(&packet).unwrap();
            assert_eq!(device_types, depacketized);
        }
        #[cfg(feature = "alloc")]
        {
            let packet = device_types.packetize();
            assert_eq!(packet.len(), 1);
            let depacketized = DeviceTypes::depacketize(&packet).unwrap();
            assert_eq!(device_types, depacketized);
        }
    }
}
