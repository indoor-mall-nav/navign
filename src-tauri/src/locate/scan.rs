//! # Beacon Data
//!
//! After

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tauri_plugin_blec::get_handler;
use tauri_plugin_blec::models::{BleDevice, ScanFilter};
use uuid::Uuid;

fn service_id_to_uuid(service_id: u16) -> Uuid {
    Uuid::from_fields(
        service_id as u32,
        0x0000,
        0x1000,
        &[0x80, 0x00, 0x00, 0x80, 0x5F, 0x9B, 0x34, 0xFB],
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanError {
    NotInitialized,
    CannotStartScan,
    ScanFailed(String),
    NoDevicesFound,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::NotInitialized => write!(f, "BLE not initialized"),
            ScanError::CannotStartScan => write!(f, "Cannot start scan"),
            ScanError::ScanFailed(err) => write!(f, "Scan failed: {}", err),
            ScanError::NoDevicesFound => write!(f, "No devices found"),
        }
    }
}

impl std::error::Error for ScanError {}

pub async fn scan_devices() -> Result<Vec<BleDevice>, ScanError> {
    if cfg!(all(desktop, dev)) {
        return Ok(
            vec![
                BleDevice {
                    address: "48:F6:EE:21:B0:7C".to_string(),
                    name: "NAVIGN_BEACON".to_string(),
                    rssi: Some(-45),
                    services: vec![
                        service_id_to_uuid(0x1819),
                        service_id_to_uuid(0x1821),
                    ],
                    manufacturer_data: HashMap::new(),
                    service_data: HashMap::new(),
                    is_bonded: false,
                    is_connected: false
                }
            ]
        );
    }
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<BleDevice>>(10);
    let handler = get_handler().map_err(|_| ScanError::NotInitialized)?;
    if handler.is_scanning().await {
        handler
            .stop_scan()
            .await
            .map_err(|_| ScanError::CannotStartScan)?;
    }
    handler
        .discover(
            Some(tx),
            3000,
            ScanFilter::AllServices(vec![service_id_to_uuid(0x1819), service_id_to_uuid(0x1821)]),
            true,
        )
        .await
        .map_err(|e| ScanError::ScanFailed(e.to_string()))?;
    let mut devices = Vec::new();
    if let Some(devs) = rx.recv().await {
        devices = devs
            .into_iter()
            .filter(|d| d.rssi.is_some() && d.name == "NAVIGN_BEACON")
            .collect();
    }
    handler.stop_scan().await.ok();
    if devices.is_empty() {
        Err(ScanError::NoDevicesFound)
    } else {
        Ok(devices)
    }
}

