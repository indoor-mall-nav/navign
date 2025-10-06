use crate::locate::beacon::BeaconInfo;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri_plugin_blec::models::BleDevice;

type Locator = (f64, f64, f64); // (x, y, rssi)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocateResult {
    Success(f64, f64),
    Forward,
    NoBeacons,
    AreaChanged(String),
    Error(String),
    Reserved,
}

fn count_effective_beacons(beacons: &[Locator]) -> usize {
    beacons
        .iter()
        .filter(|&&(_, _, rssi)| rssi >= -160.0)
        .count()
}

pub async fn handle_devices(
    devices: Vec<BleDevice>,
    pool: &SqlitePool,
    base: &str,
) -> LocateResult {
    let mut info = Vec::with_capacity(devices.len());
    for device in devices.iter() {
        if let Some(beacon_info) = BeaconInfo::get_from_mac(pool, &device.address)
            .await
            .ok()
            .flatten()
        {
            if device.rssi.is_some_and(|rssi| rssi >= -160) {
                info.push(beacon_info);
            }
        }
    }
    let groups = info.iter().zip(devices).chunk_by(|(i, _)| i.area.as_str());
    groups
        .into_iter()
        .map(|(id, group)| {
            (
                id,
                group
                    .filter_map(|(i, d)| {
                        let rssi = d.rssi? as f64;
                        let point = i.location()?;
                        Some((point.coord()?.x, point.coord()?.y, rssi))
                    })
                    .collect::<Vec<Locator>>(),
            )
        })
        .max_by_key(|(_, beacons)| count_effective_beacons(beacons))
        .map(|(area, beacons)| {
            if beacons.is_empty() {
                return LocateResult::NoBeacons;
            }
            if area != base {
                return LocateResult::AreaChanged(area.to_string());
            }
            match locate_via_beacons(&beacons) {
                Some((x, y)) => LocateResult::Success(x, y),
                None => LocateResult::Error("Failed to locate via beacons".to_string()),
            }
        })
        .unwrap_or_else(|| LocateResult::Error("No valid beacons found".to_string()))
}

fn rssi_to_distance(rssi: f64) -> f64 {
    // Using the formula: distance = 10 ^ ((TxPower - RSSI) / (10 * n))
    // where TxPower is the transmitted power in dBm (usually -59 dBm for BLE)
    // and n is the signal propagation constant (environment factor, typically between 2 and 4)
    let tx_power = -59.0; // Typical value for BLE
    let n = 2.0; // Free space
    10f64.powf((tx_power - rssi) / (10.0 * n))
}

/// # Locate via Beacons
///
/// 1. If there are RSSI values greater than -80 dBm, use the beacon with the highest RSSI value.
/// 2. If ALL RSSI values are within -80 dBm to -160 dBm, use the weighted area centroid method.
/// 3. Remove RSSI values less than -160 dBm.
pub fn locate_via_beacons(beacons: &[Locator]) -> Option<(f64, f64)> {
    if beacons.is_empty() {
        return None;
    }
    let strong_beacons: Vec<&Locator> = beacons
        .iter()
        .filter(|&&(_, _, rssi)| rssi > -80.0)
        .collect();
    if !strong_beacons.is_empty() {
        // Use the beacon with the highest RSSI value
        let &(x, y, _) = strong_beacons
            .iter()
            .max_by(|&&a, &&b| a.2.partial_cmp(&b.2).unwrap())
            .unwrap();
        return Some((*x, *y));
    }
    let filtered_beacons: Vec<&Locator> = beacons
        .iter()
        .filter(|&&(_, _, rssi)| rssi >= -160.0)
        .collect();
    if filtered_beacons.is_empty() {
        return None;
    }
    // Weighted area centroid method
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut total_weight = 0.0;
    for &&(x, y, rssi) in &filtered_beacons {
        let distance = rssi_to_distance(rssi);
        if distance > 0.0 {
            let weight = 1.0 / distance;
            sum_x += x * weight;
            sum_y += y * weight;
            total_weight += weight;
        }
    }
    if total_weight == 0.0 {
        return None;
    }
    Some((sum_x / total_weight, sum_y / total_weight))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_locate_via_beacons() {
        let beacons = vec![
            (0.0, 0.0, -70.0),
            (1.0, 1.0, -90.0),
            (2.0, 2.0, -85.0),
            (3.0, 3.0, -160.0),
            (4.0, 4.0, -170.0),
        ];
        let location = locate_via_beacons(&beacons);
        assert_eq!(location, Some((0.0, 0.0)));

        let beacons = vec![
            (1.0, 1.0, -90.0),
            (2.0, 2.0, -85.0),
            (3.0, 3.0, -160.0),
            (4.0, 4.0, -170.0),
        ];
        let location = locate_via_beacons(&beacons);
        assert!(location.is_some());
        let (x, y) = location.unwrap();
        assert!(x > 1.5 && x < 2.5);
        assert!(y > 1.5 && y < 2.5);

        let beacons = vec![(3.0, 3.0, -161.0), (4.0, 4.0, -170.0)];
        let location = locate_via_beacons(&beacons);
        assert_eq!(location, None);

        let beacons: Vec<Locator> = vec![];
        let location = locate_via_beacons(&beacons);
        assert_eq!(location, None);
    }
}
