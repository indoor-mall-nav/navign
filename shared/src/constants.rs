// BLE Protocol Message Identifiers
pub const DEVICE_REQUEST: u8 = 0x01;
pub const DEVICE_RESPONSE: u8 = 0x02;
pub const NONCE_REQUEST: u8 = 0x03;
pub const NONCE_RESPONSE: u8 = 0x04;
pub const UNLOCK_REQUEST: u8 = 0x05;
pub const UNLOCK_RESPONSE: u8 = 0x06;
pub const DEBUG_REQUEST: u8 = 0xFF;
pub const DEBUG_RESPONSE: u8 = 0xFE;

// Unlock Status Codes
pub const UNLOCK_SUCCESS: u8 = 0x01;
pub const UNLOCK_FAILURE: u8 = 0x00;

// Beacon-specific constants
pub const MAX_PACKET_SIZE: usize = 128;
pub const MAX_ATTEMPTS: u8 = 5;

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
