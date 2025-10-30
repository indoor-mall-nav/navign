pub const DEVICE_REQUEST: u8 = 0x01;
pub const DEVICE_RESPONSE: u8 = 0x02;
pub const NONCE_REQUEST: u8 = 0x03;
pub const NONCE_RESPONSE: u8 = 0x04;
pub const UNLOCK_REQUEST: u8 = 0x05;
pub const UNLOCK_RESPONSE: u8 = 0x06;

pub const UNLOCK_SUCCESS: u8 = 0x01;

pub const DEVICE_REQUEST_COUNT_LENGTH: usize = 1;
pub const IDENTIFIER_LENGTH: usize = 1;
pub const NONCE_LENGTH: usize = 16;
pub const SIGNATURE_TAIL_LENGTH: usize = 8;
/// ObjectId as used in MongoDB, 24 bytes
pub const DEVICE_ID_LENGTH: usize = 24;
pub const DEVICE_TYPE_LENGTH: usize = 1;
pub const DEVICE_CAPABILITY_LENGTH: usize = 1;
pub const SERVER_SIGNATURE_LENGTH: usize = 64;
pub const DEVICE_BYTES_LENGTH: usize = 8;
pub const VERIFY_BYTES_LENGTH: usize = 8;
pub const TIMESTAMP_LENGTH: usize = 8;
pub const UNLOCK_LENGTH: usize = 1;
pub const UNLOCK_REASON_LENGTH: usize = 1;

pub const DEVICE_REQUEST_LENGTH: usize = IDENTIFIER_LENGTH + DEVICE_REQUEST_COUNT_LENGTH;
pub const DEVICE_RESPONSE_LENGTH: usize =
    IDENTIFIER_LENGTH + DEVICE_TYPE_LENGTH + DEVICE_CAPABILITY_LENGTH + DEVICE_ID_LENGTH;
pub const NONCE_REQUEST_LENGTH: usize = IDENTIFIER_LENGTH;
pub const NONCE_RESPONSE_LENGTH: usize = IDENTIFIER_LENGTH + NONCE_LENGTH + SIGNATURE_TAIL_LENGTH;
pub const UNLOCK_REQUEST_LENGTH: usize = IDENTIFIER_LENGTH
    + NONCE_LENGTH
    + DEVICE_BYTES_LENGTH
    + VERIFY_BYTES_LENGTH
    + TIMESTAMP_LENGTH
    + SERVER_SIGNATURE_LENGTH;
pub const UNLOCK_RESPONSE_LENGTH: usize = IDENTIFIER_LENGTH + UNLOCK_LENGTH + UNLOCK_REASON_LENGTH;

