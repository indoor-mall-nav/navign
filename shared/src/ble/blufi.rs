//! BluFi Protocol Implementation
//!
//! BluFi is ESP-IDF's WiFi configuration protocol over Bluetooth.
//! This module implements the frame format and message types according to the ESP-IDF BluFi specification.
//!
//! **Current Implementation Status:**
//! - Only **SoftAP (Access Point) mode** is currently supported
//! - STA mode and mixed STA+AP mode are defined for protocol compliance but not yet implemented
//!
//! # Frame Format
//!
//! ```text
//! +---------------+---------------+---------------+---------------+--------+----------+
//! | Type (1 byte) | Frame Control | Sequence      | Data Length   | Data   | CheckSum |
//! |               | (1 byte)      | (1 byte)      | (1 byte)      | (var)  | (2 bytes)|
//! +---------------+---------------+---------------+---------------+--------+----------+
//! ```
//!
//! # Type Field
//!
//! - Bits 0-1: Frame type (Control = 0b00, Data = 0b01)
//! - Bits 2-7: Subtype (specific message type)

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use alloc::string::String;

/// Frame Control flags (1 byte)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FrameControl(pub u8);

impl FrameControl {
    pub const ENCRYPTED: u8 = 0x01;
    pub const CHECKSUM: u8 = 0x02;
    pub const DIRECTION_TO_PHONE: u8 = 0x04;
    pub const REQUIRE_ACK: u8 = 0x08;
    pub const FRAG: u8 = 0x10;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn with_encrypted(mut self) -> Self {
        self.0 |= Self::ENCRYPTED;
        self
    }

    pub fn with_checksum(mut self) -> Self {
        self.0 |= Self::CHECKSUM;
        self
    }

    pub fn with_direction_to_phone(mut self) -> Self {
        self.0 |= Self::DIRECTION_TO_PHONE;
        self
    }

    pub fn with_require_ack(mut self) -> Self {
        self.0 |= Self::REQUIRE_ACK;
        self
    }

    pub fn with_frag(mut self) -> Self {
        self.0 |= Self::FRAG;
        self
    }

    pub fn is_encrypted(&self) -> bool {
        self.0 & Self::ENCRYPTED != 0
    }

    pub fn has_checksum(&self) -> bool {
        self.0 & Self::CHECKSUM != 0
    }

    pub fn is_direction_to_phone(&self) -> bool {
        self.0 & Self::DIRECTION_TO_PHONE != 0
    }

    pub fn requires_ack(&self) -> bool {
        self.0 & Self::REQUIRE_ACK != 0
    }

    pub fn is_frag(&self) -> bool {
        self.0 & Self::FRAG != 0
    }
}

impl Default for FrameControl {
    fn default() -> Self {
        Self::new()
    }
}

/// WiFi operation mode
///
/// **Note:** Currently only `SoftAp` mode is supported.
/// Other modes are defined for protocol compliance but not yet implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
#[allow(dead_code)]
pub enum WifiOpmode {
    Null = 0x00,
    #[allow(dead_code)]
    Sta = 0x01,
    SoftAp = 0x02,
    #[allow(dead_code)]
    StaSoftAp = 0x03,
}

impl TryFrom<u8> for WifiOpmode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(WifiOpmode::Null),
            0x01 => Ok(WifiOpmode::Sta),
            0x02 => Ok(WifiOpmode::SoftAp),
            0x03 => Ok(WifiOpmode::StaSoftAp),
            _ => Err(()),
        }
    }
}

impl WifiOpmode {
    /// Check if this opmode is currently supported
    ///
    /// Currently only `SoftAp` mode is implemented.
    pub fn is_supported(&self) -> bool {
        matches!(self, WifiOpmode::SoftAp)
    }
}

/// WiFi authentication mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
#[allow(dead_code)]
pub enum WifiAuthMode {
    Open = 0x00,
    Wep = 0x01,
    WpaPsk = 0x02,
    Wpa2Psk = 0x03,
    WpaWpa2Psk = 0x04,
}

impl TryFrom<u8> for WifiAuthMode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(WifiAuthMode::Open),
            0x01 => Ok(WifiAuthMode::Wep),
            0x02 => Ok(WifiAuthMode::WpaPsk),
            0x03 => Ok(WifiAuthMode::Wpa2Psk),
            0x04 => Ok(WifiAuthMode::WpaWpa2Psk),
            _ => Err(()),
        }
    }
}

/// WiFi connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
#[allow(dead_code)]
pub enum WifiConnectionState {
    ConnectedWithIp = 0x00,
    Disconnected = 0x01,
    Connecting = 0x02,
    ConnectedNoIp = 0x03,
}

