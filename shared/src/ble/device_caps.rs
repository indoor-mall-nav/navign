#[cfg(feature = "postcard")]
use crate::{Depacketize, Packetize};
use bitflags::bitflags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    /// Capabilities that the device can report to the client.
    /// The sum of all capabilities is sent as a single byte.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct DeviceCapabilities: u8 {
        /// Capability to unlock the gate.
        const UNLOCK_GATE = 0b0000_0001;
        /// Capability to provide environmental data.
        const ENVIRONMENTAL_DATA = 0b0000_0010;
        /// Capability to report battery status.
        const BATTERY_STATUS = 0b0000_0100;
        /// Capability to perform RSSI calibration.
        const RSSI_CALIBRATION = 0b0000_1000;
        /// Capability to control a robot.
        const ROBOT_CONTROL = 0b0001_0000;
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for DeviceCapabilities {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "DeviceCapabilities({:02x})", self.bits());
    }
}

#[cfg(feature = "postcard")]
impl Depacketize for DeviceCapabilities {
    fn depacketize(packet: &[u8]) -> Option<Self> {
        postcard::from_bytes(packet).ok()
    }
}

#[cfg(all(feature = "alloc", feature = "postcard"))]
impl Packetize for DeviceCapabilities {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        postcard::to_allocvec(self).unwrap()
    }
}

#[cfg(all(feature = "heapless", feature = "postcard"))]
impl Packetize<8> for DeviceCapabilities {
    fn packetize(&self) -> heapless::Vec<u8, 8> {
        let mut buf = [0u8; 8];
        let used = postcard::to_slice(self, &mut buf).unwrap();
        let mut vec = heapless::Vec::<u8, 8>::new();
        vec.extend_from_slice(used).unwrap();
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_capabilities_packetize_depacketize() {
        let capabilities = DeviceCapabilities::UNLOCK_GATE
            | DeviceCapabilities::BATTERY_STATUS
            | DeviceCapabilities::ROBOT_CONTROL;

        #[cfg(feature = "heapless")]
        {
            let packet = capabilities.packetize();
            assert_eq!(packet.len(), 1);
            let depacketized = DeviceCapabilities::depacketize(&packet).unwrap();
            assert_eq!(capabilities, depacketized);
        }

        #[cfg(feature = "alloc")]
        {
            let packet = capabilities.packetize();
            assert_eq!(packet.len(), 1);
            let depacketized = DeviceCapabilities::depacketize(&packet).unwrap();
            assert_eq!(capabilities, depacketized);
        }
    }
}
