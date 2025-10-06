mod area;
mod beacon;
mod locator;
mod migration;
mod scan;

use serde::{Deserialize, Serialize};
use crate::locate::locator::LocateResult;
use crate::locate::scan::stop_scan;

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
        LocateResult::Success(x, y) => Ok(LocateState {
            area,
            x,
            y,
        }),
        LocateResult::Error(err) => Err(anyhow::anyhow!("Locate error: {}", err)),
        LocateResult::NoBeacons => Err(anyhow::anyhow!("No beacons found")),
        LocateResult::AreaChanged(_) => {
            todo!("We need to nest the locate_device call here, after refreshing the area")
        }
        _ => unreachable!(),
    }
}