pub const UNLOCKER_SERVICE_UUID: &str = "134b1d88-cd91-8134-3e94-5c4052743845";
pub const UNLOCKER_CHARACTERISTIC_UUID: &str = "99d92823-9e38-72ff-6cf1-d2d593316af8";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_identifiers() {
        assert_eq!(DEVICE_REQUEST, 0x01);
        assert_eq!(DEVICE_RESPONSE, 0x02);
        assert_eq!(NONCE_REQUEST, 0x03);
        assert_eq!(NONCE_RESPONSE, 0x04);
        assert_eq!(UNLOCK_REQUEST, 0x05);
        assert_eq!(UNLOCK_RESPONSE, 0x06);
    }

    #[test]
    fn test_unlock_success_constant() {
        assert_eq!(UNLOCK_SUCCESS, 0x01);
    }

    #[test]
    fn test_length_constants() {
        assert_eq!(DEVICE_REQUEST_COUNT_LENGTH, 1);
        assert_eq!(IDENTIFIER_LENGTH, 1);
        assert_eq!(NONCE_LENGTH, 16);
        assert_eq!(SIGNATURE_TAIL_LENGTH, 8);
        assert_eq!(DEVICE_ID_LENGTH, 24);
        assert_eq!(DEVICE_TYPE_LENGTH, 1);
        assert_eq!(DEVICE_CAPABILITY_LENGTH, 1);
        assert_eq!(SERVER_SIGNATURE_LENGTH, 64);
        assert_eq!(DEVICE_BYTES_LENGTH, 8);
        assert_eq!(VERIFY_BYTES_LENGTH, 8);
        assert_eq!(TIMESTAMP_LENGTH, 8);
        assert_eq!(UNLOCK_LENGTH, 1);
        assert_eq!(UNLOCK_REASON_LENGTH, 1);
    }

    #[test]
    fn test_computed_message_lengths() {
        // Test that computed lengths match expected values
        assert_eq!(DEVICE_REQUEST_LENGTH, 2);
        assert_eq!(DEVICE_RESPONSE_LENGTH, 27);
        assert_eq!(NONCE_REQUEST_LENGTH, 1);
        assert_eq!(NONCE_RESPONSE_LENGTH, 25);
        assert_eq!(UNLOCK_REQUEST_LENGTH, 98);
        assert_eq!(UNLOCK_RESPONSE_LENGTH, 3);
    }

    #[test]
    fn test_device_request_length_calculation() {
        let expected = IDENTIFIER_LENGTH + DEVICE_REQUEST_COUNT_LENGTH;
        assert_eq!(DEVICE_REQUEST_LENGTH, expected);
    }

    #[test]
    fn test_device_response_length_calculation() {
        let expected = IDENTIFIER_LENGTH + DEVICE_TYPE_LENGTH + DEVICE_CAPABILITY_LENGTH + DEVICE_ID_LENGTH;
        assert_eq!(DEVICE_RESPONSE_LENGTH, expected);
    }

    #[test]
    fn test_nonce_response_length_calculation() {
        let expected = IDENTIFIER_LENGTH + NONCE_LENGTH + SIGNATURE_TAIL_LENGTH;
        assert_eq!(NONCE_RESPONSE_LENGTH, expected);
    }

    #[test]
    fn test_unlock_request_length_calculation() {
        let expected = IDENTIFIER_LENGTH
            + NONCE_LENGTH
            + DEVICE_BYTES_LENGTH
            + VERIFY_BYTES_LENGTH
            + TIMESTAMP_LENGTH
            + SERVER_SIGNATURE_LENGTH;
        assert_eq!(UNLOCK_REQUEST_LENGTH, expected);
    }

    #[test]
    fn test_unlock_response_length_calculation() {
        let expected = IDENTIFIER_LENGTH + UNLOCK_LENGTH + UNLOCK_REASON_LENGTH;
        assert_eq!(UNLOCK_RESPONSE_LENGTH, expected);
    }

    #[test]
    fn test_uuid_formats() {
        // Test that UUIDs are properly formatted
        assert!(UNLOCKER_SERVICE_UUID.contains('-'));
        assert!(UNLOCKER_CHARACTERISTIC_UUID.contains('-'));
        assert_eq!(UNLOCKER_SERVICE_UUID.len(), 36); // Standard UUID length
        assert_eq!(UNLOCKER_CHARACTERISTIC_UUID.len(), 36);
    }

    #[test]
    fn test_uuid_uniqueness() {
        // Ensure service and characteristic UUIDs are different
        assert_ne!(UNLOCKER_SERVICE_UUID, UNLOCKER_CHARACTERISTIC_UUID);
    }

    #[test]
    fn test_command_identifier_uniqueness() {
        // All command identifiers should be unique
        let identifiers = vec![
            DEVICE_REQUEST,
            DEVICE_RESPONSE,
            NONCE_REQUEST,
            NONCE_RESPONSE,
            UNLOCK_REQUEST,
            UNLOCK_RESPONSE,
        ];
        
        let mut unique_set = std::collections::HashSet::new();
        for id in identifiers {
            assert!(unique_set.insert(id), "Command identifier {} is not unique", id);
        }
    }
}
