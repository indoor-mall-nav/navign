use crate::unlocker::constants::*;
use crate::unlocker::proof::Proof;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
/// Capabilities that the device can report to the client.
/// The sum of all capabilities is sent as a single byte.
pub enum DeviceCapability {
    UnlockGate = 0b00000001,
    EnvironmentalData = 0b00000010,
    RssiCalibration = 0b00000100,
}

impl DeviceCapability {
    pub fn packetize(capabilities: &[DeviceCapability]) -> u8 {
        capabilities.iter().fold(0u8, |acc, cap| acc | (*cap as u8))
    }

    pub fn depacketize(byte: u8) -> Vec<DeviceCapability> {
        let mut capabilities = Vec::new();
        if byte & (DeviceCapability::UnlockGate as u8) != 0 {
            capabilities.push(DeviceCapability::UnlockGate);
        }
        if byte & (DeviceCapability::EnvironmentalData as u8) != 0 {
            capabilities.push(DeviceCapability::EnvironmentalData);
        }
        if byte & (DeviceCapability::RssiCalibration as u8) != 0 {
            capabilities.push(DeviceCapability::RssiCalibration);
        }
        capabilities
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceType {
    Merchant = 0x01,
    Pathway = 0x02,
    Connection = 0x03,
    Turnstile = 0x04,
}

impl DeviceType {
    pub fn packetize(&self) -> u8 {
        match self {
            Self::Merchant => 0x01,
            Self::Pathway => 0x02,
            Self::Connection => 0x03,
            Self::Turnstile => 0x04,
        }
    }

    pub fn depacketize(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::Merchant),
            0x02 => Some(Self::Pathway),
            0x03 => Some(Self::Connection),
            0x04 => Some(Self::Turnstile),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum CryptoError {
    InvalidSignature,
    InvalidKey,
    InvalidNonce,
    VerificationFailed,
    BufferFull,
    RateLimited,
    ReplayDetected,
    ServerPublicKeyNotSet,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidKey => write!(f, "Invalid key"),
            Self::InvalidNonce => write!(f, "Invalid nonce"),
            Self::VerificationFailed => write!(f, "Verification failed"),
            Self::BufferFull => write!(f, "Buffer full"),
            Self::RateLimited => write!(f, "Rate limited"),
            Self::ReplayDetected => write!(f, "Replay detected"),
            Self::ServerPublicKeyNotSet => write!(f, "Server public key not set"),
        }
    }
}

impl CryptoError {
    pub fn packetize(&self) -> u8 {
        match self {
            Self::InvalidSignature => 0x01,
            Self::InvalidKey => 0x02,
            Self::InvalidNonce => 0x03,
            Self::VerificationFailed => 0x04,
            Self::BufferFull => 0x05,
            Self::RateLimited => 0x06,
            Self::ReplayDetected => 0x07,
            Self::ServerPublicKeyNotSet => 0x08,
        }
    }

