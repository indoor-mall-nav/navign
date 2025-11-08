# Mobile Application

The Navign mobile application represents a sophisticated cross-platform solution for indoor navigation and access control, built on a hybrid architecture that combines web technologies with native platform capabilities. Unlike traditional approaches that compromise on either performance or cross-platform compatibility, this implementation leverages Tauri 2 to achieve native performance while maintaining a single codebase across iOS, Android, macOS, Windows, and Linux platforms.

**Language:** TypeScript (Frontend), Rust (Backend)
**Framework:** Vue 3 (UI), Tauri 2 (Native Bridge)
**Runtime:** Tokio (Async Rust), V8 (JavaScript)

## Architectural Philosophy

The mobile application solves a fundamental challenge in indoor navigation: providing real-time positioning and route guidance in environments where GPS signals are unreliable or absent. The architecture reflects three core design principles:

1. **Offline-First Operation**: Indoor environments often have spotty network connectivity. The app must function seamlessly regardless of network availability, which necessitates a local-first data architecture with intelligent synchronization.

2. **Resource Efficiency**: Mobile devices have constrained battery and processing power. The design prioritizes efficient BLE scanning, minimal network requests, and optimized pathfinding computations.

3. **Security by Design**: Access control features require cryptographic operations that must be both secure and performant. The solution uses hardware-backed key storage and biometric authentication where available.

## Technical Architecture

The application employs a clean separation between the presentation layer (Vue 3 frontend) and the business logic layer (Rust backend). This separation isn't merely organizational—it reflects a fundamental architectural decision about where computation should occur.

### Frontend Layer (Vue 3 + TypeScript)

The frontend manages user interaction, visual rendering, and reactive state management. Built with Vue 3's Composition API, it provides a declarative approach to UI construction while maintaining fine-grained reactivity. The choice of Vue over React or other frameworks was deliberate: Vue's template syntax and reactivity system provide better developer ergonomics for complex state-dependent UIs like navigation interfaces.

**State Management Strategy:**

Pinia serves as the central state store, but unlike traditional single-store architectures, the application uses domain-specific stores:
- Session store: User authentication state, current location, active navigation
- Entity store: Cached building data, floor plans, area geometries
- Beacon store: Known beacon locations, RSSI measurements, positioning data

This modular approach allows granular reactivity—updating beacon RSSI values doesn't trigger re-renders of unrelated UI components.

**Map Rendering Architecture:**

Indoor map visualization combines two rendering technologies:
1. **MapLibre GL** provides the base layer—vector tile rendering of building outlines, outdoor context, and floor plan backgrounds
2. **Konva Canvas** overlays interactive elements—area polygons, navigation routes, user position markers

This dual-layer approach emerged from practical constraints. MapLibre excels at rendering geospatial data but struggles with frequently updating elements like user position or dynamic route overlays. Konva, being a canvas-based rendering library, efficiently handles these dynamic elements without triggering expensive map re-renders.

The synchronization between these layers requires careful coordinate system management. MapLibre operates in Web Mercator projection, while indoor coordinates use local Cartesian systems. The transformation layer handles this impedance mismatch transparently.

### Backend Layer (Rust + Tauri)

The Rust backend handles computationally intensive operations and platform-specific functionality. Tauri's command system provides type-safe communication between frontend and backend through a serialization boundary.

**Tauri Command Architecture:**

Commands follow a consistent pattern:
1. Frontend invokes command via Tauri API
2. Request serialization (JSON over IPC)
3. Rust handler executes business logic
4. Response serialization back to frontend

This might seem inefficient compared to native mobile development, but the IPC overhead is negligible (typically <1ms) for most operations. The real bottlenecks are BLE scanning and network requests, which would exist in any architecture.

**Plugin System:**

Tauri's plugin architecture provides access to native platform features:
- **tauri-plugin-blec**: BLE communication layer
- **tauri-plugin-sql**: SQLite database for offline storage
- **tauri-plugin-stronghold**: Encrypted key storage using platform keychains
- **tauri-plugin-biometric**: Touch ID, Face ID, fingerprint authentication
- **tauri-plugin-nfc**: Near-field communication (future feature)

Each plugin wraps platform-specific APIs in a unified Rust interface. For example, biometric authentication works differently on iOS (LAContext), Android (BiometricPrompt), and Windows (Windows Hello), but the application code only interacts with a single `authenticate()` function.

## Localization System

