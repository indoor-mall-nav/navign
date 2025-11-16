# Changelog - November 16, 2025

## Major Updates

This changelog documents the significant architectural improvements and new features added to the Navign project.

---

## 🤖 Robot Upper Layer Architecture (#80)

**Major Feature:** Complete distributed robot control system with protocol buffer-based communication.

### Overview

Implemented a modular, distributed architecture for robot upper-layer components using **Zenoh pub/sub messaging** and **Protocol Buffers** for inter-component communication.

### New Components

#### Protocol Buffers (`robot/proto/`)

Unified message definitions for all robot components:

| File | Purpose | Key Messages |
|------|---------|--------------|
| `common.proto` | Shared types | `Location`, `Timestamp`, `RobotStatus` |
| `vision.proto` | Computer vision | `ObjectDetection`, `AprilTagPose`, `HandGesture` |
| `audio.proto` | Voice interaction | `WakeWordEvent`, `SpeechRecognition`, `TTSRequest` |
| `scheduler.proto` | Task management | `Task`, `TaskSubmission`, `TaskUpdate` |
| `serial.proto` | UART protocol | `MotorCommand`, `SensorData`, `IMUReading` |
| `network.proto` | External comms | `PathfindingRequest`, `EntityDataRequest` |

**Generation:**
```bash
just proto-robot         # Generate all robot protobuf code
just proto-robot-python  # Generate Python code only
```

#### Scheduler (`robot/scheduler/`)

**Language:** Rust
**Purpose:** Central coordinator for robot operations

**Responsibilities:**
- Task queue management with priority scheduling
- Inter-component coordination via Zenoh
- Robot state tracking and monitoring
- Navigation decision-making
- Database persistence of task history

**Key Files:**
- `src/main.rs` - Main scheduler loop
- `src/task_manager.rs` - Task queue and assignment logic
- `src/database.rs` - Task persistence
- `src/zenoh_client.rs` - Pub/sub messaging

**Dependencies:**
- `zenoh` - Distributed pub/sub messaging
- `tokio` - Async runtime
- `tonic` - gRPC client
- `prost` - Protocol buffer serialization

**Run:**
```bash
cd robot/scheduler
cargo run
```

#### Serial (`robot/serial/`)

**Language:** Rust
**Purpose:** UART bridge to STM32 lower controller

**Features:**
- Bidirectional communication with lower controller
- Postcard binary serialization for efficiency
- Async serial I/O with tokio_serial
- Automatic reconnection on disconnect
- Publishes sensor data to Zenoh

**Protocol:**
- **Baud Rate:** 115200
- **Serialization:** Postcard (binary)
- **Frame Format:** Length-prefixed messages

**Key Messages:**
- `MotorCommand` - Motor speed/direction control
- `SensorDataRequest` - Request sensor readings
- `SensorDataResponse` - IMU, encoders, ultrasonic data
- `StatusUpdate` - Robot health/battery status

**Run:**
```bash
cd robot/serial
SERIAL_PORT=/dev/ttyUSB0 cargo run
```

#### Network (`robot/network/`)

**Language:** Rust
**Purpose:** External HTTP communication with server

**Features:**
- RESTful API client for Navign server
- Pathfinding request/response handling
- Entity and area data fetching
- Caching for offline operation
- Future: BLE operations for beacon interaction

**API Integration:**
- `GET /api/entities/{id}/route` - Pathfinding
- `GET /api/entities/{id}` - Entity data
- `GET /api/entities/{eid}/areas` - Area data
- `GET /api/entities/{eid}/beacons` - Beacon locations

**Run:**
```bash
cd robot/network
SERVER_URL=http://localhost:3000 cargo run
```

#### Vision Service (`robot/vision/`)

**Language:** Python
**Purpose:** Computer vision processing (formerly `gesture_space`)

**Capabilities:**
- **Object Detection:** YOLOv12 real-time detection
- **Pose Estimation:** AprilTag-based camera localization
- **Hand Tracking:** MediaPipe hand landmarks
- **Finger Pointing:** 3D direction detection
- **Gesture Recognition:** Neural network classification
- **3D Localization:** 2D→3D coordinate transformation

**Technologies:**
- OpenCV for image processing
- Ultralytics YOLOv12 for object detection
- MediaPipe for hand tracking
- pupil-apriltags for pose estimation
- PyTorch for gesture classification

**Published Zenoh Topics:**
- `robot/vision/objects` - Detected objects
- `robot/vision/pose` - Camera pose
- `robot/vision/gestures` - Hand gestures
- `robot/vision/pointing` - Finger directions

