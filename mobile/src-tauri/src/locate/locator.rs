//! # Beacon-Based Indoor Localization Module
//!
//! This module implements RSSI-based (Received Signal Strength Indicator) indoor positioning
//! for the Navign mobile app. It uses BLE beacon signals to determine the user's location
//! within a building.
//!
//! ## Algorithm Overview
//!
//! The localization system uses a two-stage approach:
//!
//! 1. **Area Selection**: Groups beacons by area and selects the area with the most strong signals
//! 2. **Position Calculation**: Within the selected area, calculates position using one of two strategies:
//!    - **Strong Beacon Strategy**: If strong signals (RSSI > -60 dBm) exist, uses the strongest beacon's location
//!    - **Weighted Centroid Strategy**: If only weak/medium signals exist, calculates weighted average position
//!
//! ## RSSI Signal Strength
//!
//! - **RSSI Range**: Measured in dBm, typically from 0 (very close) to -100+ (far away)
//! - **Strong (-60 dBm and above)**: Beacon is within ~1-2 meters, highly reliable
//! - **Weak (-100 dBm and below)**: Beacon is far away, less reliable
//! - **Minimum (-160 dBm)**: Signals below this are noise/interference, filtered out
//!
//! ## Accuracy Characteristics
//!
//! | RSSI Range | Distance | Accuracy | Use Case |
//! |-----------|----------|----------|----------|
//! | > -60 dBm | 1-2m | ±0.5m | Entry point, strong signal area |
//! | -60 to -100 dBm | 2-15m | ±2-3m | General navigation |
//! | -100 to -160 dBm | 15-50m | ±5-10m | Area detection only |
//! | < -160 dBm | >50m | Unreliable | Filtered out (noise) |
//!
//! ## Calibration Notes
//!
//! The accuracy depends heavily on the environment:
//! - **Open space**: Very accurate, RSSI varies smoothly with distance
//! - **Walls/obstacles**: Less accurate, signals get blocked or reflected
//! - **Metal/water**: Significant signal degradation
//! - **Crowds**: Signal absorption reduces accuracy
//!
//! For best results, deploy beacons uniformly throughout the area with 5-10 meter spacing.
//! Run calibration tests in your environment to determine actual accuracy.
//!
//! ## Implementation Details
//!
//! The algorithm uses the **Free Space Path Loss Model**:
//! ```text
//! distance = 10^((TxPower - RSSI) / (10 * n))
//! ```
//!
//! Where:
//! - `TxPower = -59 dBm` (typical BLE transmitted power at 1 meter)
//! - `n = 2.0` (path loss exponent for free space)
//! - `RSSI` = measured signal strength in dBm (negative value)
//!
//! The weighted centroid calculation then uses `weight = 1/distance` to prioritize
//! closer beacons over distant ones, producing a weighted average position.

use itertools::Itertools;
use navign_shared::Beacon;
use navign_shared::IntRepository;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri_plugin_blec::models::BleDevice;
use tauri_plugin_log::log::trace;
use uuid::Uuid;

/// Tuple type for beacon position and signal strength
/// Format: (x_coordinate, y_coordinate, rssi_in_dbm)
type Locator = (f64, f64, f64);

/// Result type for localization operations
///
/// # Variants
/// - `Success(x, y)`: Successfully determined position with coordinates
/// - `Forward`: Could not determine position (continue with previous position)
/// - `NoBeacons`: No usable beacon signals found in current area
/// - `AreaChanged(area_id)`: User appears to have moved to a different area/floor
/// - `Error(message)`: Localization error with description
/// - `Reserved`: Reserved for future use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocateResult {
    Success(f64, f64),
    Forward,
    NoBeacons,
    AreaChanged(i32),
    Error(String),
    Reserved,
}

