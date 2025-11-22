mod locator;
mod migration;
pub(crate) mod scan;

use crate::api::map::{Area, Beacon};
use crate::api::page_results::PaginationResponse;
use crate::locate::locator::LocateResult;
use crate::locate::scan::stop_scan;
use crate::shared::BASE_URL;
use crate::unlocker::constants::{UNLOCKER_CHARACTERISTIC_UUID, UNLOCKER_SERVICE_UUID};
use navign_shared::IntRepository;
use navign_shared::{BleMessage, Depacketize, Packetize};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{SqlitePool, migrate};
use std::str::FromStr;
use tauri::{AppHandle, Manager};
use tauri_plugin_blec::models::{BleDevice, WriteType};
use tauri_plugin_blec::{OnDisconnectHandler, get_handler};
use tauri_plugin_http::reqwest;
use tauri_plugin_log::log::{error, info};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocateState {
    pub area: i32,
    pub x: f64,
    pub y: f64,
}

pub async fn locate_device(dbpath: String, area: i32, entity: Uuid) -> anyhow::Result<LocateState> {
    info!("Locating device in area: {}, entity: {}", area, entity);
    info!("Connecting to database at: {}", dbpath);
    let options = SqliteConnectOptions::new()
        .create_if_missing(true)
        .filename(dbpath.as_str());
    let conn = SqlitePool::connect_with(options).await.map_err(|e| {
        error!("Database connection error: {}", e);
        anyhow::anyhow!("Failed to connect to database at {}: {}", dbpath, e)
    })?;
    info!("Database connected.");
    migrate!("./migrations")
        .run(&conn)
        .await
        .map_err(|e| anyhow::anyhow!("Database migration error: {}", e))?;
    info!("Starting BLE scan...");
    stop_scan()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to stop scan: {}", e))?;
    info!("Previous scan stopped. Now scanning for devices...");
    let mut devices = scan::scan_devices(false)
        .await
        .map_err(|e| anyhow::anyhow!("Scan error: {}", e))?;
    info!("Scan completed. Found {} devices.", devices.len());
    if devices.is_empty() {
        return Err(anyhow::anyhow!("No devices found"));
    }
    for device in devices.iter_mut() {
        info!("Fetching info for unknown device: {}", device.address);
        match fetch_device(&conn, device.address.as_str(), entity).await {
            Ok(object_id) => {
                device.address = object_id.to_string();
                info!("Updated device address to object ID: {}", device.address);
            }
            Err(e) => {
                error!(
                    "Failed to fetch device info for MAC {}: {}",
                    device.address, e
                );
            }
        }
    }
    let devices: Vec<BleDevice> = devices
        .into_iter()
        .filter(|d| d.address != "NAVIGN-BEACON")
        .collect();
    let result = locator::handle_devices(devices.clone(), &conn, area, entity).await;
    match result {
        LocateResult::Success(x, y) => Ok(LocateState { area, x, y }),
        LocateResult::Error(err) => Err(anyhow::anyhow!("Locate error: {}", err)),
        LocateResult::NoBeacons => Err(anyhow::anyhow!("No beacons found")),
        LocateResult::AreaChanged(new_area) => {
            info!("Area changed, updating area info...");
            update_area(&conn, new_area, entity).await?;
            match locator::handle_devices(devices, &conn, new_area, entity).await {
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

pub async fn fetch_device(conn: &SqlitePool, mac: &str, entity: Uuid) -> anyhow::Result<i32> {
    // It might be updated, so we need to check it in local database first.
    let addr = Beacon::search(conn, mac, false, 0, 1, None, false, entity)
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {}", e))?;

    if addr.len() == 1 {
        info!(
            "Device with MAC {} found in local database with ID {}.",
            mac, addr[0].id
        );
        return Ok(addr[0].id);
    }

    info!("Connecting to device with MAC: {}", mac);

    let object_id = {
        let handler = get_handler().map_err(|e| anyhow::anyhow!("BLE not initialized: {}", e))?;

        handler
            .connect(mac, OnDisconnectHandler::None, true)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to device {}: {}", mac, e))?;

        info!("Connected to device: {}", mac);

        let characteristic = Uuid::from_str(UNLOCKER_CHARACTERISTIC_UUID)?;
        let service = Uuid::from_str(UNLOCKER_SERVICE_UUID)?;

        handler
            .subscribe(characteristic, Some(service), |data| {
                info!("Notification received: {:x?}", data);
            })
            .await?;

        info!("Subscribed to characteristic: {}", characteristic);

        handler
            .send_data(
                characteristic,
                Some(service),
                &BleMessage::DeviceRequest.packetize(),
                WriteType::WithoutResponse,
            )
            .await?;
        info!("Sent device request to {}", mac);
        let received = handler.recv_data(characteristic, Some(service)).await?;
        info!("Received data from device {}: {:x?}", mac, received);
        handler.unsubscribe(characteristic).await?;
        handler.disconnect().await?;
        let depacketized = BleMessage::depacketize(received.as_slice())
            .ok_or_else(|| anyhow::anyhow!("Failed to depacketize device response"))?;

        info!("Depacketized message: {:?}", depacketized);

        let BleMessage::DeviceResponse(_, _, obj_id) = depacketized else {
            return Err(anyhow::anyhow!("Failed to extract device response"));
        };

        obj_id
    };

    if let Some(beacon) = Beacon::get_by_id(conn, object_id, entity)
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
    {
        info!(
            "Beacon with ID {} already exists in the database.",
            object_id
        );
        if Area::get_by_id(conn, beacon.area_id, entity)
            .await
            .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
            .is_none()
        {
            update_area(conn, beacon.area_id, entity).await?;
        }
    } else {
        info!("Fetched object ID: {} for MAC: {}", object_id, mac);

        let client = reqwest::Client::new();
        let url = format!("{BASE_URL}api/entities/{entity}/beacons/{object_id}");
        info!("Fetching beacon info from URL: {}", url);

        let res = client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
            .json::<Beacon>()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        if Area::get_by_id(conn, res.area_id, entity)
            .await
            .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
            .is_some()
        {
            Beacon::create(conn, &res, entity).await?;
            info!("Beacon {} inserted/updated in the database.", res.id);
        } else {
            update_area(conn, res.area_id, entity).await?;
        }
    };

    Ok(object_id)
}

async fn update_area(conn: &SqlitePool, area: i32, entity: Uuid) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{BASE_URL}api/entities/{entity}/areas/{area}");
    info!("Fetching area info from URL: {}", url);
    let res: Area = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

    info!("Fetched area id: {}", res.id);

    if Area::get_by_id(conn, area, entity)
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
        .is_none()
    {
        Area::create(conn, &res, entity).await?;
        info!("Area {} inserted into the database.", area);
    }

    let mut beacons_url = Some(format!("{}/beacons", url));

    while let Some(url) = beacons_url.as_deref() {
        info!("Fetching beacons from URL: {}", url);
        let beacons: PaginationResponse<Beacon> = client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        info!("Fetched {} beacons for area {}", beacons.data.len(), area);

        for beacon in beacons.data.iter() {
            info!("Processing beacon ID: {}, MAC: {}", beacon.id, beacon.mac);
            if Beacon::get_by_id(conn, beacon.id, entity)
                .await
                .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
                .is_some()
            {
                info!(
                    "Beacon with ID {} already exists in the database.",
                    beacon.id
                );
                Beacon::update(conn, beacon, entity).await?;
            } else {
                info!(
                    "Inserting new beacon with ID {} into the database.",
                    beacon.id
                );
                Beacon::create(conn, beacon, entity).await?;
            }
            info!("Beacon {} inserted/updated in the database.", beacon.id);
        }
        beacons_url = beacons.metadata.next_page_url;
    }

    info!("All beacons for area {} have been processed.", area);

    Ok(())
}

#[tauri::command]
pub async fn locate_handler(app: AppHandle, area: String, entity: String) -> Result<String, ()> {
    let area: i32 = area.parse().map_err(|e| {
        error!("Invalid area parameter: {}", e);
    })?;
    let entity: Uuid = Uuid::parse_str(&entity).map_err(|e| {
        error!("Invalid entity parameter: {}", e);
    })?;
    let dbpath = app
        .path()
        .app_local_data_dir()
        .map(|p| p.join("navign.db"))
        .map_err(|e| {
            error!("Failed to get app local data dir: {}", e);
        })?;
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(dbpath.parent().unwrap()).map_err(|e| {
        error!("Failed to create app local data dir: {}", e);
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