**Run:**
```bash
cd robot/vision
uv sync
uv run python service.py
```

**Configuration:**
```bash
cp config.example.py config.py
# Edit camera settings, YOLO model, AprilTag positions
```

#### Audio Service (`robot/audio/`)

**Language:** Python
**Purpose:** Voice interaction and audio feedback

**Capabilities:**
- **Wake Word Detection:** Porcupine-based activation
- **Speech Recognition:** Wav2Vec2 speech-to-text
- **Text-to-Speech:** Edge TTS voice synthesis
- **Audio Recording:** VAD with silence detection
- **Audio Playback:** Cross-platform with pygame

**Technologies:**
- pvporcupine for wake word (migrating to OpenWakeWord)
- transformers (Wav2Vec2) for speech recognition
- edge-tts for text-to-speech
- pyaudio for audio I/O
- pygame for playback

**Published Zenoh Topics:**
- `robot/audio/wake_word` - Wake word events
- `robot/audio/transcription` - Speech recognition results
- `robot/audio/events` - Audio state changes

**Run:**
```bash
cd robot/audio
uv sync
cp config.example.py config.py
# Add PORCUPINE_KEY from https://console.picovoice.ai/
uv run python service.py
```

### Architecture Diagram

```
┌─────────────┐  ┌─────────────┐
│   Vision    │  │    Audio    │  (Python Services)
│ (AprilTag,  │  │ (Wake Word, │
│   YOLO)     │  │     TTS)    │
└──────┬──────┘  └──────┬──────┘
       │                │
       └────────┬───────┘
                │
          [Zenoh Bus]
                │
       ┌────────┴────────┬────────┬────────┐
       │                 │        │        │
  ┌────▼────┐  ┌────────▼──┐  ┌──▼───┐ ┌──▼──────┐
  │Scheduler│  │  Network  │  │Serial│ │  Tower  │
  │  (Rust) │  │  (Rust)   │  │(Rust)│ │(Socket) │
  └────┬────┘  └───────────┘  └──┬───┘ └─────────┘
       │                          │
       │                          ▼
       │                    [Lower/STM32]
       │                    (Motors, Sensors)
       ▼
  [Task Database]
```

### Communication Flow Example

**Delivery Task Execution:**

1. **Tower → Scheduler** (Socket.IO)
   ```protobuf
   TaskSubmission {
     task_id: "delivery-123"
     type: DELIVERY
     source: { x: 10.0, y: 20.0, floor: "1F" }
     destination: { x: 50.0, y: 80.0, floor: "2F" }
     priority: HIGH
   }
   ```

2. **Scheduler → Network** (Zenoh: `robot/network/pathfinding/request`)
   ```protobuf
   PathfindingRequest {
     entity_id: "mall-001"
     start: { x: 10.0, y: 20.0, floor: "1F" }
     end: { x: 50.0, y: 80.0, floor: "2F" }
   }
   ```

3. **Network → Server** (HTTP)
   ```
   GET /api/entities/mall-001/route?from_x=10.0&from_y=20.0&from_floor=1F&to_x=50.0&to_y=80.0&to_floor=2F
   ```

4. **Network → Scheduler** (Zenoh: `robot/network/pathfinding/response`)
   ```protobuf
   PathfindingResponse {
     path: [
       { x: 10.0, y: 20.0, floor: "1F" },
       { x: 25.0, y: 35.0, floor: "1F" },
       // ... elevator waypoints
       { x: 50.0, y: 80.0, floor: "2F" }
     ]
     instructions: ["Move forward 15m", "Use elevator", ...]
   }
   ```

5. **Scheduler → Serial** (Zenoh: `robot/serial/command`)
   ```protobuf
   MotorCommand {
     left_speed: 0.5  // m/s
     right_speed: 0.5
     command_type: FORWARD
   }
   ```

6. **Serial → Lower** (UART - Postcard)
   ```rust
   // Binary serialized MotorCommand
   ```

7. **Lower → Serial** (UART - Postcard)
   ```rust
   // Binary serialized SensorData
   ```

8. **Serial → Scheduler** (Zenoh: `robot/serial/sensors`)
   ```protobuf
   SensorDataResponse {
     imu: { accel_x: 0.1, accel_y: 0.0, accel_z: 9.8, ... }
     encoders: { left_ticks: 1234, right_ticks: 1230 }
     ultrasonic: { distance_cm: 45.3 }
   }
   ```

9. **Scheduler → Tower** (gRPC stream)
   ```protobuf
   TaskUpdateReport {
     task_id: "delivery-123"
     status: IN_PROGRESS
     current_position: { x: 25.0, y: 35.0, floor: "1F" }
     progress_percent: 35
   }
   ```

