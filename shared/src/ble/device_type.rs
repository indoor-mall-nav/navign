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

#[cfg(feature = "postcard")]
impl Depacketize for DeviceTypes {
    fn depacketize(packet: &[u8]) -> Option<Self> {
        postcard::from_bytes(packet).ok()
    }
}

#[cfg(all(feature = "heapless", feature = "postcard"))]
impl Packetize<8> for DeviceTypes {
    fn packetize(&self) -> heapless::Vec<u8, 8> {
        let mut buf = [0u8; 8];
        let used = postcard::to_slice(self, &mut buf).unwrap();
        let mut vec = heapless::Vec::<u8, 8>::new();
        vec.extend_from_slice(used).unwrap();
        vec
    }
}

#[cfg(all(feature = "alloc", feature = "postcard"))]
impl Packetize for DeviceTypes {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        postcard::to_allocvec(self).unwrap()
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