    pub fn depacketize(code: u8) -> Option<Self> {
        match code {
            0x01 => Some(Self::InvalidSignature),
            0x02 => Some(Self::InvalidKey),
            0x03 => Some(Self::InvalidNonce),
            0x04 => Some(Self::VerificationFailed),
            0x05 => Some(Self::BufferFull),
            0x06 => Some(Self::RateLimited),
            0x07 => Some(Self::ReplayDetected),
            0x08 => Some(Self::ServerPublicKeyNotSet),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BleMessage {
    DeviceRequest,
    DeviceResponse(DeviceType, Vec<DeviceCapability>, [u8; 24]), // 24-byte MongoDB ObjectId segment
    NonceRequest,
    NonceResponse([u8; 16], [u8; 8]),
    UnlockRequest(Proof),
    UnlockResponse(bool, CryptoError),
}

impl BleMessage {
    pub fn packetize(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        match self {
            BleMessage::DeviceRequest => {
                packet.push(DEVICE_REQUEST);
            }
            BleMessage::DeviceResponse(device_type, capabilities, object_id) => {
                packet.push(DEVICE_RESPONSE);
                packet.push(device_type.packetize());
                packet.push(DeviceCapability::packetize(capabilities));
                packet.extend_from_slice(object_id);
            }
            BleMessage::NonceRequest => {
                packet.push(NONCE_REQUEST);
            }
            BleMessage::NonceResponse(nonce, signature) => {
                packet.push(NONCE_RESPONSE);
                packet.extend_from_slice(nonce);
                packet.extend_from_slice(signature);
            }
            BleMessage::UnlockRequest(proof) => {
                packet.push(UNLOCK_REQUEST);
                packet.extend_from_slice(&proof.packetize());
            }
            BleMessage::UnlockResponse(success, error) => {
                packet.push(UNLOCK_RESPONSE);
                packet.push(if *success { 1 } else { 0 });
                packet.push(error.packetize());
            }
        }
        packet
    }

    pub fn depacketize(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }
        match data[0] {
            DEVICE_REQUEST if data.len() >= DEVICE_REQUEST_LENGTH => {
                Some(BleMessage::DeviceRequest)
            }
            DEVICE_RESPONSE if data.len() >= DEVICE_RESPONSE_LENGTH => {
                let device_type = DeviceType::depacketize(data[1])?;
                let capabilities = DeviceCapability::depacketize(data[2]);
                let mut object_id = [0u8; 24];
                object_id.copy_from_slice(&data[3..27]);
                Some(BleMessage::DeviceResponse(
                    device_type,
                    capabilities,
                    object_id,
                ))
            }
            NONCE_REQUEST if data.len() >= NONCE_REQUEST_LENGTH => Some(BleMessage::NonceRequest),
            NONCE_RESPONSE if data.len() >= NONCE_RESPONSE_LENGTH => {
                let mut nonce = [0u8; NONCE_LENGTH];
                nonce.copy_from_slice(&data[1..NONCE_LENGTH + 1]);
                let mut signature_tail = [0u8; SIGNATURE_TAIL_LENGTH];
                signature_tail.copy_from_slice(&data[NONCE_LENGTH + 1..NONCE_RESPONSE_LENGTH]);
                Some(BleMessage::NonceResponse(nonce, signature_tail))
            }
            UNLOCK_REQUEST if data.len() >= UNLOCK_REQUEST_LENGTH => {
                let proof = Proof::depacketize(&data[1..])?;
                Some(BleMessage::UnlockRequest(proof))
            }
            UNLOCK_RESPONSE if data.len() >= UNLOCK_RESPONSE_LENGTH => {
                let success = data[1] != UNLOCK_SUCCESS;
                let error = CryptoError::depacketize(data[2])?;
                Some(BleMessage::UnlockResponse(success, error))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unlocker::proof::Proof;
    use tauri_plugin_log::log::info;

    #[test]
    fn test_device_capability_packetize_depacketize() {
        let capabilities = vec![
            DeviceCapability::UnlockGate,
            DeviceCapability::EnvironmentalData,
        ];
        let byte = DeviceCapability::packetize(&capabilities);
        let depacketized = DeviceCapability::depacketize(byte);
        assert_eq!(capabilities, depacketized);
    }

    #[test]
    fn test_device_type_packetize_depacketize() {
        let device_type = DeviceType::Pathway;
        let byte = device_type.packetize();
        let depacketized = DeviceType::depacketize(byte).unwrap();
        assert_eq!(device_type, depacketized);
    }

    #[test]
    fn test_crypto_error_packetize_depacketize() {
        let error = CryptoError::InvalidSignature;
        let byte = error.packetize();
        let depacketized = CryptoError::depacketize(byte).unwrap();
        assert_eq!(error, depacketized);
    }

    #[test]
    fn test_ble_message_packetize_depacketize() {
        let original_message = BleMessage::DeviceResponse(
            DeviceType::Merchant,
            vec![DeviceCapability::UnlockGate],
            [1u8; 24],
        );
        let packet = original_message.packetize();
        info!("Packet: {:?}", packet);
        assert_eq!(packet.len(), DEVICE_RESPONSE_LENGTH);
        let depacketized_message = BleMessage::depacketize(&packet).unwrap();
        match (original_message, depacketized_message) {
            (
                BleMessage::DeviceResponse(orig_type, orig_caps, orig_id),
                BleMessage::DeviceResponse(dep_type, dep_caps, dep_id),
            ) => {
                assert_eq!(orig_type, dep_type);
                assert_eq!(orig_caps, dep_caps);
                assert_eq!(orig_id, dep_id);
            }
            _ => panic!("Mismatched message types"),
        }
    }

    #[test]
    fn test_unlock_request_packetize_depacketize() {
        let proof = Proof::new([0u8; 16], [1u8; 8], [2u8; 8], 123456789, [3u8; 64]);
        let original_message = BleMessage::UnlockRequest(proof);
        let packet = original_message.packetize();
        let depacketized_message = BleMessage::depacketize(&packet).unwrap();
        match (original_message, depacketized_message) {
            (BleMessage::UnlockRequest(orig_proof), BleMessage::UnlockRequest(dep_proof)) => {
                assert_eq!(orig_proof.nonce, dep_proof.nonce);
                assert_eq!(orig_proof.device_bytes, dep_proof.device_bytes);
                assert_eq!(orig_proof.verify_bytes, dep_proof.verify_bytes);
                assert_eq!(orig_proof.timestamp, dep_proof.timestamp);
                assert_eq!(orig_proof.server_signature, dep_proof.server_signature);
            }
            _ => panic!("Mismatched message types"),
        }
    }
}