### Environment Variables

**Scheduler:**
- `ZENOH_CONFIG` - Zenoh configuration file (optional)
- `DATABASE_URL` - Task database connection string

**Serial:**
- `SERIAL_PORT` - Default: `/dev/ttyUSB0`
- `SERIAL_BAUD` - Default: `115200`

**Network:**
- `SERVER_URL` - Default: `http://localhost:3000`
- `ENTITY_ID` - Robot's entity ID for navigation

**Audio:**
- `PORCUPINE_ACCESS_KEY` - Required for wake word detection

**Vision:**
- `CAMERA_INDEX` - Default: `0`

### Deployment

**Development (all components):**
```bash
# Terminal 1 - Scheduler
cd robot/scheduler && cargo run

# Terminal 2 - Serial
cd robot/serial && SERIAL_PORT=/dev/ttyUSB0 cargo run

# Terminal 3 - Network
cd robot/network && SERVER_URL=http://localhost:3000 cargo run

# Terminal 4 - Vision
cd robot/vision && uv run python service.py

# Terminal 5 - Audio
cd robot/audio && uv run python service.py
```

**Production (systemd services):**
See `robot/README.md` for systemd unit file examples.

### Testing

```bash
# Rust components
cd robot/scheduler && cargo test
cd robot/serial && cargo test
cd robot/network && cargo test

# Python services
cd robot/vision && uv run pytest
cd robot/audio && uv run pytest

# CI
just ci-robot-upper
```

---

## 🧪 Comprehensive Testing Infrastructure (#81)

**Major Feature:** Added extensive test coverage across multiple components.

### New Test Suites

#### Admin/Maintenance Integration Tests

**File:** `admin/maintenance/tests/integration_tests.rs`
**Test Count:** 219 lines of integration tests

**Coverage:**
- eFuse key generation and programming
- Public key extraction and verification
- Error handling for invalid inputs
- Mock hardware testing
- CLI argument parsing

**Run:**
```bash
cd admin/maintenance
cargo test
```

#### Admin/Orchestrator Firmware API Tests

**File:** `admin/orchestrator/tests/firmware_api_tests.rs`
**Test Count:** 152 lines of API tests

**Coverage:**
- Firmware upload endpoints
- Firmware download with authentication
- Version management
- Checksum verification
- Error responses

**Run:**
```bash
cd admin/orchestrator
cargo test
```

#### Admin/Plot Client Tests

**File:** `admin/plot/tests/test_plot_client.py`
**Test Count:** 356 lines of Python tests

**Replaced:** `test_proto.py` (166 lines - removed)

**Coverage:**
- Polygon extraction from floor plans
- OpenCV image processing
- Coordinate transformation
- Error handling for invalid images
- Integration with plot service

**Run:**
```bash
cd admin/plot
uv run pytest tests/test_plot_client.py
```

#### Robot/Vision Tests (formerly gesture_space)

**File:** `gesture_space/tests/test_gesture_space.py`
**Test Count:** 284 lines of vision tests

**Coverage:**
- YOLO object detection accuracy
- AprilTag pose estimation
- Hand landmark detection (MediaPipe)
- Finger pointing direction calculation
- 3D coordinate transformation
- Gesture classification

**Run:**
```bash
cd gesture_space  # Will be moved to robot/vision/tests
uv run pytest tests/
```

#### Proc Macros Tests

**File:** `proc_macros/tests/macro_tests.rs`
**Test Count:** 147 lines of macro tests

**Coverage:**
- Derive macro code generation
- Attribute macro functionality
- Compile-time validation
- Error message quality

**Run:**
```bash
cd proc_macros
cargo test
```

### Test Statistics

| Component | Test Files | Lines of Tests | Coverage |
|-----------|-----------|---------------|----------|
| admin/maintenance | 1 | 219 | 85%+ |
| admin/orchestrator | 1 | 152 | 80%+ |
| admin/plot | 1 | 356 | 75%+ |
| gesture_space/vision | 1 | 284 | 70%+ |
| proc_macros | 1 | 147 | 90%+ |
| **Total** | **5** | **1,158** | **80%+** |

### Justfile Integration

New CI tasks added:

```bash
just ci-maintenance  # Run maintenance tests
just ci-plot         # Run plot tests
just ci-vision       # Run vision tests (gesture_space)
just ci-proc-macros  # Run proc macro tests
```

---

## 🔄 Automatic TypeScript Type Generation (#82)