impl TryFrom<u8> for WifiConnectionState {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(WifiConnectionState::ConnectedWithIp),
            0x01 => Ok(WifiConnectionState::Disconnected),
            0x02 => Ok(WifiConnectionState::Connecting),
            0x03 => Ok(WifiConnectionState::ConnectedNoIp),
            _ => Err(()),
        }
    }
}

/// BluFi error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
#[allow(dead_code)]
pub enum BlufiError {
    SequenceError = 0x00,
    ChecksumError = 0x01,
    DecryptError = 0x02,
    EncryptError = 0x03,
    InitSecurityError = 0x04,
    DhMallocError = 0x05,
    DhParamError = 0x06,
    ReadParamError = 0x07,
    MakePublicError = 0x08,
    DataFormatError = 0x09,
    CalculateMd5Error = 0x0a,
    WifiScanError = 0x0b,
}

impl TryFrom<u8> for BlufiError {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(BlufiError::SequenceError),
            0x01 => Ok(BlufiError::ChecksumError),
            0x02 => Ok(BlufiError::DecryptError),
            0x03 => Ok(BlufiError::EncryptError),
            0x04 => Ok(BlufiError::InitSecurityError),
            0x05 => Ok(BlufiError::DhMallocError),
            0x06 => Ok(BlufiError::DhParamError),
            0x07 => Ok(BlufiError::ReadParamError),
            0x08 => Ok(BlufiError::MakePublicError),
            0x09 => Ok(BlufiError::DataFormatError),
            0x0a => Ok(BlufiError::CalculateMd5Error),
            0x0b => Ok(BlufiError::WifiScanError),
            _ => Err(()),
        }
    }
}

/// Control Frame types (Type = 0b00)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(dead_code)]
pub enum ControlFrame {
    /// ACK frame - acknowledges received frame
    Ack { acked_sequence: u8 },

    /// Set security mode for subsequent frames
    /// - Upper 4 bits: control frame security mode
    /// - Lower 4 bits: data frame security mode
    ///
    /// Mode bits:
    /// - 0b0000 = no checksum/encryption
    /// - 0b0001 = checksum only
    /// - 0b0010 = encryption only
    /// - 0b0011 = checksum + encryption
    SetSecurityMode { mode: u8 },

    /// Set WiFi operation mode
    ///
    /// **Note:** Currently only `WifiOpmode::SoftAp` is supported.
    SetOpmode { opmode: WifiOpmode },

    /// Connect to WiFi AP (after SSID/password are set)
    ///
    /// **Note:** STA mode is not currently supported (AP mode only)
    ConnectWifi,

    /// Disconnect from WiFi AP
    ///
    /// **Note:** STA mode is not currently supported (AP mode only)
    DisconnectWifi,

    /// Request WiFi status
    GetWifiStatus,

    /// Disconnect STA from SoftAP
    #[cfg(feature = "alloc")]
    DisconnectSta { mac_addresses: Vec<[u8; 6]> },

    #[cfg(feature = "heapless")]
    DisconnectSta {
        mac_addresses: heapless::Vec<[u8; 6], 8>,
    },

    /// Get version information
    GetVersion,

    /// Disconnect BLE GATT connection
    DisconnectBle,

    /// Get WiFi scan list
    GetWifiList,
}

/// Data Frame types (Type = 0b01)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(dead_code)]
pub enum DataFrame {
    /// Negotiation data for key exchange (DH, RSA, ECC)
    #[cfg(feature = "alloc")]
    NegotiationData { data: Vec<u8> },

    #[cfg(feature = "heapless")]
    NegotiationData { data: heapless::Vec<u8, 256> },

    /// BSSID for STA mode (when SSID is hidden)
    ///
    /// **Note:** STA mode is not currently supported (AP mode only)
    StaBssid { bssid: [u8; 6] },

    /// SSID for STA mode
    ///
    /// **Note:** STA mode is not currently supported (AP mode only)
    #[cfg(feature = "alloc")]
    StaSsid { ssid: String },

    #[cfg(feature = "heapless")]
    StaSsid { ssid: heapless::String<32> },

    /// Password for STA mode
    ///
    /// **Note:** STA mode is not currently supported (AP mode only)
    #[cfg(feature = "alloc")]
    StaPassword { password: String },

    #[cfg(feature = "heapless")]
    StaPassword { password: heapless::String<64> },

    /// SSID for SoftAP mode
    #[cfg(feature = "alloc")]
    SoftApSsid { ssid: String },