Indoor positioning is fundamentally different from GPS-based navigation. GPS uses trilateration from satellites; indoor positioning uses RSSI (Received Signal Strength Indicator) trilateration from BLE beacons. The challenges are substantial: radio signals reflect off walls, human bodies attenuate signals, and RSSI values fluctuate wildly even when the device is stationary.

### Beacon Discovery and Identification

The localization process begins with BLE scanning. The application scans for advertising packets from nearby beacons, extracting MAC addresses and RSSI values. However, MAC addresses alone aren't useful—the system needs to map them to physical locations.

**Device Identification Protocol:**

When a beacon is discovered for the first time:
1. Mobile connects to the beacon via GATT (Generic Attribute Profile)
2. Subscribes to the characteristic at UUID `134b1d88-cd91-8134-3e94-5c4052743845`
3. Sends a `DeviceRequest` message
4. Beacon responds with a `DeviceResponse` containing its database ID, device type, and capabilities
5. Mobile queries the server: `GET /api/entities/{entity}/beacons/{id}` to fetch beacon metadata
6. Beacon information is cached in local SQLite database

This handshake occurs only once per beacon. Subsequent scans use the cached MAC-to-ID mapping, avoiding connection overhead.

**Database Caching Strategy:**

The SQLite schema maintains several tables:
- `beacons`: MAC address, object ID, location coordinates, area association
- `areas`: Polygon geometry (WKT format), floor identifier, accessibility metadata
- `merchants`: Points of interest for navigation destinations

This local cache enables offline operation. When network connectivity is available, the app performs background synchronization to update any changed data.

### RSSI-Based Trilateration

Once beacons are identified, the system uses RSSI measurements to estimate position. The relationship between RSSI and distance follows a logarithmic path loss model:

```
RSSI = -10n * log10(d) + A
```

Where:
- `n` is the path loss exponent (typically 2-4 for indoor environments)
- `d` is distance in meters
- `A` is the measured RSSI at 1 meter reference distance

However, this model is highly unreliable in practice. Signal multipath, absorption by materials, and interference create RSSI variations of ±10 dBm or more. The implementation addresses this through:

1. **Kalman Filtering**: RSSI measurements are filtered to smooth out high-frequency noise
2. **Weighted Least Squares**: Position estimation uses weighted trilateration, giving more weight to stronger (closer) beacons
3. **Area Context**: The system constrains position estimates to the detected area's polygon geometry

**Area Detection:**

Before trilateration can occur, the system must determine which area the user is in. This uses a majority voting algorithm:
1. For each detected beacon, lookup its associated area
2. Count area occurrences across all detected beacons
3. The area with the most beacons is considered the current area

This approach handles edge cases where a user might detect beacons from multiple adjacent areas.

### Position Update Frequency

The localization handler runs on-demand rather than continuously. Continuous BLE scanning would drain battery rapidly. Instead, the UI triggers localization when:
- User opens the navigation screen
- User requests current position
- Navigation is active (periodic updates every 5 seconds)

This event-driven approach balances responsiveness with power efficiency.

## Navigation and Routing

The navigation system provides turn-by-turn guidance through complex multi-floor buildings. Unlike outdoor navigation which operates on road networks, indoor navigation must handle arbitrary polygon geometries and vertical connections.

### Route Request Flow

When a user requests navigation from their current position to a destination:

1. **Origin and Destination Resolution:**
   - Current position: `(x, y, area_id)` from localization system
   - Destination: Either coordinates `(x, y, area_id)` or a merchant ID
   - If merchant ID provided, resolve to coordinates via local database

2. **Server Route Query:**
   ```
   GET /api/entities/{entity_id}/route?from={x},{y},{area_id}&to={x},{y},{area_id}&disallow={constraints}
   ```

3. **Instruction Parsing:**
   The server returns a sequence of navigation instructions. The mobile app parses these into actionable steps with human-readable descriptions.

4. **Offline Fallback:**
   If the server is unreachable, the app attempts local pathfinding using cached polygon geometries. This is limited to same-floor navigation but provides degraded functionality during network outages.

### Instruction Interpretation

Server instructions arrive as structured data describing abstract navigation steps. The mobile frontend transforms these into user-facing guidance:

**Walk Instructions:**
These describe movement within an area. The instruction includes distance and direction, but the mobile app enhances this with visual route overlay on the map canvas.

**Transport Instructions:**
These indicate transitions between areas via elevators, stairs, or escalators. The UI presents these prominently since they represent decision points where users might become confused.

