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
}