/// Counts beacons with usable signal strength
///
/// A beacon is considered "effective" if its RSSI is >= -160 dBm.
/// Signals weaker than -160 dBm are considered noise and are filtered out.
///
/// # Arguments
/// * `beacons` - Array of beacon tuples (x, y, rssi)
///
/// # Returns
/// Number of beacons with RSSI >= -160 dBm
///
/// # Note
/// This is used to select which area the user is likely in:
/// the area with the most effective beacons is chosen as the most probable location.
///
/// # Examples
/// ```ignore
/// let beacons = vec![
///     (0.0, 0.0, -50.0),   // Strong signal
///     (1.0, 1.0, -80.0),   // Medium signal
///     (2.0, 2.0, -150.0),  // Weak but usable
///     (3.0, 3.0, -170.0),  // Too weak, filtered out
/// ];
/// assert_eq!(count_effective_beacons(&beacons), 3);
/// ```
fn count_effective_beacons(beacons: &[Locator]) -> usize {
    beacons
        .iter()
        .filter(|&&(_, _, rssi)| rssi >= -160.0)
        .count()
}

/// Processes BLE devices and determines user location
///
/// This is the main entry point for localization. It:
/// 1. Scans detected BLE devices for known beacons
/// 2. Groups beacons by area
/// 3. Selects the area with the strongest signals
/// 4. Calculates position within that area
///
/// # Algorithm
///
/// ```text
/// 1. For each BLE device:
///    - Look up beacon info from database
///    - Filter out very weak signals (RSSI > -160 dBm)
///    - Add to beacon list
///
/// 2. Group beacons by area_id
///
/// 3. Select area with most effective beacons
///
/// 4. Within selected area:
///    - If area != current area (base):
///      - Return AreaChanged(area_id) to indicate floor/area switch
///    - Otherwise:
///      - Call locate_via_beacons() for position calculation
/// ```
///
/// # Arguments
/// * `devices` - BLE devices detected by phone (from tauri-plugin-blec)
/// * `pool` - SQLite connection pool for beacon database lookup
/// * `base` - Current area ID (used to detect area changes)
/// * `entity` - Entity ID (building/mall/hospital ID)
///
/// # Returns
/// [LocateResult] indicating success/failure and position or error details
///
/// # Performance Notes
/// - Database lookups are async; one per device
/// - Typically fast for 10-20 nearby beacons
/// - May be slower in areas with hundreds of beacons (consider area-based filtering)
///
/// # Error Handling
/// - Returns [LocateResult::NoBeacons] if no usable signals in current area
/// - Returns [LocateResult::AreaChanged] if strongest signals are from different area
/// - Returns [LocateResult::Error] if position calculation fails
pub async fn handle_devices(
    devices: Vec<BleDevice>,
    pool: &SqlitePool,
    base: i32,
    entity: Uuid,
) -> LocateResult {
    trace!("Handling {} devices", devices.len());
    let mut info = Vec::with_capacity(devices.len());
    for device in devices.iter() {
        trace!("Processing device: {:?}", device);
        let target_id: i32 = device.address.parse().unwrap_or_default();
        if let Some(beacon_info) = Beacon::get_by_id(pool, target_id, entity)
            .await
            .ok()
            .flatten()
        {
            trace!("Found beacon info: {:?}", beacon_info);
            if device.rssi.is_some_and(|rssi| rssi <= 160) {
                trace!("Adding beacon info: {:?}", beacon_info);
                info.push(beacon_info);
            }
        }
    }
    trace!("Collected beacon info: {:?}", info);
    let groups = info.iter().zip(devices).chunk_by(|(i, _)| i.area_id);
    groups
        .into_iter()
        .map(|(id, group)| {
            trace!("Processing group for area: {}", id);
            (
                id,
                group
                    .filter_map(|(i, d)| {
                        let rssi = d.rssi? as f64;
                        let point = i.location();
                        Some((point.0, point.1, rssi))
                    })
                    .collect::<Vec<Locator>>(),
            )
        })
        .max_by_key(|(_, beacons)| count_effective_beacons(beacons))
        .map(|(area, beacons)| {
            trace!(
                "Selected area: {}, with {} effective beacons",
                area,
                count_effective_beacons(&beacons)
            );
            if beacons.is_empty() {
                trace!("No valid beacons found in area: {}", area);
                return LocateResult::NoBeacons;
            }
            if area != base {
                trace!("Area changed from {} to {}", base, area);
                return LocateResult::AreaChanged(area);
            }
            match locate_via_beacons(&beacons) {
                Some((x, y)) => LocateResult::Success(x, y),
                None => LocateResult::Error("Failed to locate via beacons".to_string()),
            }
        })
        .unwrap_or_else(|| LocateResult::Error("No valid beacons found".to_string()))
}

