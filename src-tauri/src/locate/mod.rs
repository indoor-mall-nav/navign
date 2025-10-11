mod area;
mod beacon;
mod locator;
mod migration;
mod scan;
mod merchant;

use crate::locate::locator::LocateResult;
use crate::locate::scan::stop_scan;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocateState {
    pub area: String,
    pub x: f64,
    pub y: f64,
}

pub async fn locate_device(area: String) -> anyhow::Result<LocateState> {
    let conn = sqlx::SqlitePool::connect("sqlite:navign.db").await?;
    stop_scan()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to stop scan: {}", e))?;
    let devices = scan::scan_devices()
        .await
        .map_err(|e| anyhow::anyhow!("Scan error: {}", e))?;
    if devices.is_empty() {
        return Err(anyhow::anyhow!("No devices found"));
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

#[tauri::command]
pub async fn locate_handler(_app: AppHandle, area: String) -> Result<String, ()> {
    match locate_device(area).await {
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