**Area Transition Management:**
When navigation crosses area boundaries, the app must:
1. Update the displayed floor plan
2. Adjust the map viewport to the new area
3. Re-run localization in the new context

This coordination between navigation state and UI state requires careful reactivity management to avoid race conditions.

### Visual Route Rendering

The route overlay uses Konva's vector graphics capabilities to draw the path on the map canvas. The rendering algorithm:
1. Transforms route waypoints from local coordinates to screen coordinates
2. Draws a polyline connecting waypoints
3. Adds directional arrows at intervals
4. Highlights the next upcoming instruction

The rendering updates in real-time as the user moves, with the path remaining visually stable even as the map pans and zooms.

## Access Control System

The mobile app serves as a digital key for unlocking doors, gates, and turnstiles equipped with Navign beacons. This system implements a challenge-response protocol that prevents replay attacks and ensures only authorized users can access controlled areas.

### Cryptographic Foundation

Each mobile device generates a P-256 ECDSA key pair on first launch. The private key is stored in platform-specific secure storage:
- **iOS**: Keychain with `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` attribute
- **Android**: Android Keystore with biometric protection
- **Desktop**: Tauri Stronghold encrypted vault

The public key is registered with the server and associated with the user's account. Beacons maintain a list of authorized public keys (or delegate authorization checks to the server).

### Unlock Protocol

The complete unlock sequence involves multiple round-trips between mobile, beacon, and server:

**Phase 1: Nonce Challenge**
1. User initiates unlock request in the mobile UI
2. Mobile connects to beacon via BLE GATT
3. Mobile sends `NonceRequest` to beacon
4. Beacon generates a 32-byte random nonce
5. Beacon signs the nonce with its private key (proof of beacon authenticity)
6. Beacon sends `NonceResponse(nonce, signature_identifier)`