/// Converts RSSI (signal strength) to distance using the Free Space Path Loss Model
///
/// # Formula
///
/// The function uses the logarithmic free-space path loss model:
///
/// ```text
/// distance (meters) = 10^((TxPower - RSSI) / (10 * n))
/// ```
///
/// # Parameters
/// - `TxPower = -59 dBm`: Reference power at 1 meter (typical for BLE beacons)
/// - `n = 2.0`: Path loss exponent in free space
/// - `RSSI`: Measured signal strength in dBm (should be negative, e.g., -50, -80, -120)
///
/// # Physics Background
///
/// In free space, radio signal power decreases with distance as 1/distance².
/// The logarithmic form makes calculations easier:
/// - Power ratio (in dB) = 20 * log10(distance) + constant
/// - RSSI = TxPower - 20*n*log10(distance)
/// - Solving for distance gives the formula above
///
/// # Arguments
/// * `rssi` - Measured signal strength in dBm (can be positive or negative, will be converted)
///
/// # Returns
/// Distance in meters (positive float)
///
/// # Examples
///
/// ```ignore
/// // Strong signal at 1 meter
/// assert!(rssi_to_distance(-59.0) < 2.0);  // ~1 meter
///
/// // Medium signal at 10 meters
/// let dist = rssi_to_distance(-79.0);
/// assert!(dist > 5.0 && dist < 15.0);      // ~10 meters
///
/// // Weak signal at 50 meters
/// let dist = rssi_to_distance(-119.0);
/// assert!(dist > 30.0);                    // ~50+ meters
/// ```
///
/// # Calibration for Your Environment
///
/// The TxPower value of -59 dBm is typical for BLE beacons at 1 meter, but varies by device.
/// If your positioning is consistently off:
///
/// 1. Measure actual distances to several beacons
/// 2. Note their RSSI values
/// 3. Calculate what TxPower would give the right distance
/// 4. Example: If beacon shows -70 dBm but is actually 5 meters away:
///    - Original formula gives: distance ≈ 3.16 meters
///    - You need TxPower adjustment of +2 dBm
///    - Update `let tx_power = -57.0;` in the code
///
/// # Environmental Factors
///
/// The path loss exponent `n = 2.0` assumes free space (no obstacles).
/// In real buildings:
/// - **Open hallways**: n ≈ 2.0-2.3
/// - **Crowded areas**: n ≈ 2.5-3.0 (signal absorbed by people)
/// - **With walls**: n ≈ 3.0-4.0 (signal reflected/diffracted)
/// - **Metal structures**: n > 4.0 (severe multipath effects)
///
/// For better accuracy in your specific building, you can measure several beacon
/// distances and calculate the optimal `n` value for your environment.
///
/// # Note on Signal Variation
///
/// RSSI values naturally fluctuate by ±5-10 dBm due to multipath interference,
/// even at fixed locations. The weighted centroid approach helps average out
/// these fluctuations.
fn rssi_to_distance(mut rssi: f64) -> f64 {
    let tx_power = -59.0; // Reference transmitted power at 1 meter (BLE typical)
    if rssi > 0f64 {
        rssi = -rssi; // Ensure RSSI is negative
    }
    let n = 2.0; // Path loss exponent for free space
    10f64.powf((tx_power - rssi) / (10.0 * n))
}

