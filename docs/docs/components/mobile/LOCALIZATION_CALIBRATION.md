# Beacon-Based Localization Calibration Guide

This guide provides step-by-step instructions for calibrating the RSSI-based indoor positioning system in your specific environment.

## Overview

The localization algorithm uses two key parameters that affect accuracy:
1. **TxPower**: Reference transmitted power at 1 meter (default: -59 dBm)
2. **Path Loss Exponent (n)**: Environment-specific signal propagation factor (default: 2.0)

For best results, these should be calibrated for your specific building and environment.

## Quick Start: TxPower Calibration

TxPower calibration is the easiest and fastest way to improve accuracy.

### Step 1: Prepare Equipment
- Navign mobile app (with localization enabled)
- Tape measure or laser rangefinder (accuracy to 0.5m)
- At least 3 calibration points per beacon
- Notebook to record measurements

### Step 2: Select Calibration Beacons

Choose 2-3 beacons in different areas of your facility:
- One in an open area (hallway/corridor)
- One in a constrained area (office, stairwell)
- One with potential obstacles (near metal, water)

### Step 3: Record RSSI at Known Distances

For each beacon:

1. **Close distance (1-2 meters)**
   - Stand 1 meter from beacon
   - Record app RSSI reading (average of 5 readings)
   - Stand 2 meters from beacon
   - Record RSSI reading

2. **Medium distance (5-10 meters)**
   - Stand 5 meters from beacon
   - Record RSSI reading
   - Stand 10 meters from beacon (if space allows)
   - Record RSSI reading

3. **Far distance (20-30 meters)**
   - Stand 20 meters from beacon
   - Record RSSI reading
   - Stand 30 meters if possible

Example table:
```
Beacon A (Hallway):
Distance (m) | RSSI (dBm) | Notes
1.0          | -58        | Clear line of sight
2.0          | -62        | Clear line of sight
5.0          | -72        | Clear line of sight
10.0         | -82        | Clear line of sight
20.0         | -92        | Clear line of sight
```

### Step 4: Calculate Reference TxPower

For each distance measurement:

```
TxPower = RSSI + 20 * n * log10(distance)

Where:
- RSSI = measured signal strength (negative, e.g., -58)
- n = 2.0 (use default for now)
- distance = meters from beacon
- log10 = logarithm base 10
```

**Example calculation:**
```
At 1 meter with RSSI -58:
TxPower = -58 + 20 * 2.0 * log10(1)
TxPower = -58 + 20 * 2.0 * 0
TxPower = -58 dBm

At 10 meters with RSSI -82:
TxPower = -82 + 20 * 2.0 * log10(10)
TxPower = -82 + 20 * 2.0 * 1.0
TxPower = -82 + 40
TxPower = -42 dBm (this seems off - indicates environment effect)
```

### Step 5: Update TxPower Value

If your calculated TxPower differs significantly from -59 dBm:

1. Open `mobile/src-tauri/src/locate/locator.rs`
2. Find the `rssi_to_distance()` function
3. Update the TxPower value:

```rust
fn rssi_to_distance(mut rssi: f64) -> f64 {
    let tx_power = -59.0;  // ← Change this value
    // ... rest of function
}
```

For example, if your calculations average to -55 dBm:
```rust
let tx_power = -55.0;  // Adjusted for your environment
```

4. Rebuild the mobile app:
```bash
cd mobile
pnpm run tauri build
```

### Step 6: Validate Improvements

Repeat your measurements with the updated TxPower:
- Positions should now be more accurate
- Distance estimates should match your measurements better
- Document the improvement for your records

## Advanced: Path Loss Exponent Calibration

If TxPower calibration alone doesn't achieve desired accuracy, calibrate the path loss exponent `n`.

### Theory

The path loss exponent depends on environment:

| Environment | n Value | Notes |
|-------------|---------|-------|
| Free space (outdoor) | 2.0 | Ideal, rarely occurs indoors |
| Open hallway | 2.0-2.3 | Clean line of sight, few obstacles |
| Typical office | 2.5-3.0 | Some walls, furniture |
| Dense environment | 3.0-3.5 | Many obstacles, people |
| Metal/water heavy | 3.5-4.5+ | Severe signal degradation |

### Measurement Procedure

1. **Collect distance-RSSI pairs** (minimum 8-10 pairs across different distances)

Example data:
```
Distance (m) | RSSI (dBm)
1.0          | -59
2.0          | -65
5.0          | -73
10.0         | -83
15.0         | -89
20.0         | -95
```

2. **Use regression analysis** to find optimal `n`:

The path loss formula rearranges to:
```
RSSI = TxPower - 20*n*log10(distance)
```

Which is linear: `RSSI = a - b*log10(distance)` where `b = 20*n`

Using linear regression (or spreadsheet):
- Plot distance vs RSSI
- Fit a line
- Calculate: `n = b / 20` where `b` is the slope magnitude

### Using a Spreadsheet

In Excel/Google Sheets:

1. Create columns:
   - A: Distance (meters)
   - B: RSSI (dBm)
   - C: log10(Distance) = LOG10(A1)

2. Use LINEST or SLOPE function:
   ```
   slope = SLOPE(B:B, C:C)
   n = slope / 20
   ```

3. Example with sample data:
   - slope = -40.3
   - n = 40.3 / 20 = 2.015 ≈ 2.0

If your n value differs from 2.0:

```rust
fn rssi_to_distance(mut rssi: f64) -> f64 {
    let tx_power = -59.0;
    if rssi > 0f64 {
        rssi = -rssi;
    }
    let n = 2.0;  // ← Update this value, e.g., 2.5, 3.0, etc.
    10f64.powf((tx_power - rssi) / (10.0 * n))
}
```

