#[cfg(feature = "alloc")]
use alloc::string::String;
use core::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "mongodb")]
use bson::oid::ObjectId;

#[cfg(all(feature = "mongodb", feature = "serde"))]
use bson::serde_helpers::serialize_object_id_as_hex_string;

/// Firmware artifact schema - represents a beacon firmware binary
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Firmware {
    pub id: String,
    /// Semantic version of the firmware (e.g., "1.0.0")
    pub version: String,
    /// Target device type
    pub device: FirmwareDevice,
    /// Short description of the firmware
    pub description: Option<String>,
    /// File path or storage key for the firmware binary
    pub file_path: String,
    /// File size in bytes
    pub file_size: u64,
    /// SHA-256 checksum of the firmware binary (hex string)
    pub checksum: String,
    /// Whether this is the latest version for this device
    pub is_latest: bool,
    /// Git commit hash if available
    pub git_commit: Option<String>,
    /// Build timestamp
    pub build_time: i64,
    /// Upload timestamp
    pub created_at: i64,
    /// Optional release notes
    pub release_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum FirmwareDevice {
    Esp32,
    Esp32C3,
    Esp32C5,
    Esp32C6,
    Esp32S3,
}

impl FirmwareDevice {
    pub fn as_str(&self) -> &'static str {
        match self {
            FirmwareDevice::Esp32 => "esp32",
            FirmwareDevice::Esp32C3 => "esp32c3",
            FirmwareDevice::Esp32C5 => "esp32c5",
            FirmwareDevice::Esp32C6 => "esp32c6",
            FirmwareDevice::Esp32S3 => "esp32s3",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseFirmwareDeviceError;

impl FromStr for FirmwareDevice {
    type Err = ParseFirmwareDeviceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "esp32" => Ok(FirmwareDevice::Esp32),
            "esp32c3" => Ok(FirmwareDevice::Esp32C3),
            "esp32c5" => Ok(FirmwareDevice::Esp32C5),
            "esp32c6" => Ok(FirmwareDevice::Esp32C6),
            "esp32s3" => Ok(FirmwareDevice::Esp32S3),
            _ => Err(ParseFirmwareDeviceError),
        }
    }
}

/// Request to upload new firmware
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FirmwareUploadRequest {
    pub version: String,
    pub device: FirmwareDevice,
    pub description: Option<String>,
    pub checksum: String,
    pub git_commit: Option<String>,
    pub release_notes: Option<String>,
}

/// Response after firmware upload
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FirmwareUploadResponse {
    pub id: String,
    pub version: String,
    pub device: FirmwareDevice,
    pub file_size: u64,
    pub checksum: String,
    pub created_at: i64,
}

/// Query parameters for firmware listing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FirmwareQuery {
    pub device: Option<FirmwareDevice>,
    pub version: Option<String>,
    pub latest_only: Option<bool>,
}