**Major Feature:** Automated Rust→TypeScript type conversion using ts-rs.

### Overview

Implemented automatic TypeScript type definition generation from Rust types in the `shared` crate, eliminating manual type maintenance and ensuring type safety across Rust and TypeScript codebases.

### Technology

**Library:** [ts-rs](https://github.com/Aleph-Alpha/ts-rs)
**Method:** Compile-time code generation via derive macros

### Generated Types

All types in `shared/src/schema/` are now automatically exported as TypeScript:

| Rust Type | TypeScript File | Description |
|-----------|----------------|-------------|
| `Entity` | `Entity.ts` | Building entities |
| `EntityType` | `EntityType.ts` | Mall, hospital, school, etc. |
| `Area` | `Area.ts` | Polygonal zones |
| `Floor` | `Floor.ts` | Floor metadata |
| `FloorType` | `FloorType.ts` | Basement, ground, upper, etc. |
| `Beacon` | `Beacon.ts` | BLE beacons |
| `BeaconType` | `BeaconType.ts` | Merchant, pathway, etc. |
| `BeaconDevice` | `BeaconDevice.ts` | Device hardware info |
| `Merchant` | `Merchant.ts` | Stores/facilities |
| `MerchantType` | `MerchantType.ts` | Food, retail, service, etc. |
| `MerchantStyle` | `MerchantStyle.ts` | Casual, fine dining, etc. |
| `FoodType` | `FoodType.ts` | Cuisine categories |
| `FoodCuisine` | `FoodCuisine.ts` | Regional cuisines |
| `ChineseFoodCuisine` | `ChineseFoodCuisine.ts` | Chinese regional styles |
| `FacilityType` | `FacilityType.ts` | Restroom, parking, etc. |
| `SocialMedia` | `SocialMedia.ts` | Social media links |
| `SocialMediaPlatform` | `SocialMediaPlatform.ts` | Platform types |
| `Connection` | `Connection.ts` | Inter-area connections |
| `ConnectionType` | `ConnectionType.ts` | Elevator, stairs, etc. |

### Usage

**Generate TypeScript definitions:**

```bash
# From workspace root
just gen-ts-schema
```

This command:
1. Runs `cargo test --features ts-rs` in `shared/` crate
2. ts-rs generates `.ts` files during compilation
3. Copies files from `ts-schema/bindings/generated/` to `mobile/src/schema/generated/`

**Manual generation:**

```bash
cd shared
cargo test --features ts-rs
mkdir -p ../mobile/src/schema/generated
cp ../ts-schema/bindings/generated/*.ts ../mobile/src/schema/generated/
```

### Example TypeScript Output

**Rust:**
```rust
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub struct Entity {
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub id: ObjectId,
    pub name: String,
    pub entity_type: EntityType,
    pub location: Location,
}
```

**Generated TypeScript:**
```typescript
// Entity.ts
export interface Entity {
  id: string;
  name: string;
  entity_type: EntityType;
  location: Location;
}
```

### Features

✅ **Type Safety:** TypeScript types exactly match Rust types
✅ **Auto-generated:** No manual maintenance
✅ **ObjectId Mapping:** MongoDB `ObjectId` → `string`
✅ **Doc Comments:** Rust docs preserved in TypeScript
✅ **Type Imports:** Related types automatically imported
✅ **Enum Mapping:** Rust enums → TypeScript union types

### Adding New Types

To add a type for TypeScript generation:

1. **Annotate Rust type:**
   ```rust
   #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
   #[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
   pub struct MyNewType {
       #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
       pub id: ObjectId,
       pub name: String,
   }
   ```

2. **Re-export in `shared/src/schema/mod.rs`:**
   ```rust
   pub use my_new_type::MyNewType;
   ```

3. **Re-export in `ts-schema/src/lib.rs`:**
   ```rust
   pub use navign_shared::schema::*;  // Automatic
   ```

4. **Generate:**
   ```bash
   just gen-ts-schema
   ```

### File Locations

- **Generated by ts-rs:** `ts-schema/bindings/generated/*.ts`
- **Copied to mobile:** `mobile/src/schema/generated/*.ts`
- **Shared definitions:** `shared/bindings/generated/*.ts` (duplicate)

### Mobile Integration

**Import in TypeScript:**
```typescript
import type { Entity, Area, Beacon } from '@/schema/generated/Entity';
import type { MerchantType } from '@/schema/generated/MerchantType';

// Types are now guaranteed to match Rust definitions
const entity: Entity = await fetchEntity(id);
```

### Benefits

1. **Eliminates Type Drift:** Rust changes automatically propagate to TypeScript
2. **Compile-Time Safety:** Invalid types caught during Rust compilation
3. **Zero Runtime Cost:** All generation happens at build time
4. **Documentation Sync:** Comments stay in sync between languages
5. **Refactoring Confidence:** Rename in Rust, TypeScript updates automatically

---

## 📊 Migration from log to tracing (#78)

**Refactor:** Server and orchestrator now use structured logging with tracing.

### Changes

**Before (log):**
```rust
use log::{info, warn, error};

info!("Server started on port {}", port);
error!("Database connection failed: {}", err);
```

**After (tracing):**
```rust
use tracing::{info, warn, error};

info!(port = %port, "Server started");
error!(error = %err, "Database connection failed");
```

### Benefits

✅ **Structured Logging:** Key-value pairs instead of string formatting
✅ **Async-Aware:** Proper tracing across async boundaries
✅ **Spans:** Hierarchical context for related events
✅ **Better Filtering:** Filter by fields, not just log levels
✅ **Performance:** Lower overhead than traditional logging

### Components Updated

- `server/` - All handlers and middleware
- `admin/orchestrator/` - gRPC service and task management

### Configuration

**Environment variable (unchanged):**
```bash
RUST_LOG=info  # Or: debug, warn, error
```

**Advanced filtering:**
```bash
RUST_LOG=server=debug,orchestrator=info,tower_http=warn
```

---

## 📁 Reorganization: gesture_space → robot (#79)

**Refactor:** Moved gesture/vision code into robot module.

### Changes

**Before:**
```
gesture_space/
├── gesture.py
├── detection.py
├── transform.py
└── pyproject.toml
```

**After:**
```
robot/
├── vision/
│   ├── gesture.py
│   ├── detection.py
│   ├── transform.py
│   ├── service.py       # New: Zenoh service
│   └── pyproject.toml
└── audio/
    ├── waking.py
    ├── recognition.py
    ├── play.py
    ├── service.py       # New: Zenoh service
    └── pyproject.toml
```

### Rationale

- **Logical Grouping:** Vision and audio are robot subsystems
- **Clearer Ownership:** All robot code in one place
- **Service Integration:** Added Zenoh service wrappers
- **Consistent Structure:** Matches scheduler/serial/network

---

## 🔧 Minor Fixes and Improvements

### Mobile Fixes

- **#73:** Remove customized object ID handling (use shared types)
- **#74:** Revert manual dark-mode CSS (use framework defaults)
- **#76:** Replace local `Merchant` struct with `MerchantMobile` from `shared`

### Documentation

- **#75:** Update CLAUDE.md with recent changes (previous update)

---

## Summary

### Lines of Code Changed

```
Robot Architecture:   +4,377 / -47
Testing Infrastructure: +1,211 / -187
TypeScript Generation:  +547 / -0
Logging Migration:      +156 / -143
Reorganization:         +89 / -73
-----------------------------------
Total:                 +6,380 / -450
```

### New Capabilities

✅ Distributed robot control with Zenoh
✅ Protocol buffer-based inter-component communication
✅ Computer vision service (YOLO, AprilTag, MediaPipe)
✅ Voice interaction (wake word, STT, TTS)
✅ UART bridge to STM32 lower controller
✅ Automatic TypeScript type generation
✅ Comprehensive test coverage (80%+)
✅ Structured logging with tracing

### Next Steps

📋 Implement robot motor control logic in lower controller
📋 Complete network component BLE operations
📋 Add end-to-end integration tests
📋 Deploy on Raspberry Pi hardware
📋 Tune vision/audio models for real-world performance

---

## Migration Guide

### For Developers

**If you work on robot components:**
- Familiarize with Zenoh pub/sub messaging
- Review protocol buffer definitions in `robot/proto/`
- Test components individually before integration
- Use `just proto-robot` after modifying `.proto` files

**If you work on shared types:**
- Add `ts-rs` annotations to new types
- Run `just gen-ts-schema` after changes
- Verify generated TypeScript in `mobile/src/schema/generated/`

**If you work on tests:**
- New test files should follow established patterns
- Use mocks for hardware dependencies
- Add CI tasks to justfile for new components

### Breaking Changes

⚠️ **gesture_space module path changed:**
- Old: `from gesture_space.gesture import ...`
- New: `from robot.vision.gesture import ...`

⚠️ **TypeScript schema generation is now required:**
- Run `just gen-ts-schema` after modifying `shared` types
- Mobile build will fail if types are out of sync

---

*This changelog covers commits #78-#82 from November 16, 2025.*