pub async fn stop_scan() -> Result<(), ScanError> {
    if cfg!(all(desktop, dev)) {
        return Ok(());
    }
    let handler = get_handler().map_err(|_| ScanError::NotInitialized)?;
    if handler.is_scanning().await {
        handler
            .stop_scan()
            .await
            .map_err(|_| ScanError::CannotStartScan)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_uuid_conversion() {
        let uuid = service_id_to_uuid(0x1819);
        assert_eq!(uuid.to_string(), "00001819-0000-1000-8000-00805f9b34fb");
        let uuid = service_id_to_uuid(0x1821);
        assert_eq!(uuid.to_string(), "00001821-0000-1000-8000-00805f9b34fb");
    }

    #[test]
    fn test_service_id_to_uuid_conversion() {
        let uuid_1819 = service_id_to_uuid(0x1819);
        assert_eq!(uuid_1819.to_string(), "00001819-0000-1000-8000-00805f9b34fb");

        let uuid_1821 = service_id_to_uuid(0x1821);
        assert_eq!(uuid_1821.to_string(), "00001821-0000-1000-8000-00805f9b34fb");
    }

    #[test]
    fn test_scan_error_display() {
        assert_eq!(ScanError::NotInitialized.to_string(), "BLE not initialized");
        assert_eq!(ScanError::CannotStartScan.to_string(), "Cannot start scan");
        assert_eq!(ScanError::ScanFailed("test error".to_string()).to_string(), "Scan failed: test error");
        assert_eq!(ScanError::NoDevicesFound.to_string(), "No devices found");
    }

    #[tokio::test]
    async fn test_scan_devices_desktop_mock() {
        // This test runs the mock data path for desktop development
        let result = scan_devices().await;

        if cfg!(all(desktop, dev)) {
            assert!(result.is_ok());
            let devices = result.unwrap();
            assert_eq!(devices.len(), 1);
            assert_eq!(devices[0].address, "48:F6:EE:21:B0:7C");
            assert_eq!(devices[0].name, "NAVIGN_BEACON");
            assert_eq!(devices[0].rssi, Some(-45));
            assert_eq!(devices[0].services.len(), 2);
        }
    }

    #[test]
    fn test_ble_device_filtering() {
        let mock_devices = vec![
            BleDevice {
                address: "48:F6:EE:21:B0:7C".to_string(),
                name: "NAVIGN_BEACON".to_string(),
                rssi: Some(-45),
                services: vec![],
                manufacturer_data: HashMap::new(),
                service_data: HashMap::new(),
                is_bonded: false,
                is_connected: false,
            },
            BleDevice {
                address: "AA:BB:CC:DD:EE:FF".to_string(),
                name: "OTHER_BEACON".to_string(),
                rssi: Some(-60),
                services: vec![],
                manufacturer_data: HashMap::new(),
                service_data: HashMap::new(),
                is_bonded: false,
                is_connected: false,
            },
            BleDevice {
                address: "11:22:33:44:55:66".to_string(),
                name: "NAVIGN_BEACON".to_string(),
                rssi: None, // Should be filtered out
                services: vec![],
                manufacturer_data: HashMap::new(),
                service_data: HashMap::new(),
                is_bonded: false,
                is_connected: false,
            },
        ];

        let filtered: Vec<BleDevice> = mock_devices
            .into_iter()
            .filter(|d| d.rssi.is_some() && d.name == "NAVIGN_BEACON")
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].address, "48:F6:EE:21:B0:7C");
    }

    #[test]
    fn test_service_uuid_consistency() {
        // Test that UUIDs are consistent across multiple calls
        let uuid1 = service_id_to_uuid(0x1819);
        let uuid2 = service_id_to_uuid(0x1819);
        assert_eq!(uuid1, uuid2);

        // Test different service IDs produce different UUIDs
        let uuid_a = service_id_to_uuid(0x1819);
        let uuid_b = service_id_to_uuid(0x1821);
        assert_ne!(uuid_a, uuid_b);
    }

    #[tokio::test]
    async fn test_stop_scan_desktop_mock() {
        // Test that stop_scan works correctly in desktop mock mode
        let result = stop_scan().await;

        if cfg!(all(desktop, dev)) {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_scan_error_traits() {
        let error = ScanError::NotInitialized;

        // Test Debug trait
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NotInitialized"));

        // Test Clone trait
        let cloned_error = error.clone();
        assert_eq!(format!("{:?}", error), format!("{:?}", cloned_error));

        // Test Error trait
        use std::error::Error;
        let error_trait: &dyn Error = &error;
        assert_eq!(error_trait.to_string(), "BLE not initialized");
    }

    #[test]
    fn test_beacon_filtering_edge_cases() {
        let edge_case_devices = vec![
            // Device with empty name
            BleDevice {
                address: "00:00:00:00:00:00".to_string(),
                name: "".to_string(),
                rssi: Some(-50),
                services: vec![],
                manufacturer_data: HashMap::new(),
                service_data: HashMap::new(),
                is_bonded: false,
                is_connected: false,
            },
            // Device with partial name match
            BleDevice {
                address: "11:11:11:11:11:11".to_string(),
                name: "NAVIGN".to_string(),
                rssi: Some(-50),
                services: vec![],
                manufacturer_data: HashMap::new(),
                service_data: HashMap::new(),
                is_bonded: false,
                is_connected: false,
            },
            // Device with case-different name
            BleDevice {
                address: "22:22:22:22:22:22".to_string(),
                name: "navign_beacon".to_string(),
                rssi: Some(-50),
                services: vec![],
                manufacturer_data: HashMap::new(),
                service_data: HashMap::new(),
                is_bonded: false,
                is_connected: false,
            },
        ];

        let filtered: Vec<BleDevice> = edge_case_devices
            .into_iter()
            .filter(|d| d.rssi.is_some() && d.name == "NAVIGN_BEACON")
            .collect();

        // None should match due to exact string matching requirement
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_rssi_filtering() {
        let rssi_test_devices = vec![
            // Strong signal
            BleDevice {
                address: "AA:AA:AA:AA:AA:AA".to_string(),
                name: "NAVIGN_BEACON".to_string(),
                rssi: Some(-30),
                services: vec![],
                manufacturer_data: HashMap::new(),
                service_data: HashMap::new(),
                is_bonded: false,
                is_connected: false,
            },
            // Weak signal
            BleDevice {
                address: "BB:BB:BB:BB:BB:BB".to_string(),
                name: "NAVIGN_BEACON".to_string(),
                rssi: Some(-90),
                services: vec![],
                manufacturer_data: HashMap::new(),
                service_data: HashMap::new(),
                is_bonded: false,
                is_connected: false,
            },
            // No RSSI data
            BleDevice {
                address: "CC:CC:CC:CC:CC:CC".to_string(),
                name: "NAVIGN_BEACON".to_string(),
                rssi: None,
                services: vec![],
                manufacturer_data: HashMap::new(),
                service_data: HashMap::new(),
                is_bonded: false,
                is_connected: false,
            },
        ];

        let filtered: Vec<BleDevice> = rssi_test_devices
            .into_iter()
            .filter(|d| d.rssi.is_some() && d.name == "NAVIGN_BEACON")
            .collect();

        assert_eq!(filtered.len(), 2);

        // Check that devices with RSSI are included
        let addresses: Vec<&str> = filtered.iter().map(|d| d.address.as_str()).collect();
        assert!(addresses.contains(&"AA:AA:AA:AA:AA:AA"));
        assert!(addresses.contains(&"BB:BB:BB:BB:BB:BB"));
        assert!(!addresses.contains(&"CC:CC:CC:CC:CC:CC"));
    }
}
