mod area;
mod beacon;
mod locator;
mod merchant;
mod migration;
mod scan;

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
use sqlx::SqlitePool;
use std::str::FromStr;
use tauri::AppHandle;
use tauri_plugin_blec::models::WriteType;
use tauri_plugin_blec::OnDisconnectHandler;
use tauri_plugin_http::reqwest;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocateState {
    pub area: String,
    pub x: f64,
    pub y: f64,
}

pub async fn locate_device(area: String, entity: String) -> anyhow::Result<LocateState> {
    let conn = SqlitePool::connect("sqlite:navign.db").await?;
    stop_scan()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to stop scan: {}", e))?;
    let devices = scan::scan_devices()
        .await
        .map_err(|e| anyhow::anyhow!("Scan error: {}", e))?;
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
            devices_unknown.push((device.address.clone(), device.rssi.unwrap()));
        }
    }
    devices_unknown.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by RSSI descending
    for (mac, _) in devices_unknown.iter() {
        if let Err(e) = fetch_device(&conn, mac, entity.as_str()).await {
            eprintln!("Failed to fetch device {}: {}", mac, e);
        }
    }
    let result = locator::handle_devices(devices, &conn, area.as_str()).await;
    match result {
        LocateResult::Success(x, y) => Ok(LocateState { area, x, y }),
        LocateResult::Error(err) => Err(anyhow::anyhow!("Locate error: {}", err)),
        LocateResult::NoBeacons => Err(anyhow::anyhow!("No beacons found")),
        LocateResult::AreaChanged(_) => {
            todo!("We need to nest the locate_device call here, after refreshing the area")
        }
        _ => unreachable!(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Beacon {
    #[serde(rename = "_id")]
    pub id: CustomizedObjectId,
    pub entity: CustomizedObjectId,
    pub area: CustomizedObjectId,
    pub merchant: Option<CustomizedObjectId>,
    pub connection: Option<CustomizedObjectId>,
    pub name: String,
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
        return Ok(());
    }

    let handler = tauri_plugin_blec::get_handler()
        .map_err(|e| anyhow::anyhow!("BLE not initialized: {}", e))?;

    handler
        .connect(mac, OnDisconnectHandler::None, true)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to device {}: {}", mac, e))?;

    let characteristic = Uuid::from_str(UNLOCKER_CHARACTERISTIC_UUID)?;
    let service = Uuid::from_str(UNLOCKER_SERVICE_UUID)?;

    handler
        .send_data(characteristic, Some(service), &[], WriteType::WithResponse)
        .await?;

    let received = handler.recv_data(characteristic, Some(service)).await?;
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

    let client = reqwest::Client::new();
    let url = format!("{BASE_URL}api/entities/{entity}/beacons/{object_id}");
    let res: Beacon = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
        .json()
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
            res.merchant
                .map(|m| m.to_string())
                .unwrap_or("unknown".into()),
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
    let res: ActiveArea = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

    if ActiveArea::get_by_id(conn, area)
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
        .is_none()
    {
        res.insert(conn).await?;
        println!("Area {} inserted into the database.", area);
    }

    let mut beacons_url = Some(format!("{}/beacons", url));

    while let Some(url) = beacons_url.as_deref() {
        let beacons: PaginationResponse<Beacon> = client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        for beacon in beacons.data.iter() {
            let beacon_info = BeaconInfo::new(
                beacon.id.to_string(),
                beacon.mac.clone(),
                beacon.location,
                beacon
                    .merchant
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or("unknown".into()),
                beacon.area.to_string(),
                entity.to_string(),
            );
            beacon_info.insert(conn).await?;
            println!("Beacon {} inserted/updated in the database.", beacon.id);
        }
        beacons_url = beacons.metadata.next_page_url;
    }

    Ok(())
}

#[tauri::command]
pub async fn locate_handler(_app: AppHandle, area: String, entity: String) -> Result<String, ()> {
    match locate_device(area, entity).await {
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
            let result = serde_json::json!({
                "status": "error",
                "message": e.to_string(),
            });
            Ok(result.to_string())
        }
    }
}
