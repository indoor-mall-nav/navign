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

impl Depacketize for DeviceCapabilities {
    fn depacketize(packet: &[u8]) -> Option<Self> {
        if packet.len() != 1 {
            return None;
        }
        Some(DeviceCapabilities::from_bits_truncate(packet[0]))
    }
}

#[cfg(feature = "alloc")]
impl Packetize for DeviceCapabilities {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        alloc::vec![self.bits()]
    }
}

#[cfg(feature = "heapless")]
impl Packetize<1> for DeviceCapabilities {
    fn packetize(&self) -> heapless::Vec<u8, 1> {
        let mut vec = heapless::Vec::<u8, 1>::new();
        vec.push(self.bits()).unwrap();
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