## Performance Validation

After calibration, measure accuracy in real-world conditions:

### Test Method 1: Fixed Position Accuracy

1. Stand at 10 known locations across your facility
2. Let the app localize for 10 seconds each (averaging multiple readings)
3. Record calculated position and actual position
4. Calculate error distance: `√((x_calc - x_actual)² + (y_calc - y_actual)²)`

**Target metrics:**
- Open space: ±1.5m (90% of measurements)
- Typical office: ±2.5m (90% of measurements)
- Dense areas: ±4m (90% of measurements)

### Test Method 2: Tracking Accuracy

1. Walk a known path (straight line, grid pattern)
2. Record GPS-like position trace from app
3. Compare against actual path
4. Calculate deviation from expected route

**Target metric:** Path deviation < 2m from actual route in typical areas

## Troubleshooting

### Problem: Positions are consistently too far from beacon

**Cause:** TxPower value is too high (too positive)

**Solution:**
- Decrease TxPower value (e.g., -59 → -62)
- This makes the algorithm think signals are weaker than they are
- Results in smaller distance estimates

### Problem: Positions are consistently too close to beacon

**Cause:** TxPower value is too low (too negative)

**Solution:**
- Increase TxPower value (e.g., -59 → -56)
- This increases distance estimates

### Problem: Accuracy varies greatly depending on direction

**Cause:** Environmental obstruction or multipath interference

**Solutions:**
- Add beacons to reduce reliance on any single beacon
- Check for metal structures, water features that block signals
- Increase path loss exponent `n` (e.g., 2.0 → 2.5)
- Deploy beacons in more locations to provide coverage from multiple directions

### Problem: Accuracy is poor in crowded areas

**Cause:** People absorb radio signals, reducing RSSI

**Solutions:**
- Increase path loss exponent `n` to 3.0-3.5
- This accounts for signal absorption
- Add additional beacons in high-traffic areas
- Consider time-of-day calibration (different `n` for peak vs off-peak)

### Problem: Positions jump around erratically

**Cause:** Insufficient beacons or multipath interference

**Solutions:**
- Verify at least 3 beacons are visible with RSSI > -100 dBm
- Check for reflective surfaces (mirrors, metal panels)
- Increase beacon density (space them 8-10m apart instead of 20m)
- In very noisy environments, consider hybrid approach using accelerometer/compass

## Environment-Specific Tuning

### Shopping Malls

- **Characteristics:** Open spaces, some obstacles (kiosks), metal structures
- **Recommended n:** 2.5-3.0
- **Beacon spacing:** 10-12m
- **TxPower:** Usually -59 to -56 dBm
- **Best location:** Center of corridors, at least 2m from walls

### Hospitals

- **Characteristics:** Many rooms with walls, metal equipment, high interference
- **Recommended n:** 3.0-3.5
- **Beacon spacing:** 8-10m (closer than usual)
- **TxPower:** -59 to -55 dBm
- **Challenges:** Elevator shafts block signals; place beacons outside elevators
- **Special case:** Operating rooms may need hardened shielding

### Airport Terminals

- **Characteristics:** Large open spaces, metal structures (gates, railings)
- **Recommended n:** 2.0-2.3
- **Beacon spacing:** 12-15m (can be larger due to open space)
- **TxPower:** Usually -59 to -56 dBm
- **Challenge:** Massive multipath due to metal; may need denser coverage

### Schools/Universities

- **Characteristics:** Mixed (classrooms + hallways), varying wall materials
- **Recommended n:** 2.5-3.0
- **Beacon spacing:** 8-10m
- **TxPower:** -59 to -56 dBm
- **Special case:** Stairwells often have poor coverage; place beacons on each floor landing

## Continuous Monitoring

### Metrics to Track

1. **Accuracy over time**: Re-test monthly to catch beacon failures
2. **RSSI distribution**: Verify beacons are still transmitting at expected power
3. **Coverage gaps**: Identify "dead zones" with no strong signals
4. **Beacon battery**: Monitor beacon power levels (indicates battery health)

### Automated Validation

Consider adding periodic validation runs:

```rust
#[test]
fn validate_calibration_accuracy() {
    // Known positions and expected RSSI ranges
    let test_cases = vec![
        ((0.0, 0.0), vec![(0.0, 0.0, -50.0)]),  // At beacon
        ((5.0, 0.0), vec![(0.0, 0.0, -73.0)]),  // 5m away
        ((0.0, 10.0), vec![(0.0, 0.0, -83.0)]), // 10m away
    ];

    for (expected, beacons) in test_cases {
        if let Some(actual) = locate_via_beacons(&beacons) {
            let error = distance(expected, actual);
            assert!(error < TOLERANCE);
        }
    }
}
```

## References

- Free Space Path Loss Model: [Friis Equation](https://en.wikipedia.org/wiki/Friis_transmission_equation)
- RSSI and Distance: [Bluetooth RSSI Blog](https://www.argenox.com/library/bluetooth-low-energy/rssi/)
- Multipath Fading: [IEEE Wireless Propagation Model](https://www.ieee802.org/)

## Summary

| Step | Effort | Impact | Complexity |
|------|--------|--------|-----------|
| TxPower calibration | 30 min | ±30% accuracy improvement | Easy |
| Path loss exponent (n) | 1-2 hours | Additional ±20% improvement | Medium |
| Beacon repositioning | Varies | Can double accuracy | Hard |
| Hybrid sensors (compass, accel) | 2-3 days | Smooth trajectories | Hard |

**Recommended approach:**
1. Start with TxPower calibration (easiest, high impact)
2. If needed, calibrate path loss exponent
3. If still insufficient, add more beacons or hybrid sensors