    #[cfg(feature = "heapless")]
    SoftApSsid { ssid: heapless::String<32> },

    /// Password for SoftAP mode
    #[cfg(feature = "alloc")]
    SoftApPassword { password: String },

    #[cfg(feature = "heapless")]
    SoftApPassword { password: heapless::String<64> },

    /// Maximum connection number for SoftAP (1-4)
    SoftApMaxConnNum { max_conn: u8 },

    /// Authentication mode for SoftAP
    SoftApAuthMode { auth_mode: WifiAuthMode },

    /// Channel number for SoftAP (1-14)
    SoftApChannel { channel: u8 },

    /// Username for enterprise WiFi
    #[cfg(feature = "alloc")]
    Username { username: String },

    #[cfg(feature = "heapless")]
    Username { username: heapless::String<64> },

    /// CA certificate for enterprise WiFi
    #[cfg(feature = "alloc")]
    CaCert { cert: Vec<u8> },

    #[cfg(feature = "heapless")]
    CaCert { cert: heapless::Vec<u8, 2048> },

    /// Client certificate for enterprise WiFi
    #[cfg(feature = "alloc")]
    ClientCert { cert: Vec<u8> },

    #[cfg(feature = "heapless")]
    ClientCert { cert: heapless::Vec<u8, 2048> },

    /// Server certificate for enterprise WiFi
    #[cfg(feature = "alloc")]
    ServerCert { cert: Vec<u8> },

    #[cfg(feature = "heapless")]
    ServerCert { cert: heapless::Vec<u8, 2048> },

    /// Client private key for enterprise WiFi
    #[cfg(feature = "alloc")]
    ClientPrivateKey { key: Vec<u8> },

    #[cfg(feature = "heapless")]
    ClientPrivateKey { key: heapless::Vec<u8, 2048> },

    /// Server private key for enterprise WiFi
    #[cfg(feature = "alloc")]
    ServerPrivateKey { key: Vec<u8> },

    #[cfg(feature = "heapless")]
    ServerPrivateKey { key: heapless::Vec<u8, 2048> },

    /// WiFi connection state report
    WifiConnectionState {
        opmode: WifiOpmode,
        sta_connection_state: WifiConnectionState,
        softap_connection_count: u8,
        #[cfg(feature = "alloc")]
        extra_info: Vec<u8>,
        #[cfg(feature = "heapless")]
        extra_info: heapless::Vec<u8, 128>,
    },

    /// Version information
    Version { major: u8, minor: u8 },

    /// WiFi scan list
    #[cfg(feature = "alloc")]
    WifiList { ssid_list: Vec<(i8, String)> }, // (RSSI, SSID)

    #[cfg(feature = "heapless")]
    WifiList {
        ssid_list: heapless::Vec<(i8, heapless::String<32>), 16>,
    },

    /// Error report
    Error { error: BlufiError },

    /// Custom data
    #[cfg(feature = "alloc")]
    CustomData { data: Vec<u8> },

    #[cfg(feature = "heapless")]
    CustomData { data: heapless::Vec<u8, 512> },

    /// Maximum WiFi reconnection time
    MaxWifiReconnectTime { max_time: u8 },

    /// WiFi connection end reason
    WifiConnectionEndReason { reason: u8 },

    /// RSSI at WiFi connection end (-128 for invalid)
    WifiConnectionEndRssi { rssi: i8 },
}

/// Complete BluFi message with frame metadata
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(dead_code)]
pub struct BlufiMessage {
    pub frame_control: FrameControl,
    pub sequence: u8,
    pub payload: BlufiPayload,
}

/// BluFi frame payload (either control or data)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(dead_code)]
#[allow(clippy::large_enum_variant)]
pub enum BlufiPayload {
    Control(ControlFrame),
    Data(DataFrame),
}

impl BlufiMessage {
    pub fn new_control(frame_control: FrameControl, sequence: u8, control: ControlFrame) -> Self {
        Self {
            frame_control,
            sequence,
            payload: BlufiPayload::Control(control),
        }
    }

    pub fn new_data(frame_control: FrameControl, sequence: u8, data: DataFrame) -> Self {
        Self {
            frame_control,
            sequence,
            payload: BlufiPayload::Data(data),
        }
    }

