// Copyright (c) 2025 Ethan Wu
// SPDX-License-Identifier: MIT
#![allow(unused)]

//! Tauri commands for BluFi provisioning
//!
//! These commands expose BluFi functionality to the TypeScript frontend.
//! All business logic is implemented in Rust; TypeScript only handles UI.

use super::{
    ProvisioningBeacon, connect_beacon, disconnect_beacon, provision_beacon,
    scan_provisioning_beacons, scan_wifi_networks,
};
use navign_shared::{BluFiConfig, BluFiProvisioningResult, WiFiNetwork};
use serde_json::json;
use tauri_plugin_log::log::error;

/// Scan for beacons in provisioning mode
///
/// Returns JSON: `{ "status": "success"|"error", "beacons": [...] | "message": "..." }`
#[tauri::command]
pub async fn blufi_scan_beacons() -> Result<String, ()> {
    match scan_provisioning_beacons().await {
        Ok(beacons) => Ok(json!({
            "status": "success",
            "beacons": beacons,
        })
        .to_string()),
        Err(e) => {
            error!("Failed to scan beacons: {}", e);
            Ok(json!({
                "status": "error",
                "message": e.to_string(),
            })
            .to_string())
        }
    }
}

/// Connect to a beacon for provisioning
///
/// # Arguments
/// * `mac_address` - MAC address of the beacon
///
/// Returns JSON: `{ "status": "success"|"error", "message": "..." }`
#[tauri::command]
pub async fn blufi_connect(mac_address: String) -> Result<String, ()> {
    match connect_beacon(&mac_address).await {
        Ok(_) => Ok(json!({
            "status": "success",
            "message": "Connected to beacon",
        })
        .to_string()),
        Err(e) => {
            error!("Failed to connect to beacon: {}", e);
            Ok(json!({
                "status": "error",
                "message": e.to_string(),
            })
            .to_string())
        }
    }
}

/// Scan WiFi networks through connected beacon
///
/// Returns JSON: `{ "status": "success"|"error", "networks": [...] | "message": "..." }`
#[tauri::command]
pub async fn blufi_scan_wifi() -> Result<String, ()> {
    match scan_wifi_networks().await {
        Ok(networks) => Ok(json!({
            "status": "success",
            "networks": networks,
        })
        .to_string()),
        Err(e) => {
            error!("Failed to scan WiFi networks: {}", e);
            Ok(json!({
                "status": "error",
                "message": e.to_string(),
            })
            .to_string())
        }
    }
}

/// Provision WiFi credentials to beacon
///
/// # Arguments
/// * `config` - BluFi configuration (JSON string)
///
/// Returns JSON: `{ "status": "success"|"error", "result": {...} | "message": "..." }`
#[tauri::command]
pub async fn blufi_provision(config: String) -> Result<String, ()> {
    let config: BluFiConfig = match serde_json::from_str(&config) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to parse BluFi config: {}", e);
            return Ok(json!({
                "status": "error",
                "message": format!("Invalid configuration: {}", e),
            })
            .to_string());
        }
    };

    match provision_beacon(config).await {
        Ok(result) => Ok(json!({
            "status": if result.success { "success" } else { "error" },
            "result": result,
        })
        .to_string()),
        Err(e) => {
            error!("Failed to provision beacon: {}", e);
            Ok(json!({
                "status": "error",
                "message": e.to_string(),
            })
            .to_string())
        }
    }
}

/// Disconnect from beacon
///
/// Returns JSON: `{ "status": "success"|"error", "message": "..." }`
#[tauri::command]
pub async fn blufi_disconnect() -> Result<String, ()> {
    match disconnect_beacon().await {
        Ok(_) => Ok(json!({
            "status": "success",
            "message": "Disconnected from beacon",
        })
        .to_string()),
        Err(e) => {
            error!("Failed to disconnect from beacon: {}", e);
            Ok(json!({
                "status": "error",
                "message": e.to_string(),
            })
            .to_string())
        }
    }
}