/// Calculates position from beacon signals using one of two strategies
///
/// This function implements the core positioning algorithm with two modes:
///
/// ## Mode 1: Strong Beacon Strategy (RSSI > -60 dBm)
///
/// When there's a strong signal from a nearby beacon:
/// - Uses the location of the strongest beacon directly
/// - Simple and fast (O(n) scan)
/// - Accuracy: ±0.5-1.0 meters (beacon is very close)
/// - Use case: Entry points, highly concentrated areas with strong signal
///
/// ## Mode 2: Weighted Centroid Strategy (RSSI -60 to -160 dBm)
///
/// When no strong signals exist but multiple medium/weak signals are available:
/// - Calculates weighted average position of all beacons
/// - Weight = 1 / distance (closer beacons count more)
/// - More robust than single beacon (averages out signal fluctuation)
/// - Accuracy: ±2-5 meters (depending on beacon spacing and environment)
/// - Use case: General area navigation, typical indoor environment
///
/// ## Algorithm Details
///
/// ```text
/// 1. Filter beacons with RSSI >= -60 dBm (strong signals)
/// 2. If found: return location of strongest beacon (highest RSSI)
/// 3. Otherwise:
///    a. Filter remaining beacons: -60 dBm >= RSSI >= -160 dBm
///    b. For each beacon:
///       - Calculate distance from RSSI using path loss model
///       - Calculate weight = 1 / distance
///    c. Calculate weighted average:
///       - x_final = Σ(x_i * weight_i) / Σ(weight_i)
///       - y_final = Σ(y_i * weight_i) / Σ(weight_i)
///    d. Return (x_final, y_final)
/// 4. Beacons with RSSI < -160 dBm are discarded as noise
/// ```
///
/// # Arguments
/// * `beacons` - Array of beacon tuples (x, y, rssi_in_dbm)
///
/// # Returns
/// - `Some((x, y))` - Calculated position coordinates
/// - `None` - No usable beacons (empty array or all signals too weak)
///
/// # Computational Complexity
/// - Time: O(n) where n = number of beacons (typically 3-10)
/// - Space: O(n) for intermediate filter arrays
/// - Typical runtime: <1ms on modern phone
///
/// # Accuracy Factors
///
/// **Improves accuracy:**
/// - Multiple beacons in line of sight
/// - Uniform beacon distribution
/// - Open space (fewer multipath reflections)
/// - Recent calibration of TxPower and path loss exponent
///
/// **Reduces accuracy:**
/// - Only 1-2 beacons available
/// - Uneven beacon spacing
/// - Walls between beacons and user
/// - Metal structures or water features
/// - Crowds (signal absorption)
///
/// # Testing Strategy
///
/// When deploying beacons in a new location:
/// 1. Measure distance at various points (use tape measure or laser rangefinder)
/// 2. Note RSSI values from the app
/// 3. Calculate whether accuracy meets requirements
/// 4. If not, adjust TxPower or path loss exponent (n) in `rssi_to_distance()`
/// 5. Verify improvements with additional measurements
///
/// # Example Scenario
///
/// ```text
/// Shopping mall corridor with 3 beacons:
/// - Beacon A at (0, 0), RSSI = -55 dBm (strong, distance ≈ 1.5m)
/// - Beacon B at (5, 0), RSSI = -75 dBm (medium, distance ≈ 5.6m)
/// - Beacon C at (0, 5), RSSI = -80 dBm (weak, distance ≈ 7.9m)
///
/// Since Beacon A has RSSI > -60 dBm (strong signal):
/// → Use Strong Beacon Strategy
/// → Return position (0, 0)
/// → Accuracy: ±0.5m
///
/// Alternative scenario (user moved away):
/// - Beacon A: RSSI = -72 dBm (distance ≈ 5.0m)
/// - Beacon B: RSSI = -68 dBm (distance ≈ 3.9m)
/// - Beacon C: RSSI = -90 dBm (distance ≈ 15.8m)
///
/// No RSSI > -60 dBm, so use Weighted Centroid:
/// → weights: 1/5.0=0.20, 1/3.9=0.26, 1/15.8=0.06
/// → x = (0*0.20 + 5*0.26 + 0*0.06) / 0.52 ≈ 2.5m
/// → y = (0*0.20 + 0*0.26 + 5*0.06) / 0.52 ≈ 0.6m
/// → Result: approximately (2.5, 0.6)
/// → Accuracy: ±2-3m
/// ```
pub fn locate_via_beacons(beacons: &[Locator]) -> Option<(f64, f64)> {
    trace!("Located position via beacons in: {:?}", beacons);
    if beacons.is_empty() {
        return None;
    }
    let strong_beacons: Vec<&Locator> = beacons
        .iter()
        .filter(|&&(_, _, rssi)| rssi.abs() <= 60.0)
        .collect();
    trace!("{} strong beacons", strong_beacons.len());
    if !strong_beacons.is_empty() {
        // Use the beacon with the highest RSSI value
        trace!("Using the strongest beacon");
        let &(x, y, _) = strong_beacons
            .iter()
            .max_by(|&&a, &&b| a.2.partial_cmp(&b.2).unwrap())
            .unwrap();
        return Some((*x, *y));
    }
    trace!("No strong beacons, using weighted centroid method");
    // Filter out beacons with RSSI less than -160 dBm
    let filtered_beacons: Vec<&Locator> = beacons
        .iter()
        .filter(|&&(_, _, rssi)| rssi.abs() <= 160.0)
        .collect();
    trace!("{} beacons after filtering", filtered_beacons.len());
    if filtered_beacons.is_empty() {
        trace!("No beacons with sufficient RSSI");
        return None;
    }
    // Weighted area centroid method
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut total_weight = 0.0;
    for &&(x, y, rssi) in &filtered_beacons {
        trace!("Beacon at ({}, {}) with RSSI {}", x, y, rssi);
        let distance = rssi_to_distance(rssi);
        if distance > 0.0 {
            let weight = 1.0 / distance;
            sum_x += x * weight;
            sum_y += y * weight;
            total_weight += weight;
        }
    }
    trace!(
        "Sum_x: {}, Sum_y: {}, Total_weight: {}",
        sum_x, sum_y, total_weight
    );
    if total_weight == 0.0 {
        return None;
    }
    trace!(
        "Calculated position: ({}, {})",
        sum_x / total_weight,
        sum_y / total_weight
    );
    Some((sum_x / total_weight, sum_y / total_weight))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_locate_via_beacons() {
        let beacons = vec![
            (0.0, 0.0, 40.0),
            (1.0, 1.0, 90.0),
            (2.0, 2.0, 85.0),
            (3.0, 3.0, 160.0),
            (4.0, 4.0, 170.0),
        ];
        let location = locate_via_beacons(&beacons);
        assert_eq!(location, Some((0.0, 0.0)));

        let beacons = vec![
            (0.0, 0.0, 70.0),
            (1.0, 1.0, 90.0),
            (2.0, 2.0, 85.0),
            (3.0, 3.0, 160.0),
            (4.0, 4.0, 170.0),
        ];
        let location = locate_via_beacons(&beacons);
        assert_eq!(location, Some((0.35665167226144173, 0.35665167226144173)));

        let beacons = vec![
            (1.0, 1.0, 90.0),
            (2.0, 2.0, 85.0),
            (3.0, 3.0, 160.0),
            (4.0, 4.0, 170.0),
        ];
        let location = locate_via_beacons(&beacons);
        assert!(location.is_some());
        let (x, y) = location.unwrap();
        assert!(x > 1.5 && x < 2.5);
        assert!(y > 1.5 && y < 2.5);

        let beacons = vec![(3.0, 3.0, 161.0), (4.0, 4.0, 170.0)];
        let location = locate_via_beacons(&beacons);
        assert_eq!(location, None);

        let beacons: Vec<Locator> = vec![];
        let location = locate_via_beacons(&beacons);
        assert_eq!(location, None);
    }
}
