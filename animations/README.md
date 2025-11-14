# Navign Animations

Manim animations visualizing the Navign indoor navigation system architecture and pipelines.

## Overview

These animations are created with [Manim Community](https://www.manim.community/) to explain the technical architecture, data flows, and algorithms used in the Navign system.

## Animation Scenes

### 1. Introduction (`intro.py`)

**Scene:** `NavignIntro`

Introduces the Navign system with a problem/solution framework:
- **Problem:** GPS failures indoors, complex multi-floor routing, insecure access control
- **Solution:** BLE beacon localization, Dijkstra multi-floor routing, ECDSA + biometric auth

**Duration:** ~4s

### 2. BLE Localization Pipeline (`localize.py`)

**Scene:** `LocalizationVisualization`

Visualizes the three-stage BLE localization process:
1. **BLE Scan:** Discover beacons with RSSI values
2. **Area Selection:** Group beacons by area and select current area
3. **Position Calculation:** Weighted centroid trilateration using RSSI→distance conversion

**Duration:** ~14s

**Key Concepts:** RSSI filtering, distance estimation, weighted centroid, area boundaries

### 3. Pathfinding Visualization (`path.py`)

**Scene:** `PathfindingVisualization`

Shows the two-tier pathfinding algorithm:
1. **Area-Level Routing:** Dijkstra's algorithm on connectivity graph (elevators, stairs, escalators)
2. **Inner-Area Routing:** Polygon quantification → bounded blocks → pathfinding

**Duration:** ~9s

**Key Concepts:** Graph construction, Dijkstra, polygon quantification, ray-casting, bounded blocks

### 4. Access Control - Beacon Logic (`beacon.py`)

**Scene:** `BeaconUnlockLogic`

Demonstrates beacon hardware unlock logic:
- Human sensor detection (PIR)
- Relay activation (HIGH signal)
- Door unlock (5-second hold timer)
- Automatic re-lock after timeout

**Duration:** ~4s

**Key Concepts:** Hardware triggers, relay control, timing logic

### 5. Full Unlock Pipeline (`unlock.py`)

**Scene:** `FullUnlockPipeline`

Complete end-to-end unlock protocol with BLE and HTTPS:
1. **Device Discovery:** BLE scan → DeviceRequest/Response (0x01/0x02)
2. **Nonce Challenge:** BLE NonceRequest/Response (0x03/0x04)
3. **Server Challenge:** HTTPS POST to create unlock instance
4. **Client Signature:** Biometric auth + ECDSA sign server challenge
5. **Server Proof:** Server signs beacon nonce + verifier
6. **Unlock Transmission:** BLE UnlockRequest (0x05) with full proof
7. **Outcome Reporting:** HTTPS PUT with success/failure

**Duration:** ~17s

**Key Concepts:** BLE protocol, ECDSA signatures, challenge-response, packet structures, REST API

### 6. OTA Update Pipeline (`ota_update.py`)

**Scene:** `OTAUpdatePipeline`

Shows the beacon firmware over-the-air update system:
1. **Flash Partition Layout:** Dual-bank system (OTA_0, OTA_1)
2. **Download & Write:** WiFi download in 4KB chunks, write to inactive partition
3. **Verification & Activation:** SHA-256 checksum, OTA Data partition update, reboot
4. **Rollback Safety:** Automatic rollback to previous firmware on boot failure

**Duration:** ~13s

**Key Concepts:** Dual-bank flash, checksum verification, partition management, rollback protection

### 7. Robot Task Assignment (`task_assignment.py`)

**Scene:** `RobotTaskAssignment`

Visualizes the distributed robot orchestration system:
1. **Architecture:** Mobile App → Orchestrator (Rust gRPC) → Tower (Go Socket.IO) → Robots
2. **Task Submission:** Delivery task with priority queuing
3. **Robot Selection:** Scoring algorithm (battery + proximity + capability)
4. **Task Execution:** Pathfinding, navigation, status updates

**Duration:** ~13s

**Key Concepts:** Microservices, gRPC streams, Socket.IO, priority queues, robot selection

### 8. Robot Architecture (`robot.py`)

**Scenes:**
- `RobotArchitecture`: Dual-purpose robot dog with upper layer (Raspberry Pi/Jetson) and lower layer (STM32)
- `DataFlowDetailed`: Guide scenario data flow ("Take me to Starbucks")
- `LocalizationHierarchy`: Multi-tier localization (BLE, AprilTags, IMU+Odometry) with EKF fusion
- `ControlLoop`: STM32 PID motor control and IMU sensor fusion

**Duration:** ~varies per scene

**Key Concepts:** ROS2, subsystems, serial communication, sensor fusion, PID control

### 9. Database Schema (`schema.py`)

**Scene:** `DatabaseSchema`

Visualizes the MongoDB database structure:
- Collections: Entity, Area, Connection, Merchant, Beacon
- Relationships: Entity → Areas/Beacons/Merchants, Area → Merchants/Beacons
- WKT format for polygon geometries

**Duration:** ~5s

**Key Concepts:** Document relationships, Well-Known Text (WKT), MongoDB schema

### 10. Outro (`outro.py`)

**Scene:** `NavignOutro`

Closing scene with:
- Three-tier architecture (Mobile, Server, Beacon)
- Communication protocols (HTTPS, BLE)
- Open source GitHub repository
- Future roadmap (Vision Pro, autonomous robots)

**Duration:** ~6s

## Rendering Animations

### Prerequisites

```bash
# Install Manim Community
pip install manim

# Or use uv (faster)
cd animations
uv sync
```

### Render a Single Scene

```bash
# Render at 1080p 60fps
manim -pqh intro.py NavignIntro

# Render at 720p (faster preview)
manim -pql intro.py NavignIntro

# Render at 4K
manim -pqk intro.py NavignIntro
```

### Render All Scenes

```bash
# Create a script to render all
for file in *.py; do
    if [ "$file" != "__init__.py" ]; then
        manim -pqh "$file"
    fi
done
```

### Output

Rendered videos are saved to `media/videos/<filename>/<quality>/`

## Animation Guidelines

### Style Conventions

- **Colors:**
  - BLUE: Mobile/client components
  - GREEN: Server/backend components
  - RED: Beacons/hardware
  - ORANGE: Robots
  - PURPLE: Admin/orchestration
  - YELLOW: Highlights/important info
  - GRAY: Secondary info

- **Fonts:**
  - Titles: 32-40pt
  - Sections: 20-24pt
  - Labels: 14-16pt
  - Details: 9-12pt

- **Timing:**
  - Intro/transitions: 0.3-0.5s
  - Main content: 0.5-0.8s
  - Important highlights: 0.6-1.0s
  - Pauses for reading: 0.3-0.5s

### Technical Accuracy

All animations are based on actual implementation in the codebase:
- Pathfinding uses Dijkstra (not A*)
- BLE protocol matches `shared/src/ble/message.rs`
- OTA matches `firmware/src/bin/ota.rs`
- Robot architecture matches `docs/docs/components/robot/`

When updating animations, verify against:
- `CLAUDE.md` - Comprehensive technical documentation
- `docs/docs/pipelines/` - Pipeline documentation
- Source code in respective component directories

## Contributing

When creating new animations:

1. **Choose a clear topic** - One pipeline or concept per scene
2. **Match documentation** - Verify against docs and source code
3. **Use consistent styling** - Follow color/font conventions
4. **Add to this README** - Document the new scene
5. **Test rendering** - Ensure no errors at all quality levels

## Dependencies

```toml
[project.dependencies]
manim = ">=0.19.0"
numpy = ">=2.3.3"
scipy = ">=1.16.2"
```

## License

MIT License - Same as the main Navign project.
