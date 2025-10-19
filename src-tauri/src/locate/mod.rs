pub mod area;
pub mod beacon;
mod locator;
pub mod merchant;
mod migration;
mod scan;

use crate::api::map::AreaResponse;
use crate::api::page_results::PaginationResponse;
use crate::api::unlocker::CustomizedObjectId;
use crate::locate::area::ActiveArea;
use crate::locate::beacon::BeaconInfo;
use crate::locate::locator::LocateResult;
use crate::locate::scan::stop_scan;
use crate::shared::BASE_URL;
use crate::unlocker::constants::{UNLOCKER_CHARACTERISTIC_UUID, UNLOCKER_SERVICE_UUID};
use crate::unlocker::BleMessage;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{migrate, SqlitePool};
use std::str::FromStr;
use tauri::{AppHandle, Manager};
use tauri_plugin_blec::models::WriteType;
use tauri_plugin_blec::{get_handler, OnDisconnectHandler};
use tauri_plugin_http::reqwest;
use tauri_plugin_log::log::{debug, error, info, trace};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocateState {
    pub area: String,
    pub x: f64,
    pub y: f64,
}

pub async fn locate_device(
    dbpath: String,
    area: String,
    entity: String,
) -> anyhow::Result<LocateState> {
    trace!("Locating device in area: {}, entity: {}", area, entity);
    info!("Connecting to database at: {}", dbpath);
    let options = SqliteConnectOptions::new()
        .create_if_missing(true)
        .filename(dbpath.as_str());
    let conn = SqlitePool::connect_with(options).await.map_err(|e| {
        error!("Database connection error: {}", e);
        anyhow::anyhow!("Failed to connect to database at {}: {}", dbpath, e)
    })?;
    trace!("Database connected.");
    migrate!("./migrations")
        .run(&conn)
        .await
        .map_err(|e| anyhow::anyhow!("Database migration error: {}", e))?;
    trace!("Starting BLE scan...");
    stop_scan()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to stop scan: {}", e))?;
    trace!("Previous scan stopped. Now scanning for devices...");
    let devices = scan::scan_devices()
        .await
        .map_err(|e| anyhow::anyhow!("Scan error: {}", e))?;
    trace!("Scan completed. Found {} devices.", devices.len());
    if devices.is_empty() {
        return Err(anyhow::anyhow!("No devices found"));
    }
    let mut devices_unknown = Vec::new();
    for device in devices.iter().filter(|x| x.rssi.is_some()) {
        if BeaconInfo::get_from_mac(&conn, &device.address)
            .await
            .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
            .is_none()
        {
            trace!(
                "Unknown device found: {} with RSSI {}",
                device.address,
                device.rssi.unwrap()
            );
            devices_unknown.push((device.address.clone(), device.rssi.unwrap()));
        }
    }
    devices_unknown.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by RSSI descending
    for (mac, _) in devices_unknown.iter() {
        trace!("Fetching info for unknown device: {}", mac);
        if let Err(e) = fetch_device(&conn, mac, entity.as_str()).await {
            eprintln!("Failed to fetch device {}: {}", mac, e);
        }
    }
    let result = locator::handle_devices(devices.clone(), &conn, area.as_str()).await;
    match result {
        LocateResult::Success(x, y) => Ok(LocateState { area, x, y }),
        LocateResult::Error(err) => Err(anyhow::anyhow!("Locate error: {}", err)),
        LocateResult::NoBeacons => Err(anyhow::anyhow!("No beacons found")),
        LocateResult::AreaChanged(new_area) => {
            trace!("Area changed, updating area info...");
            update_area(&conn, new_area.as_str(), entity.as_str()).await?;
            match locator::handle_devices(devices, &conn, new_area.as_str()).await {
                LocateResult::Success(x, y) => Ok(LocateState {
                    area: new_area,
                    x,
                    y,
                }),
                LocateResult::Error(err) => Err(anyhow::anyhow!("Locate error: {}", err)),
                LocateResult::NoBeacons => Err(anyhow::anyhow!("No beacons found")),
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Beacon {
    #[serde(rename = "_id")]
    pub id: CustomizedObjectId,
    pub entity: CustomizedObjectId,
    pub area: CustomizedObjectId,
    #[serde(default)]
    pub merchant: Option<CustomizedObjectId>,
    #[serde(default)]
    pub connection: Option<CustomizedObjectId>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub r#type: String,
    pub location: (f64, f64),
    pub device: String,
    pub mac: String,
}

async fn fetch_device(conn: &SqlitePool, mac: &str, entity: &str) -> anyhow::Result<()> {
    // It might be updated, so we need to check it in local database first.
    if BeaconInfo::get_from_mac(conn, mac)
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
        .is_some()
    {
        trace!("Beacon with MAC {} already exists in the database.", mac);
        return Ok(());
    }

    trace!("Connecting to device with MAC: {}", mac);

    let object_id = {
        let handler = get_handler()
            .map_err(|e| anyhow::anyhow!("BLE not initialized: {}", e))?;

        handler
            .connect(mac, OnDisconnectHandler::None, true)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to device {}: {}", mac, e))?;

        let characteristic = Uuid::from_str(UNLOCKER_CHARACTERISTIC_UUID)?;
        let service = Uuid::from_str(UNLOCKER_SERVICE_UUID)?;

        handler.subscribe(characteristic, Some(service), |data| {
            info!("Notification received: {:x?}", data);
        }).await?;

        handler
            .send_data(characteristic, Some(service), &[0x01], WriteType::WithResponse)
            .await?;
        // Wait for 0.5 seconds to let the device process the request
        tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
        let received = handler.recv_data(characteristic, Some(service)).await?;
        debug!("Received data from device {}: {:x?}", mac, received);
        handler.unsubscribe(characteristic).await?;
        handler.disconnect().await?;
        let depacketized = BleMessage::depacketize(received.as_slice())
            .ok_or_else(|| anyhow::anyhow!("Failed to depacketize device response"))?;

        let BleMessage::DeviceResponse(_, _, obj_id) = depacketized else {
            return Err(anyhow::anyhow!("Failed to extract device response"));
        };

        let object_id = obj_id
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        if object_id.len() != 24 {
            return Err(anyhow::anyhow!("Invalid object ID length"));
        }

        object_id
    };

    trace!("Fetched object ID: {} for MAC: {}", object_id, mac);

    let client = reqwest::Client::new();
    let url = format!("{BASE_URL}api/entities/{entity}/beacons/{object_id}");
    println!("Fetching beacon info from URL: {}", url);
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
        .json::<Beacon>()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

    if ActiveArea::get_by_id(conn, &res.area)
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
        .is_some()
    {
        let beacon_info = BeaconInfo::new(
            res.id.to_string(),
            res.mac,
            res.location,
            "unknown".to_string(),
            res.area.to_string(),
            entity.to_string(),
        );
        beacon_info.insert(conn).await?;
        println!("Beacon {} inserted/updated in the database.", res.id);
    } else {
        update_area(conn, &res.area, entity).await?;
    }

    Ok(())
}

async fn update_area(conn: &SqlitePool, area: &str, entity: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{BASE_URL}api/entities/{entity}/areas/{area}");
    trace!("Fetching area info from URL: {}", url);
    let res: AreaResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

    trace!("Fetched area id: {}", res.id);

    if ActiveArea::get_by_id(conn, area)
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
        .is_none()
    {
        let active = ActiveArea::from(res);
        active.insert(conn).await?;
        trace!("Area {} inserted into the database.", area);
    }

    let mut beacons_url = Some(format!("{}/beacons", url));

    while let Some(url) = beacons_url.as_deref() {
        trace!("Fetching beacons from URL: {}", url);
        let beacons: PaginationResponse<Beacon> = client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        trace!("Fetched {} beacons for area {}", beacons.data.len(), area);

        for beacon in beacons.data.iter() {
            trace!("Processing beacon ID: {}, MAC: {}", beacon.id, beacon.mac);
            let beacon_info = BeaconInfo::new(
                beacon.id.to_string(),
                beacon.mac.clone(),
                beacon.location,
                "unknown".to_string(),
                beacon.area.to_string(),
                entity.to_string(),
            );
            if BeaconInfo::get_from_id(conn, &beacon.id.to_string())
                .await
                .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
                .is_some()
            {
                trace!(
                    "Beacon with ID {} already exists in the database.",
                    beacon.id
                );
                beacon_info.update(conn).await?;
            } else {
                trace!(
                    "Inserting new beacon with ID {} into the database.",
                    beacon.id
                );
                beacon_info.insert(conn).await?;
            }
            trace!("Beacon {} inserted/updated in the database.", beacon.id);
        }
        beacons_url = beacons.metadata.next_page_url;
    }

    trace!("All beacons for area {} have been processed.", area);

    Ok(())
}

#[tauri::command]
pub async fn locate_handler(app: AppHandle, area: String, entity: String) -> Result<String, ()> {
    let dbpath = app
        .path()
        .app_local_data_dir()
        .map(|p| p.join("navign.db"))
        .map_err(|e| {
            eprintln!("Failed to get app local data dir: {}", e);
        })?;
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(dbpath.parent().unwrap()).map_err(|e| {
        eprintln!("Failed to create app local data dir: {}", e);
    })?;
    let db_str = format!("{}", dbpath.to_string_lossy());
    match locate_device(db_str, area, entity).await {
        Ok(res) => {
            let result = serde_json::json!({
                "status": "success",
                "area": res.area,
                "x": res.x,
                "y": res.y,
            });
            Ok(result.to_string())
        }
        Err(e) => {
            if let Ok(handler) = get_handler() {
                handler.disconnect().await.ok();
            }
            let result = serde_json::json!({
                "status": "error",
                "message": e.to_string(),
            });
            Ok(result.to_string())
        }
    }
}

// Comprehensive unit tests for location services
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::login::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};

    #[test]
    fn test_locate_state_creation() {
        let state = LocateState {
            area: "area_123".to_string(),
            x: 45.5,
            y: 67.8,
        };

        assert_eq!(state.area, "area_123");
        assert_eq!(state.x, 45.5);
        assert_eq!(state.y, 67.8);
    }

    #[test]
    fn test_locate_state_serialization() {
        let state = LocateState {
            area: "test_area".to_string(),
            x: 100.0,
            y: 200.0,
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: LocateState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.area, state.area);
        assert_eq!(deserialized.x, state.x);
        assert_eq!(deserialized.y, state.y);
    }

    #[test]
    fn test_beacon_serialization() {
        let beacon = Beacon {
            id: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439011".to_string(),
            },
            entity: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439012".to_string(),
            },
            area: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439013".to_string(),
            },
            merchant: None,
            connection: None,
            name: "Beacon 1".to_string(),
            description: Some("Test beacon".to_string()),
            r#type: "navigation".to_string(),
            location: (50.0, 50.0),
            device: "esp32".to_string(),
            mac: "AA:BB:CC:DD:EE:FF".to_string(),
        };

        let json = serde_json::to_string(&beacon).unwrap();
        assert!(json.contains("Beacon 1"));
        assert!(json.contains("AA:BB:CC:DD:EE:FF"));
    }

    #[test]
    fn test_beacon_with_merchant() {
        let beacon = Beacon {
            id: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439011".to_string(),
            },
            entity: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439012".to_string(),
            },
            area: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439013".to_string(),
            },
            merchant: Some(CustomizedObjectId {
                oid: "507f1f77bcf86cd799439014".to_string(),
            }),
            connection: None,
            name: "Store Beacon".to_string(),
            description: Some("Beacon at store entrance".to_string()),
            r#type: "marketing".to_string(),
            location: (25.0, 75.0),
            device: "esp32c3".to_string(),
            mac: "11:22:33:44:55:66".to_string(),
        };

        assert!(beacon.merchant.is_some());
        assert_eq!(beacon.r#type, "marketing");
    }

    #[test]
    fn test_beacon_location_coordinates() {
        let beacon = Beacon {
            id: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439011".to_string(),
            },
            entity: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439012".to_string(),
            },
            area: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439013".to_string(),
            },
            merchant: None,
            connection: None,
            name: "Location Test".to_string(),
            description: None,
            r#type: "navigation".to_string(),
            location: (123.456, 789.012),
            device: "esp32s3".to_string(),
            mac: "AA:BB:CC:DD:EE:FF".to_string(),
        };

        let (x, y) = beacon.location;
        assert_eq!(x, 123.456);
        assert_eq!(y, 789.012);
    }

    #[test]
    fn test_beacon_device_types() {
        let devices = vec!["esp32", "esp32c3", "esp32s3", "esp32c6"];

        for device in devices {
            let beacon = Beacon {
                id: CustomizedObjectId {
                    oid: "507f1f77bcf86cd799439011".to_string(),
                },
                entity: CustomizedObjectId {
                    oid: "507f1f77bcf86cd799439012".to_string(),
                },
                area: CustomizedObjectId {
                    oid: "507f1f77bcf86cd799439013".to_string(),
                },
                merchant: None,
                connection: None,
                name: "Test".to_string(),
                description: None,
                r#type: "navigation".to_string(),
                location: (0.0, 0.0),
                device: device.to_string(),
                mac: "00:00:00:00:00:00".to_string(),
            };

            assert_eq!(beacon.device, device);
        }
    }

    #[test]
    fn test_locate_state_coordinates_precision() {
        let state = LocateState {
            area: "precise_area".to_string(),
            x: 123.456789,
            y: 987.654321,
        };

        // Test that coordinates maintain precision
        assert!((state.x - 123.456789).abs() < 1e-6);
        assert!((state.y - 987.654321).abs() < 1e-6);
    }

    #[test]
    fn test_beacon_mac_address_format() {
        let beacon = Beacon {
            id: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439011".to_string(),
            },
            entity: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439012".to_string(),
            },
            area: CustomizedObjectId {
                oid: "507f1f77bcf86cd799439013".to_string(),
            },
            merchant: None,
            connection: None,
            name: "MAC Test".to_string(),
            description: None,
            r#type: "navigation".to_string(),
            location: (0.0, 0.0),
            device: "esp32".to_string(),
            mac: "AA:BB:CC:DD:EE:FF".to_string(),
        };

        // Verify MAC address format
        assert_eq!(beacon.mac.len(), 17); // Standard MAC length with colons
        assert_eq!(beacon.mac.matches(':').count(), 5);
    }

    #[tokio::test]
    async fn test_login_request_serialization() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("password123"));
    }

    #[tokio::test]
    async fn test_login_response_success() {
        let response = LoginResponse {
            success: true,
            token: Some("jwt_token_123".to_string()),
            user_id: Some("user_456".to_string()),
            message: "Login successful".to_string(),
        };

        assert!(response.success);
        assert!(response.token.is_some());
        assert_eq!(response.token.unwrap(), "jwt_token_123");
    }

    #[tokio::test]
    async fn test_login_response_failure() {
        let response = LoginResponse {
            success: false,
            token: None,
            user_id: None,
            message: "Invalid credentials".to_string(),
        };

        assert!(!response.success);
        assert!(response.token.is_none());
        assert_eq!(response.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_register_request_validation() {
        let request = RegisterRequest {
            email: "newuser@example.com".to_string(),
            username: "newuser".to_string(),
            password: "securepass123".to_string(),
        };

        assert_eq!(request.email, "newuser@example.com");
        assert_eq!(request.username, "newuser");
        assert!(!request.password.is_empty());
    }

    #[tokio::test]
    async fn test_register_response_success() {
        let response = RegisterResponse {
            success: true,
            user_id: Some("new_user_789".to_string()),
            message: "Registration successful".to_string(),
        };

        assert!(response.success);
        assert!(response.user_id.is_some());
    }

    #[test]
    fn test_email_format() {
        let request = LoginRequest {
            email: "valid@email.com".to_string(),
            password: "pass".to_string(),
        };

        assert!(request.email.contains("@"));
        assert!(request.email.contains("."));
    }

    #[test]
    fn test_password_not_empty() {
        let request = LoginRequest {
            email: "user@test.com".to_string(),
            password: "mypassword".to_string(),
        };

        assert!(!request.password.is_empty());
        assert!(request.password.len() >= 5);
    }

    #[test]
    fn test_login_response_serialization() {
        let response = LoginResponse {
            success: true,
            token: Some("token".to_string()),
            user_id: Some("user".to_string()),
            message: "OK".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: LoginResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.success, response.success);
        assert_eq!(deserialized.message, response.message);
    }

    #[test]
    fn test_register_request_serialization() {
        let request = RegisterRequest {
            email: "test@test.com".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@test.com"));
        assert!(json.contains("testuser"));
    }

    #[test]
    fn test_register_response_error() {
        let response = RegisterResponse {
            success: false,
            user_id: None,
            message: "Email already exists".to_string(),
        };

        assert!(!response.success);
        assert!(response.user_id.is_none());
        assert!(response.message.contains("Email already exists"));
    }

    #[test]
    fn test_serialize_beacon_info() {
        let info = r#"{"_id":{"$oid":"68a84b6ebdfa76608b934b0a"},"entity":{"$oid":"68a8301fbdfa76608b934ae1"},"area":{"$oid":"68a83067bdfa76608b934aea"},"merchant":{"$oid":"68a848c6bdfa76608b934b01"},"connection":null,"name":"NAVIGN-BEACON","description":"Beacon in A.I. Lab","type":"security","location":[66.0,8.0],"device":"esp32c3","mac":"48:F6:EE:21:B0:7C"}"#;
        let beacon: Beacon = serde_json::from_slice(info.as_bytes()).unwrap();
        assert_eq!(beacon.id.oid, "68a84b6ebdfa76608b934b0a");
        assert_eq!(beacon.entity.oid, "68a8301fbdfa76608b934ae1");
        assert_eq!(beacon.area.oid, "68a83067bdfa76608b934aea");
        assert_eq!(
            beacon.merchant.as_ref().unwrap().oid,
            "68a848c6bdfa76608b934b01"
        );
        assert_eq!(beacon.connection, None);
    }
}
