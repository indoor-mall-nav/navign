# Localization Pipeline

The localization pipeline transforms raw Bluetooth Low Energy signal measurements into precise indoor position estimates. Unlike GPS-based outdoor positioning which relies on satellites, indoor localization faces unique challenges: multipath interference, signal attenuation through building materials, and human body blockage. The Navign system addresses these challenges through a multi-stage pipeline combining hardware signal processing, database caching, and algorithmic filtering.

## Pipeline Overview

The complete localization process involves coordination between mobile app, beacons, and server:

```
BLE Scan → Device Identification → Database Lookup → Area Detection → Trilateration → Position Output
```

Each stage handles specific aspects of the positioning problem, with failure modes and fallbacks designed to maintain degraded functionality when subsystems are unavailable.

## Stage 1: BLE Scanning and RSSI Measurement

The mobile application initiates localization by scanning for BLE advertisement packets from nearby beacons. This passive scanning requires no connection establishment, making it power-efficient and scalable.

**Scan Parameters:**
- Scan window: 100ms active, 400ms sleep (20% duty cycle)
- Scan duration: 2-5 seconds per localization request
- Filter: Service UUID `0x1819` (Location and Navigation)

Each discovered beacon yields:
- MAC address (48-bit hardware identifier)
- RSSI value (signal strength in dBm, typically -90 to -30)
- Advertisement payload (device name, service UUIDs)

**RSSI Characteristics:**

RSSI measurements exhibit significant variability even when the mobile device is stationary. A single beacon might report values ranging from -65 dBm to -75 dBm over consecutive packets. This variance stems from:

1. **Multipath Propagation**: Radio waves reflect off walls, floors, and furniture, creating constructive/destructive interference patterns
2. **Human Body Attenuation**: The human body is largely water, which absorbs 2.4 GHz signals. Holding the phone differently can change RSSI by 10+ dBm
3. **RF Noise**: Other 2.4 GHz devices (WiFi, microwaves, other BLE devices) create interference
4. **Fresnel Zone Obstruction**: Even partial obstruction of the line-of-sight path affects signal strength non-linearly

The mobile app collects multiple RSSI samples per beacon during the scan window, then applies filtering to reduce noise.

## Stage 2: Device Identification

MAC addresses alone don't provide position information. The mobile must map each MAC to a beacon database record containing location coordinates. This mapping occurs in two ways:

### First-Time Discovery

When encountering an unknown MAC address:

1. **GATT Connection**: Mobile connects to the beacon
2. **Device Query**: Sends `DeviceRequest` message
3. **Response Parsing**: Beacon returns:
   - Device type (Merchant, Pathway, Connection, Turnstile)
   - Capabilities bitmap (UnlockGate, EnvironmentalData)
   - Database ID (24-character hex ObjectId)
4. **Server Query**: `GET /api/entities/{entity}/beacons/{id}`
5. **Database Insert**: Cache beacon metadata in local SQLite

This handshake takes 200-500ms depending on BLE connection latency. For a typical deployment with 20-30 beacons per area, initial discovery might take 5-10 seconds. However, this occurs only once—the mapping is persisted.

### Cached Lookup

For known beacons, the mobile queries SQLite:

```sql
SELECT id, location_x, location_y, area FROM beacons WHERE mac = ?
```

This returns sub-millisecond, enabling real-time localization without server dependency.

**Cache Management:**

The beacon cache must handle:
- **MAC Address Changes**: If beacon hardware is replaced, the MAC changes but database ID remains constant
- **Position Updates**: Beacons might be physically relocated, requiring cache invalidation
- **Deletion**: Beacons removed from deployment must be purged from cache

The mobile implements TTL-based expiration: cached beacon records older than 24 hours are re-validated against the server during the next online sync.

## Stage 3: Area Detection

Before trilateration can occur, the system must determine which area the user occupies. Each beacon associates with exactly one area, so the mobile uses majority voting:

```
For each detected beacon:
    area_votes[beacon.area] += 1

current_area = area_with_most_votes
```

This simple algorithm handles edge cases where a user straddles area boundaries. If 5 beacons from Area A and 2 beacons from Area B are detected, the user is assumed to be in Area A (the 2 beacons from B are likely bleeding through doorways).

**Area Transition Detection:**

When the detected area differs from the previous localization result, the system triggers an area transition event:

1. Update current area in session state
2. Fetch area metadata from cache (polygon bounds, floor identifier)
3. Fetch area's beacon list to update expected beacons
4. Trigger UI update (change floor plan display)

This approach avoids ambiguity during transitions—the user "jumps" from one area to another atomically rather than gradually drifting across boundaries.

## Stage 4: RSSI Filtering and Distance Estimation

Raw RSSI values are too noisy for direct use. The mobile applies Kalman filtering to smooth measurements:

**Kalman Filter Model:**
```
State: [estimated_RSSI, rate_of_change]
Measurement: raw_RSSI_sample
Process noise: σ_process = 2 dBm (accounts for slow environmental changes)
Measurement noise: σ_measurement = 5 dBm (accounts for fast fluctuations)
```

The filter maintains state for each beacon, updating with each new RSSI sample. After 3-5 samples, the filtered estimate converges to a stable value with ±2 dBm accuracy (compared to ±10 dBm for raw measurements).

**Distance Conversion:**

The filtered RSSI converts to distance via the log-distance path loss model:

```
RSSI = -10 * n * log10(d) + A

Where:
n = path loss exponent (calibrated per environment, typically 2.5-3.5)
A = RSSI at 1 meter reference distance (calibrated, typically -50 to -60 dBm)
d = distance in meters
```

Solving for d:
```
d = 10^((A - RSSI) / (10 * n))
```

**Calibration Process:**

The path loss parameters (n, A) are environment-specific. Calibration involves:
1. Place mobile at known distances from beacon (1m, 2m, 5m, 10m)
2. Record RSSI measurements at each distance
3. Perform least-squares regression to fit n and A
4. Store calibration parameters in area metadata

Without calibration, distance estimates can be off by 50-100%. With calibration, accuracy improves to ±1-2 meters in ideal conditions.

## Stage 5: Weighted Least Squares Trilateration

Given distance estimates to multiple beacons with known positions, trilateration calculates the user's position. The problem is overdetermined (more beacons than spatial dimensions), allowing weighted least squares to find the best-fit position.

**Mathematical Formulation:**

For beacon i at position (x_i, y_i) with estimated distance d_i:
```
(x - x_i)² + (y - y_i)² = d_i²
```

This system of equations is non-linear. The mobile linearizes using Taylor expansion around an initial guess, then iterates to convergence:

```
Iteration k:
  For each beacon i:
    predicted_distance = sqrt((x_k - x_i)² + (y_k - y_i)²)
    error = measured_distance - predicted_distance
    jacobian = [(x_k - x_i) / predicted_distance,
                (y_k - y_i) / predicted_distance]

  Solve: J^T W J Δx = J^T W error
  Update: x_(k+1) = x_k + Δx
```

Where:
- J is the Jacobian matrix (partial derivatives)
- W is a diagonal weight matrix (inverse variance of distance estimates)
- Δx is the position correction

**Weighting Strategy:**

Not all beacons contribute equally. Closer beacons have more accurate RSSI→distance mappings, so they receive higher weights:

```
weight_i = 1 / (1 + d_i²)
```

This quadratic decay ensures distant beacons (with large errors) don't dominate the solution.

**Convergence:**

The algorithm iterates until position change falls below 0.1 meters or 10 iterations complete. In practice, convergence occurs within 3-5 iterations for typical beacon configurations.

## Stage 6: Polygon Constraint Enforcement

Trilateration alone can produce positions outside area boundaries (due to RSSI errors). The mobile enforces geometric constraints using the area's polygon bounds.

**Boundary Checking:**

The polygon is represented as a sequence of vertices in WKT (Well-Known Text) format:
```
POLYGON((x1 y1, x2 y2, ..., xn yn, x1 y1))
```

Point-in-polygon testing uses the ray casting algorithm:
```
Cast a ray from the estimated position to infinity
Count how many polygon edges it crosses
If count is odd, point is inside; if even, point is outside
```

**Constraint Projection:**

If the trilateration result falls outside the polygon:
1. Find the nearest polygon edge
2. Project the position onto that edge
3. Move the position inward by 0.5 meters (to avoid boundary flickering)

This ensures all position estimates lie within navigable space, preventing nonsensical results like "user is inside a wall."

## Stage 7: Temporal Smoothing

Even after all filtering, position estimates exhibit small jumps between localization requests. The mobile applies temporal smoothing to produce smooth motion:

**Exponential Moving Average:**
```
smoothed_position = α * new_position + (1-α) * previous_smoothed_position

Where α = 0.3 (30% weight on new measurement, 70% on history)
```

This low-pass filter removes high-frequency jitter while allowing the position to track actual movement.

**Motion Detection:**

If the new position differs from the smoothed position by >3 meters, the system assumes actual motion (not just noise) and increases α to 0.7, allowing faster tracking. Once position stabilizes, α returns to 0.3.

## Failure Modes and Degradation

The pipeline includes several failure handling strategies:

**No Beacons Detected:**
- Return last known position with timestamp
- UI indicates "position unavailable"
- Navigation continues with outdated position

**Insufficient Beacons (<3):**
- Cannot perform trilateration
- Fall back to nearest beacon position
- Accuracy degrades to ~5-10 meters

**Database Cache Miss:**
- Attempt online server query
- If offline, exclude unknown beacons from trilateration
- Reduced accuracy but system remains functional

**Area Detection Ambiguity:**
- If vote counts are tied, use previous area
- Prevents oscillation between adjacent areas

## Performance Characteristics

On typical mobile hardware (iPhone 12, Pixel 6):
- BLE scan: 2-3 seconds
- Device identification (cached): <1ms
- Kalman filtering: ~5ms for 10 beacons
- Trilateration: 10-20ms (3-5 iterations)
- Total latency: 2.5-3.5 seconds from scan start to position output

The localization can run at up to 0.3 Hz continuous (limited by BLE scan duration), but the mobile typically triggers it every 5 seconds during active navigation to conserve battery.

**Accuracy:**
- Ideal conditions (open space, 6+ beacons): ±1-2 meters
- Typical conditions (hallways, 4-5 beacons): ±2-4 meters
- Poor conditions (corners, 2-3 beacons): ±5-10 meters

## Related Documentation

- [Mobile BLE Implementation](/components/mobile#localization-system)
- [Beacon Advertising Protocol](/components/beacon#advertising-mode)
- [Navigation Pipeline](/pipelines/navigation)