    /// Get the type byte for this message
    pub fn type_byte(&self) -> u8 {
        match &self.payload {
            BlufiPayload::Control(ctrl) => {
                let subtype = match ctrl {
                    ControlFrame::Ack { .. } => 0x00,
                    ControlFrame::SetSecurityMode { .. } => 0x01,
                    ControlFrame::SetOpmode { .. } => 0x02,
                    ControlFrame::ConnectWifi => 0x03,
                    ControlFrame::DisconnectWifi => 0x04,
                    ControlFrame::GetWifiStatus => 0x05,
                    ControlFrame::DisconnectSta { .. } => 0x06,
                    ControlFrame::GetVersion => 0x07,
                    ControlFrame::DisconnectBle => 0x08,
                    ControlFrame::GetWifiList => 0x09,
                };
                subtype << 2 // Control type = 0b00
            }
            BlufiPayload::Data(data) => {
                let subtype = match data {
                    DataFrame::NegotiationData { .. } => 0x00,
                    DataFrame::StaBssid { .. } => 0x01,
                    DataFrame::StaSsid { .. } => 0x02,
                    DataFrame::StaPassword { .. } => 0x03,
                    DataFrame::SoftApSsid { .. } => 0x04,
                    DataFrame::SoftApPassword { .. } => 0x05,
                    DataFrame::SoftApMaxConnNum { .. } => 0x06,
                    DataFrame::SoftApAuthMode { .. } => 0x07,
                    DataFrame::SoftApChannel { .. } => 0x08,
                    DataFrame::Username { .. } => 0x09,
                    DataFrame::CaCert { .. } => 0x0a,
                    DataFrame::ClientCert { .. } => 0x0b,
                    DataFrame::ServerCert { .. } => 0x0c,
                    DataFrame::ClientPrivateKey { .. } => 0x0d,
                    DataFrame::ServerPrivateKey { .. } => 0x0e,
                    DataFrame::WifiConnectionState { .. } => 0x0f,
                    DataFrame::Version { .. } => 0x10,
                    DataFrame::WifiList { .. } => 0x11,
                    DataFrame::Error { .. } => 0x12,
                    DataFrame::CustomData { .. } => 0x13,
                    DataFrame::MaxWifiReconnectTime { .. } => 0x14,
                    DataFrame::WifiConnectionEndReason { .. } => 0x15,
                    DataFrame::WifiConnectionEndRssi { .. } => 0x16,
                };
                (subtype << 2) | 0b01 // Data type = 0b01
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_control_flags() {
        let fc = FrameControl::new().with_encrypted().with_checksum();

        assert!(fc.is_encrypted());
        assert!(fc.has_checksum());
        assert!(!fc.is_direction_to_phone());
        assert!(!fc.requires_ack());
        assert!(!fc.is_frag());
    }

    #[test]
    fn test_frame_control_all_flags() {
        let fc = FrameControl::new()
            .with_encrypted()
            .with_checksum()
            .with_direction_to_phone()
            .with_require_ack()
            .with_frag();

        assert_eq!(fc.0, 0x1F);
    }

    #[test]
    fn test_wifi_opmode_conversion() {
        assert_eq!(WifiOpmode::try_from(0x00), Ok(WifiOpmode::Null));
        assert_eq!(WifiOpmode::try_from(0x01), Ok(WifiOpmode::Sta));
        assert_eq!(WifiOpmode::try_from(0x02), Ok(WifiOpmode::SoftAp));
        assert_eq!(WifiOpmode::try_from(0x03), Ok(WifiOpmode::StaSoftAp));
        assert_eq!(WifiOpmode::try_from(0x04), Err(()));
    }

    #[test]
    fn test_opmode_is_supported() {
        assert!(!WifiOpmode::Null.is_supported());
        assert!(!WifiOpmode::Sta.is_supported());
        assert!(WifiOpmode::SoftAp.is_supported());
        assert!(!WifiOpmode::StaSoftAp.is_supported());
    }

    #[test]
    fn test_wifi_auth_mode_conversion() {
        assert_eq!(WifiAuthMode::try_from(0x00), Ok(WifiAuthMode::Open));
        assert_eq!(WifiAuthMode::try_from(0x04), Ok(WifiAuthMode::WpaWpa2Psk));
        assert_eq!(WifiAuthMode::try_from(0x05), Err(()));
    }

    #[test]
    fn test_connection_state_conversion() {
        assert_eq!(
            WifiConnectionState::try_from(0x00),
            Ok(WifiConnectionState::ConnectedWithIp)
        );
        assert_eq!(
            WifiConnectionState::try_from(0x01),
            Ok(WifiConnectionState::Disconnected)
        );
        assert_eq!(WifiConnectionState::try_from(0x04), Err(()));
    }

    #[test]
    fn test_blufi_error_conversion() {
        assert_eq!(BlufiError::try_from(0x00), Ok(BlufiError::SequenceError));
        assert_eq!(BlufiError::try_from(0x0b), Ok(BlufiError::WifiScanError));
        assert_eq!(BlufiError::try_from(0x0c), Err(()));
    }

    #[test]
    fn test_control_frame_type_byte() {
        let msg = BlufiMessage::new_control(FrameControl::new(), 0, ControlFrame::ConnectWifi);

        // ConnectWifi subtype = 0x03, type = 0b00
        // Type byte = (0x03 << 2) | 0b00 = 0b00001100 = 0x0C
        assert_eq!(msg.type_byte(), 0x0C);
    }

    #[test]
    fn test_data_frame_type_byte() {
        let msg = BlufiMessage::new_data(
            FrameControl::new(),
            0,
            DataFrame::Version { major: 1, minor: 0 },
        );

        // Version subtype = 0x10, type = 0b01
        // Type byte = (0x10 << 2) | 0b01 = 0b01000001 = 0x41
        assert_eq!(msg.type_byte(), 0x41);
    }

    #[test]
    fn test_ack_frame_type_byte() {
        let msg = BlufiMessage::new_control(
            FrameControl::new(),
            0,
            ControlFrame::Ack { acked_sequence: 5 },
        );

        // ACK subtype = 0x00, type = 0b00
        // Type byte = (0x00 << 2) | 0b00 = 0x00
        assert_eq!(msg.type_byte(), 0x00);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_custom_data_frame() {
        let data = vec![1, 2, 3, 4, 5];
        let msg = BlufiMessage::new_data(
            FrameControl::new().with_checksum(),
            1,
            DataFrame::CustomData { data: data.clone() },
        );

        assert_eq!(msg.sequence, 1);
        assert!(msg.frame_control.has_checksum());

        if let BlufiPayload::Data(DataFrame::CustomData { data: payload_data }) = msg.payload {
            assert_eq!(payload_data, data);
        } else {
            panic!("Expected CustomData payload");
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_sta_credentials() {
        let ssid = String::from("MyNetwork");
        let password = String::from("MyPassword123");

        let ssid_msg = BlufiMessage::new_data(
            FrameControl::new(),
            1,
            DataFrame::StaSsid { ssid: ssid.clone() },
        );

        let pwd_msg = BlufiMessage::new_data(
            FrameControl::new(),
            2,
            DataFrame::StaPassword {
                password: password.clone(),
            },
        );

        if let BlufiPayload::Data(DataFrame::StaSsid { ssid: s }) = ssid_msg.payload {
            assert_eq!(s, ssid);
        } else {
            panic!("Expected StaSsid payload");
        }

        if let BlufiPayload::Data(DataFrame::StaPassword { password: p }) = pwd_msg.payload {
            assert_eq!(p, password);
        } else {
            panic!("Expected StaPassword payload");
        }
    }

    #[test]
    fn test_set_opmode() {
        let msg = BlufiMessage::new_control(
            FrameControl::new(),
            0,
            ControlFrame::SetOpmode {
                opmode: WifiOpmode::Sta,
            },
        );

        if let BlufiPayload::Control(ControlFrame::SetOpmode { opmode }) = msg.payload {
            assert_eq!(opmode, WifiOpmode::Sta);
        } else {
            panic!("Expected SetOpmode payload");
        }
    }

    #[test]
    fn test_security_mode() {
        // Upper 4 bits = control frame security, lower 4 bits = data frame security
        // 0b0011 = checksum + encryption for both
        let mode = 0x33;

        let msg = BlufiMessage::new_control(
            FrameControl::new(),
            0,
            ControlFrame::SetSecurityMode { mode },
        );

        if let BlufiPayload::Control(ControlFrame::SetSecurityMode { mode: m }) = msg.payload {
            assert_eq!(m, mode);
            assert_eq!(m & 0x0F, 0x03); // Data frame: checksum + encryption
            assert_eq!(m >> 4, 0x03); // Control frame: checksum + encryption
        } else {
            panic!("Expected SetSecurityMode payload");
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_wifi_list() {
        let list = vec![
            (-50, String::from("Network1")),
            (-70, String::from("Network2")),
            (-80, String::from("Network3")),
        ];

        let msg = BlufiMessage::new_data(
            FrameControl::new(),
            1,
            DataFrame::WifiList {
                ssid_list: list.clone(),
            },
        );

        if let BlufiPayload::Data(DataFrame::WifiList { ssid_list }) = msg.payload {
            assert_eq!(ssid_list.len(), 3);
            assert_eq!(ssid_list[0].0, -50);
            assert_eq!(ssid_list[0].1, "Network1");
        } else {
            panic!("Expected WifiList payload");
        }
    }
}