The signature identifier is the last 8 bytes of the beacon's signature. This allows the mobile to verify it's communicating with a genuine Navign beacon without performing full signature verification (which would require knowing the beacon's public key a priori).

**Phase 2: Server Verification**
7. Mobile requests unlock instance from server:
   ```
   POST /api/entities/{eid}/beacons/{bid}/unlocker
   ```
8. Server generates a TOTP code for this specific unlock attempt
9. Server returns the TOTP and updates its database with unlock instance metadata

**Phase 3: Proof Generation**
10. Mobile constructs proof payload: `nonce || device_id || totp`
11. Mobile signs the payload with its private key (ECDSA signature)
12. This signature proves the mobile possesses the private key associated with the authorized user

**Phase 4: Beacon Verification**
13. Mobile sends `UnlockRequest(proof)` to beacon
14. Beacon extracts the signature and payload from the proof
15. Beacon verifies the ECDSA signature using the mobile's registered public key
16. Beacon checks the nonce hasn't expired (5-second TTL)
17. Beacon checks the nonce hasn't been used before (replay attack prevention)
18. Beacon enforces rate limiting (max 5 attempts per 5 minutes)

**Phase 5: Physical Unlock**
19. If all checks pass, beacon activates the relay (or servo, or IR transmitter)
20. Physical lock mechanism opens
21. Beacon sends `UnlockResponse(success=true)`

**Phase 6: Audit Trail**
22. Mobile reports outcome to server:
    ```
    PUT /api/entities/{eid}/beacons/{bid}/unlocker/{instance}/outcome
    ```
23. Server logs the access event for audit purposes

### Security Considerations

This multi-phase protocol addresses several attack vectors:

**Replay Attacks:**
Nonces are single-use and expire after 5 seconds. Even if an attacker captures a valid unlock proof, they cannot reuse it.

**Relay Attacks:**
The tight time constraint (5-second nonce expiration) makes relay attacks impractical in most scenarios. An attacker would need to capture the nonce, relay it to the legitimate user, obtain their signature, and relay it back—all within 5 seconds.

**Beacon Impersonation:**
The beacon signs its nonce, proving it possesses the private key associated with a registered beacon. This prevents rogue devices from posing as legitimate access points.

**User Impersonation:**
The mobile's proof signature requires possession of the private key, which is hardware-protected and biometrically gated. An attacker would need to extract the key from secure storage or bypass biometric authentication.

### Biometric Integration

On mobile platforms, the unlock process requires biometric authentication before accessing the private key. The flow:

1. User taps "Unlock" in the UI
2. System prompts for Face ID / Touch ID / Fingerprint
3. On successful authentication, Tauri retrieves the private key from Stronghold
4. Key is used to sign the proof, then immediately cleared from memory

This ensures that even if the device is unlocked, an unauthorized person cannot trigger access control without the owner's biometric authentication.

### Offline Unlock Capability

The current implementation requires server connectivity for TOTP generation. A future enhancement could pre-fetch time-based unlock tokens during online periods, enabling offline unlock with the caveat that the mobile device's clock must be synchronized.

## Data Synchronization

The mobile app maintains a local SQLite database that mirrors a subset of the server's data. This enables offline operation but introduces synchronization challenges.

### Sync Strategy

The application uses a **last-write-wins** strategy with server authority. The sync process:

1. **On Login:**
   - Clear local database (or migrate schema if version changed)
   - Fetch entity data for user's current location
   - Populate areas, beacons, merchants for that entity

2. **Background Sync:**
   - When app returns to foreground
   - Periodically (every 15 minutes if network available)
   - On-demand when user pulls to refresh

3. **Partial Sync:**
   - Only sync changed data using timestamp-based queries
   - Server provides pagination for large result sets

### Schema Migrations

SQLite schema evolution uses Tauri's migration system. Migrations are embedded in the binary and run automatically on app startup:

```rust
Migration {
    version: 2,
    description: "comprehensive schema with WKT support",
    sql: include_str!("navign_v2.sql"),
    kind: MigrationKind::Up,
}
```

This ensures that users upgrading from older app versions have their local database schema updated transparently.

### Cache Invalidation

The challenge with caching is knowing when cached data becomes stale. The app handles this through:

1. **TTL-based Expiration**: Cached entities expire after 24 hours
2. **Server-Driven Invalidation**: Server includes `Last-Modified` headers; client compares with cache timestamps
3. **User-Initiated Refresh**: Pull-to-refresh forces re-fetch from server

## Platform-Specific Implementations

While Tauri provides cross-platform abstractions, certain features require platform-specific code paths.

### BLE Implementation Differences

**iOS:**
Core Bluetooth framework requires explicit peripheral connection and characteristic discovery. The BLE plugin abstracts this, but iOS enforces stricter background scanning limitations. The app can only scan for 10 seconds at a time when backgrounded.

**Android:**
Android's Bluetooth stack has evolved significantly across versions. The plugin handles API level detection and uses the appropriate APIs (legacy Bluetooth vs. BLE APIs).

**Desktop:**
Desktop BLE support varies by platform. macOS has native BLE through IOBluetooth, Windows uses Windows.Devices.Bluetooth, and Linux requires BlueZ. The plugin wraps these platform APIs.

### File System Access

Mobile platforms sandbox file system access. Tauri's path resolver provides platform-appropriate directories:
- **iOS**: Application Support directory (iCloud synced if configured)
- **Android**: Internal storage (app-private directory)
- **Desktop**: System-specific app data directories

The SQLite database location is resolved at runtime to ensure it's stored in the correct platform-specific location.

### Biometric Authentication

Biometric implementations vary significantly:

**iOS**: Local Authentication framework with LAContext
**Android**: BiometricPrompt API (API 28+) with fallback to FingerprintManager
**Desktop**: Platform-specific (Windows Hello, Touch ID on macOS)

The plugin provides a unified interface that handles these platform differences transparently.

## Performance Optimizations

Several optimizations ensure the app remains responsive even on lower-end devices:

### Lazy Loading

Map data and merchant information are loaded on-demand rather than upfront. When a user opens an area, only that area's geometry and merchants are fetched.

### Render Throttling

Position updates trigger map re-renders, but these are throttled to 10 FPS maximum. The human eye can't perceive faster updates, and throttling reduces GPU usage significantly.

### BLE Scan Batching

Rather than processing each BLE advertisement packet individually, the scan handler batches packets over 500ms windows. This reduces context switching and improves RSSI filtering accuracy.

### Reactive Dependency Optimization

Vue's reactivity system tracks dependencies automatically, but unnecessary dependencies can cause cascade re-renders. The codebase uses `shallowRef` and `shallowReactive` for large data structures where deep reactivity isn't needed.

## Related Documentation

- [Mobile Localization Pipeline](/pipelines/localization)
- [Mobile Unlock Pipeline](/pipelines/unlock)
- [BLE Protocol Specification](/components/beacon#ble-protocol)
- [Tauri Backend Commands](/components/mobile/tauri-commands)
