// Copyright (c) 2025 Ethan Wu
// SPDX-License-Identifier: MIT
#![allow(unused)]

//! BluFi (Bluetooth + WiFi) configuration schemas for ESP32 beacon provisioning
//!
//! BluFi is Espressif's protocol for configuring WiFi credentials over Bluetooth.
//! This module provides types for:
//! - WiFi network scanning and selection
//! - WiFi credential provisioning
//! - Orchestrator connection configuration
//! - Provisioning status tracking

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use alloc::string::String;

/// WiFi security mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum WiFiSecurityMode {
    /// Open network (no password)
    Open,
    /// WEP encryption (deprecated)
    Wep,
    /// WPA-PSK encryption
    WpaPsk,
    /// WPA2-PSK encryption (most common)
    Wpa2Psk,
    /// WPA/WPA2-PSK mixed mode
    WpaWpa2Psk,
    /// WPA2 Enterprise
    Wpa2Enterprise,
    /// WPA3-PSK encryption
    Wpa3Psk,
    /// WPA2/WPA3-PSK mixed mode
    Wpa2Wpa3Psk,
}

/// WiFi network information from scan
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg(feature = "alloc")]
pub struct WiFiNetwork {
    /// Network SSID (name)
    pub ssid: String,
    /// BSSID (MAC address of access point)
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bssid: Option<String>,
    /// Signal strength (RSSI in dBm)
    pub rssi: i8,
    /// WiFi channel
    pub channel: u8,
    /// Security mode
    pub security: WiFiSecurityMode,
    /// Whether network is hidden
    #[cfg_attr(feature = "serde", serde(default))]
    pub hidden: bool,
}

/// BluFi provisioning configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg(feature = "alloc")]
pub struct BluFiConfig {
    /// WiFi SSID to connect to
    pub ssid: String,
    /// WiFi password
    pub password: String,
    /// WiFi security mode
    pub security: WiFiSecurityMode,
    /// Optional orchestrator URL
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub orchestrator_url: Option<String>,
    /// Optional orchestrator port
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub orchestrator_port: Option<u16>,
    /// Optional entity ID
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub entity_id: Option<String>,
    /// Optional beacon name
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub beacon_name: Option<String>,
    /// Optional beacon location
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub beacon_location: Option<BeaconLocation>,
}

/// Beacon location for provisioning
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BeaconLocation {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Floor identifier
    #[cfg(feature = "alloc")]
    pub floor: String,
}

/// BluFi provisioning state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum BluFiState {
    /// Idle, not started
    Idle,
    /// Scanning for BLE beacons
    Scanning,
    /// Connecting to beacon
    Connecting,
    /// Negotiating encryption
    Negotiating,
    /// Sending WiFi credentials
    Provisioning,
    /// Verifying WiFi connection
    Verifying,
    /// Successfully connected
    Connected,
    /// Failed
    Failed,
}

/// BluFi error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum BluFiErrorType {
    /// Bluetooth is disabled
    BluetoothDisabled,
    /// Beacon not found
    BeaconNotFound,
    /// Connection failed
    ConnectionFailed,
    /// Authentication failed
    AuthenticationFailed,
    /// Provisioning failed
    ProvisioningFailed,
    /// Timeout error
    TimeoutError,
    /// Unknown error
    UnknownError,
}

/// BluFi error details
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg(feature = "alloc")]
pub struct BluFiError {
    /// Error type
    pub error_type: BluFiErrorType,
    /// Error message
    pub message: String,
    /// Optional error details
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub details: Option<String>,
    /// Optional error code
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub code: Option<i32>,
}

/// BluFi provisioning result
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg(feature = "alloc")]
pub struct BluFiProvisioningResult {
    /// Whether provisioning was successful
    pub success: bool,
    /// Current state
    pub state: BluFiState,
    /// Optional message
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub message: Option<String>,
    /// Optional IP address assigned to beacon
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ip_address: Option<String>,
    /// Optional MAC address
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub mac_address: Option<String>,
    /// Optional connected SSID
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub connected_ssid: Option<String>,
    /// Optional error
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub error: Option<BluFiError>,
}

/// Beacon provisioning status
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg(feature = "alloc")]
pub struct BeaconProvisioningStatus {
    /// Beacon ID (hardware ID)
    pub beacon_id: i32,
    /// Device ID (hardware ID)
    pub device_id: String,
    /// Current provisioning state
    pub state: BluFiState,
    /// Whether WiFi is connected
    pub wifi_connected: bool,
    /// Whether orchestrator is connected
    pub orchestrator_connected: bool,
    /// Optional IP address
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ip_address: Option<String>,
    /// Optional firmware version
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub firmware_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_security_modes() {
        let modes = [
            WiFiSecurityMode::Open,
            WiFiSecurityMode::Wpa2Psk,
            WiFiSecurityMode::Wpa3Psk,
        ];
        assert_eq!(modes.len(), 3);
    }

    #[test]
    fn test_blufi_state_transitions() {
        let state = BluFiState::Idle;
        assert_eq!(state, BluFiState::Idle);
        let state = BluFiState::Scanning;
        assert_eq!(state, BluFiState::Scanning);
    }

    #[cfg(all(feature = "serde", feature = "alloc"))]
    #[test]
    fn test_wifi_network_serialization() {
        let network = WiFiNetwork {
            ssid: "TestNetwork".to_string(),
            bssid: Some("AA:BB:CC:DD:EE:FF".to_string()),
            rssi: -45,
            channel: 6,
            security: WiFiSecurityMode::Wpa2Psk,
            hidden: false,
        };

        let json = serde_json::to_string(&network).unwrap();
        assert!(json.contains("TestNetwork"));
    }

    #[cfg(all(feature = "serde", feature = "alloc"))]
    #[test]
    fn test_blufi_config_serialization() {
        let config = BluFiConfig {
            ssid: "MyWiFi".to_string(),
            password: "password123".to_string(),
            security: WiFiSecurityMode::Wpa2Psk,
            orchestrator_url: Some("http://localhost:50051".to_string()),
            orchestrator_port: Some(50051),
            entity_id: Some("entity_123".to_string()),
            beacon_name: Some("Beacon-01".to_string()),
            beacon_location: Some(BeaconLocation {
                x: 10.5,
                y: 20.3,
                floor: "L1".to_string(),
            }),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("MyWiFi"));
    }
}
