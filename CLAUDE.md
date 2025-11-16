# CLAUDE.md - AI Assistant Development Guide for Navign

This document provides comprehensive guidance for AI assistants working on the Navign indoor navigation system. It is based on actual code analysis and should be considered the authoritative source for development practices.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [Technology Stack](#technology-stack)
4. [Repository Structure](#repository-structure)
5. [Component Details](#component-details)
6. [Development Workflow](#development-workflow)
7. [Security Considerations](#security-considerations)
8. [Testing Strategy](#testing-strategy)
9. [Common Development Tasks](#common-development-tasks)
10. [Important Conventions](#important-conventions)
11. [Gotchas and Critical Notes](#gotchas-and-critical-notes)

---

## Project Overview

**Navign** is an indoor navigation and access control system designed for large buildings such as malls, airports, hospitals, and schools. The system combines:

- **BLE beacon-based indoor positioning** using ESP32-C3 microcontrollers
- **Real-time pathfinding** with multi-floor support (elevators, escalators, stairs)
- **Secure access control** using P-256 ECDSA cryptography and TOTP
- **Cross-platform mobile app** built with Vue 3 and Tauri 2
- **Robot fleet management** for autonomous delivery systems
- **Gesture recognition** for spatial understanding and interaction

**License:** MIT
**Version:** 0.1.0
**Primary Language:** Rust (with TypeScript, Go, Python, Swift)

### Key Use Cases

- Turn-by-turn indoor navigation in complex multi-floor buildings
- Contactless door/gate access control via mobile app
- Autonomous robot delivery coordination
- Environmental monitoring (temperature, humidity)
- Gesture-based spatial interaction

---

## Architecture

Navign is a **polyglot monorepo** with multiple interconnected components:

```
┌─────────────┐
│   Mobile    │──────┐
│  (Vue/Tauri)│      │
└─────────────┘      │
                     ▼
┌─────────────┐   ┌──────────┐   ┌─────────────┐
│   Beacon    │◄──┤  Server  │──►│    Admin    │
│  (ESP32-C3) │   │  (Axum)  │   │ (Rust + Go) │
└─────────────┘   └──────────┘   └─────────────┘
      │                │                 │
      │                │                 ▼
      │                │            ┌─────────┐
      │                │            │  Robot  │
      │                │            └─────────┘
      │                │
      └────────────────┴──────► MongoDB
```

### Data Flow Examples

**Indoor Positioning:**
1. Beacon broadcasts BLE advertisement
2. Mobile app receives RSSI signals from multiple beacons
3. Mobile calculates position via triangulation
4. Server provides pathfinding on demand

**Access Control:**
1. Mobile requests nonce from beacon via BLE
2. Beacon generates nonce, signs with private key
3. Mobile creates proof using user's private key + nonce
4. Beacon validates signature, controls relay/servo
5. Server logs access events

**Robot Delivery:**
1. Mobile app → Server: Delivery request
2. Server → Orchestrator: Task creation
3. Orchestrator: Robot selection algorithm
4. Orchestrator → Tower: gRPC streaming
5. Tower → Robot: Socket.IO task assignment
6. Robot → Server: Pathfinding query
7. Robot executes navigation, reports progress

---

## Technology Stack

### Backend (Rust)

**Server:** `server/`
- **Framework:** Axum 0.8.6 (async web framework)
- **Runtime:** Tokio 1.47.1 (async runtime)
- **Database:** MongoDB 3.3.0 (current primary), PostgreSQL via SQLx 0.8.6 (implemented, optional dual-database support)
- **Cryptography:** p256 0.13.2 (ECDSA), sha2 0.10.9, bcrypt 0.17.1, rsa 0.9.8
- **Authentication:** jsonwebtoken 10.0.0, oauth2 5.0.0 (GitHub, Google, WeChat)
- **Pathfinding:** bumpalo 3.18 (bump allocator for Dijkstra's algorithm)
- **Geo:** wkt 0.14.0 (Well-Known Text for polygons)

**Admin Orchestrator:** `admin/orchestrator/`
- **Framework:** Tonic 0.12 (gRPC server)
- **Protocol:** Protocol Buffers (task.proto)
- **Task Scheduling:** Custom robot selection algorithm

### Embedded (Rust)

**Firmware:** `firmware/`
- **HAL:** esp-hal 1.0.0-rc.1 (bare-metal, no RTOS initially)
- **BLE Stack:** bleps (async BLE protocol stack)
- **Radio:** esp-radio 0.16.0 (WiFi + BLE + coexistence)
- **RTOS:** esp-rtos 0.1.1 (FreeRTOS wrapper)
- **Networking:** smoltcp 0.12.0 (TCP/IP stack)
- **Crypto:** p256 0.13.2 (ECDSA), sha2 0.10.9
- **Sensors:** embedded-dht-rs 0.5.0 (DHT11 temp/humidity)
- **Storage:** esp-storage 0.8.0 (efuse key storage)

**Important:** Beacon requires `opt-level = "s"` for size optimization and performance.

### Frontend (TypeScript/Vue)

**Mobile:** `mobile/`
- **Framework:** Vue 3.5.18 (reactive UI)
- **Desktop/Mobile:** Tauri 2.8.1 (native wrapper)
- **State:** Pinia 3.0.3 (state management)
- **Router:** Vue Router 4.5.1
- **Forms:** Vee Validate 4.15.1 + Zod 4.1.1
- **UI Components:** Reka UI 2.4.1 (headless components)
- **Maps:** MapLibre GL 5.6.2 (vector maps)
- **Canvas:** Konva 9.3.22 + Vue Konva 3.2.2 (polygon rendering)
- **Styling:** Tailwind CSS 4.1.12 + Tailwind Merge
- **Build:** Vite (rolldown-vite), TypeScript 5.9.2
- **Testing:** Vitest 3.2.4
- **Plugins:**
  - tauri-plugin-blec (BLE communication)
  - tauri-plugin-sql (SQLite local storage)
  - tauri-plugin-biometric (Touch ID/Face ID)
  - tauri-plugin-stronghold (secure credential storage)
  - tauri-plugin-nfc (NFC support)

### Backend (Go)

**Tower:** `admin/tower/`
- **WebSocket:** go-socket.io 1.7.0
- **gRPC Client:** google.golang.org/grpc 1.76.0
- **Concurrency:** One goroutine per robot connection

### Robot Upper Layer (Rust + Python)

**Robot Components:** `robot/`
- **Scheduler (Rust):** `robot/scheduler/` - Task coordination and management
- **Serial (Rust):** `robot/serial/` - UART bridge to STM32 lower controller
- **Network (Rust):** `robot/network/` - HTTP client for server communication
- **Vision (Python):** `robot/vision/` - Computer vision (YOLO, AprilTag, MediaPipe)
- **Audio (Python):** `robot/audio/` - Wake word, speech recognition, TTS
- **Messaging:** Zenoh pub/sub for inter-component communication
- **Protocol:** Protocol Buffers for message serialization
- **Package Manager:** uv (Python), cargo (Rust)

### Shared Libraries

**Shared:** `shared/`
- **no_std compatible** Rust library with multiple feature flags:
  - `heapless`: Embedded systems (mutually exclusive with `alloc`)
  - `alloc`: Heap allocation (mutually exclusive with `heapless`)
  - `std`: Standard library features
  - `serde`: Serialization support
  - `crypto`: Cryptographic primitives
  - `mongodb`: MongoDB BSON integration
  - `sql`: SQL/SQLite integration
  - `postgres`: PostgreSQL integration (requires `std`, `serde`, `sql`)
  - `base64`: Base64 encoding
  - `postcard`: Efficient binary serialization (used for BLE protocol)
  - `defmt`: Embedded debugging and logging
  - `geo`: Geographic/geometric types
  - `chrono`: Date and time handling
  - `ts-rs`: TypeScript type generation (compile-time)

**Critical:** Never enable both `heapless` and `alloc` features simultaneously.

**TypeScript Schema Generator:** `ts-schema/`
- **Purpose:** Automatic Rust→TypeScript type conversion
- **Technology:** ts-rs derive macros
- **Output:** TypeScript definitions for mobile app
- **Command:** `just gen-ts-schema`

---

## Repository Structure

```
navign/
├── server/                      # Axum REST API server
│   ├── src/
│   │   ├── main.rs              # 209 lines - Axum router setup
│   │   ├── database.rs          # MongoDB connection
│   │   ├── kernel/              # Core business logic
│   │   │   ├── auth/            # OAuth2 + password auth
│   │   │   ├── route/           # Pathfinding algorithms
│   │   │   │   ├── implementations/ # Dijkstra, graph building
│   │   │   │   └── types/       # Area, Entity, Connection
│   │   │   ├── unlocker/        # Access control instances
│   │   │   └── totp.rs          # TOTP generation
│   │   └── schema/              # MongoDB data models
│   └── Cargo.toml               # Dependencies
│
├── firmware/                    # ESP32-C3 BLE firmware
│   ├── src/bin/
│   │   ├── main.rs              # 342 lines - BLE advertising + GATT
│   │   ├── crypto/              # P-256 ECDSA, nonce, proof
│   │   ├── ble/                 # BLE protocol, manager
│   │   ├── storage/             # Efuse key storage, nonce manager
│   │   ├── execute/             # Relay/servo control
│   │   └── ota.rs               # OTA update manager
│   ├── OTA_INTEGRATION.md       # OTA integration guide
│   └── Cargo.toml               # ESP-specific dependencies
│
├── mobile/                      # Cross-platform mobile app
│   ├── src/
│   │   ├── main.ts              # Vue app entry point
│   │   ├── views/               # Vue pages
│   │   ├── components/          # UI components
│   │   │   ├── map/             # MapLibre + Konva integration
│   │   │   └── ui/              # Reka UI components
│   │   ├── lib/                 # Utilities
│   │   │   └── api/tauri.ts     # Tauri backend communication
│   │   ├── schema/              # TypeScript type definitions
│   │   └── states/              # Pinia stores
│   ├── src-tauri/               # Rust backend (Tauri)
│   │   ├── src/lib.rs           # Tauri commands, BLE, crypto
│   │   └── Cargo.toml           # Tauri dependencies
│   ├── package.json
│   └── justfile                 # Mobile-specific tasks
│
├── admin/                       # Robot fleet management and maintenance
│   ├── proto/                   # Protocol Buffer definitions
│   │   ├── task.proto           # Robot task management (OrchestratorService)
│   │   ├── plot.proto           # Polygon extraction (PlotService)
│   │   └── sync.proto           # Orchestrator-server sync (OrchestratorSync)
│   ├── orchestrator/            # Rust gRPC server
│   │   ├── src/main.rs          # Task assignment logic
│   │   └── build.rs             # Protobuf compilation
│   ├── plot/                    # Python plot extraction client
│   │   ├── plot_client.py       # gRPC client for polygon extraction
│   │   ├── generate_proto.sh    # Proto code generation script
│   │   └── proto/               # Generated Python protobuf code
│   ├── tower/                   # Go Socket.IO server
│   │   ├── cmd/tower/main.go    # Server entry point
│   │   ├── internal/
│   │   │   ├── controller/      # gRPC client
│   │   │   ├── robot/           # Robot state management
│   │   │   └── socket_server/   # Socket.IO server
│   │   ├── Makefile             # Proto generation (use justfile instead)
│   │   └── go.mod
│   └── maintenance/             # ESP32-C3 key management CLI (Rust)
│       ├── src/main.rs          # CLI for eFuse key programming
│       └── Cargo.toml           # Dependencies
│
├── robot/                       # Robot components
│   ├── proto/                   # Protocol Buffer definitions
│   │   ├── common.proto         # Shared types
│   │   ├── vision.proto         # Vision service messages
│   │   ├── audio.proto          # Audio service messages
│   │   ├── scheduler.proto      # Task management messages
│   │   ├── serial.proto         # UART protocol messages
│   │   └── network.proto        # External communication messages
│   ├── scheduler/               # Rust task coordinator
│   │   ├── src/main.rs          # Main scheduler loop
│   │   ├── src/task_manager.rs  # Task queue management
│   │   ├── src/database.rs      # Task persistence
│   │   └── src/zenoh_client.rs  # Pub/sub messaging
│   ├── serial/                  # Rust UART bridge
│   │   └── src/main.rs          # Serial communication to STM32
│   ├── network/                 # Rust HTTP client
│   │   └── src/main.rs          # Server API client
│   ├── vision/                  # Python CV system
│   │   ├── service.py           # Zenoh service wrapper
│   │   ├── gesture.py           # Hand landmark detection
│   │   ├── detection.py         # YOLOv12 object detection
│   │   ├── locate.py            # AprilTag pose estimation
│   │   ├── transform.py         # 3D coordinate transforms
│   │   ├── calibrate.py         # Camera calibration
│   │   ├── config.example.py    # Configuration template
│   │   └── pyproject.toml       # uv dependencies
│   ├── audio/                   # Python audio system
│   │   ├── service.py           # Zenoh service wrapper
│   │   ├── waking.py            # Wake word detection
│   │   ├── recognition.py       # Speech-to-text
│   │   ├── play.py              # Text-to-speech
│   │   ├── config.example.py    # Configuration template
│   │   └── pyproject.toml       # uv dependencies
│   ├── lower/                   # STM32F407 lower controller (Embassy async)
│   │   ├── src/main.rs          # Motor control, sensors, actuators
│   │   └── Cargo.toml           # Embassy + STM32 HAL dependencies
│   └── README.md                # Robot architecture documentation
│
├── shared/                      # Shared Rust library (no_std)
│   ├── src/
│   │   ├── lib.rs               # Feature-gated exports
│   │   ├── schema/              # Area, Beacon, Entity, etc.
│   │   ├── ble/                 # BLE message protocol (Postcard serialization)
│   │   ├── crypto/              # Cryptographic helpers
│   │   ├── traits/              # Packetize/Depacketize
│   │   └── errors/              # Error types (thiserror)
│   └── Cargo.toml               # Multiple feature flags
│
├── proc_macros/                 # Procedural macros for code generation
│   ├── src/lib.rs               # Derive macros, attribute macros
│   ├── tests/macro_tests.rs     # Macro tests
│   └── Cargo.toml               # Proc-macro dependencies
│
├── ts-schema/                   # Rust → TypeScript schema generator (ts-rs)
│   ├── src/lib.rs               # Re-exports from shared
│   ├── bindings/generated/      # Generated TypeScript files
│   └── README.md                # Usage documentation
├── docs/                        # VitePress documentation site
│   └── docs/components/         # Component documentation
├── vision/                      # Apple Vision Pro app (Swift)
├── miniapp/                     # WeChat Mini Program (TypeScript)
├── animations/                  # Manim animations (Python)
├── presentation/                # Slidev presentation
├── schematics/                  # KiCad PCB designs
│
│
├── Cargo.toml                   # Rust workspace configuration
├── pnpm-workspace.yaml          # pnpm workspace + catalog
├── package.json                 # Root dependencies
├── justfile                     # Command runner (init, fmt, lint, test)
├── .github/workflows/ci.yml     # CI/CD pipeline
├── deny.toml                    # Cargo security policy
└── .typos.toml                  # Spell check configuration
```

---

## Component Details

### Server (`server/`)

**Purpose:** Centralized backend for navigation, access control, and entity management.

**Key Features:**
- RESTful API on port 3000
- MongoDB data persistence
- OAuth2 authentication (GitHub, Google, WeChat)
- Password-based authentication with bcrypt
- JWT token generation
- Multi-floor pathfinding (Dijkstra with bump allocation)
- TOTP generation for access control
- CORS enabled for cross-origin requests

**API Endpoints:**
```
GET  /                                    # Health check
GET  /health                              # Database ping
GET  /cert                                # Server public key (PEM)
POST /api/auth/register                   # User registration
POST /api/auth/login                      # User login

GET    /api/entities                      # Search entities
POST   /api/entities                      # Create entity
GET    /api/entities/{id}                 # Get entity
PUT    /api/entities                      # Update entity
DELETE /api/entities/{id}                 # Delete entity
GET    /api/entities/{id}/route           # Pathfinding

GET    /api/entities/{eid}/beacons        # List beacons
POST   /api/entities/{eid}/beacons        # Create beacon
GET    /api/entities/{eid}/beacons/{id}   # Get beacon
PUT    /api/entities/{eid}/beacons        # Update beacon
DELETE /api/entities/{eid}/beacons/{id}   # Delete beacon

GET    /api/entities/{eid}/areas          # List areas
POST   /api/entities/{eid}/areas          # Create area
GET    /api/entities/{eid}/areas/{id}     # Get area
PUT    /api/entities/{eid}/areas          # Update area
DELETE /api/entities/{eid}/areas/{id}     # Delete area

GET    /api/entities/{eid}/merchants      # List merchants
POST   /api/entities/{eid}/merchants      # Create merchant
GET    /api/entities/{eid}/merchants/{id} # Get merchant
PUT    /api/entities/{eid}/merchants      # Update merchant
DELETE /api/entities/{eid}/merchants/{id} # Delete merchant

GET    /api/entities/{eid}/connections    # List connections
POST   /api/entities/{eid}/connections    # Create connection
GET    /api/entities/{eid}/connections/{id} # Get connection
PUT    /api/entities/{eid}/connections    # Update connection
DELETE /api/entities/{eid}/connections/{id} # Delete connection

POST /api/entities/{eid}/beacons/{id}/unlocker                # Create unlock instance
PUT  /api/entities/{eid}/beacons/{id}/unlocker/{instance}/status  # Update status
PUT  /api/entities/{eid}/beacons/{id}/unlocker/{instance}/outcome # Record result
```

**Database Schema:**
- `entities`: Buildings (malls, hospitals, etc.)
- `areas`: Polygonal zones within entities
- `beacons`: BLE devices for positioning/access
- `merchants`: Stores, restaurants, facilities
- `connections`: Inter-area links (elevators, stairs)
- `users`: User accounts and authentication
- `beacon_secrets`: Private keys for beacons

**Pathfinding Algorithm:**
- Location: `server/src/kernel/route/implementations/`
- Uses Dijkstra's algorithm with bump allocation for performance
- Supports multi-floor routing via `Connection` entities
- Returns navigation instructions (ENTER_AREA, USE_CONNECTION, etc.)

**Environment Variables:**
```bash
DATABASE_URL=mongodb://localhost:27017
DATABASE_NAME=navign
RUST_LOG=info
```

---

### Firmware (`firmware/`)

**Purpose:** ESP32-C3 firmware for BLE advertising and access control.

**Hardware:**
- ESP32-C3 microcontroller (RISC-V, WiFi + BLE)
- DHT11 temperature/humidity sensor (GPIO 4)
- Button input (GPIO 3)
- Relay output (GPIO 7) - door/gate control
- LED indicator (GPIO 8)
- Human body sensor (GPIO 1) - PIR motion detection

**BLE Services:**
- `0x183D`: Authorization Control Service (if `UnlockGate` capability)
- `0x1819`: Location and Navigation Service
- `0x1821`: Indoor Positioning Service
- `0x181A`: Environmental Sensing Service (if `EnvironmentalData` capability)

**GATT Characteristics:**
```
Service UUID: 134b1d88-cd91-8134-3e94-5c4052743845
Characteristic UUID: 99d92823-9e38-72ff-6cf1-d2d593316af8
  - Read: Returns response messages
  - Write: Accepts request messages
  - Notify: Sends responses to subscribed clients
```

**BLE Protocol:**
1. **DeviceRequest** → **DeviceResponse**: Beacon type, capabilities, device ID
2. **NonceRequest** → **NonceResponse**: Fresh nonce + signature identifier
3. **UnlockRequest (proof)** → **UnlockResponse**: Success/failure + error code
4. **DebugRequest** → **DebugResponse**: Random data for testing

**Security:**
- Private key stored in ESP32 efuse `BLOCK_KEY0` (write-once, read-protected)
- P-256 ECDSA signature verification
- Nonce-based challenge-response (prevents replay attacks)
- Rate limiting: Max 5 unlock attempts per 5 minutes
- Nonce expiration: 5 seconds

**Device Types:**
```rust
enum DeviceType {
    Merchant,   // Commercial establishment
    Pathway,    // Navigation waypoint
    Connection, // Area junction
    Turnstile,  // Access gate
}
```

**Unlock Methods:**
```rust
enum UnlockMethod {
    Relay(Output),  // Digital relay control
    Servo(Servo),   // Servo motor control (not yet implemented)
    Infrared(IR),   // IR transmitter (not yet implemented)
}
```

**Flashing Instructions:**
```bash
# Requires esp-idf toolchain
cd firmware
cargo build --release
espflash flash target/riscv32imc-esp-espidf/release/navign-firmware
```

**Setting Private Key:**
```bash
cd admin/maintenance
cargo run -- fuse-priv-key --output-dir ./keys --port /dev/ttyUSB0
```

**OTA (Over-The-Air) Updates:**

The beacon firmware includes OTA update capability for remote firmware upgrades without physical access.

**Location:** `firmware/src/bin/ota.rs`

**Architecture:**
- Uses ESP-IDF bootloader OTA partition system
- Supports dual-bank updates (OTA0/OTA1 partitions)
- Automatic rollback on boot failure (if bootloader configured)
- WiFi and HTTP download code NOT included (to be implemented separately)

**Partition Layout:**
```
0x000000  Bootloader
0x010000  Factory (initial firmware)
0x110000  OTA0 (first update slot)
0x210000  OTA1 (second update slot)
0x310000  OTA Data (active partition tracker)
```

**Usage Example:**
```rust
use crate::ota::{OtaManager, OtaError, OtaState};
use esp_storage::FlashStorage;

// Initialize on boot
let flash = FlashStorage::new(peripherals.FLASH);
let mut ota_manager = OtaManager::new(flash)?;

// Mark current firmware as valid (prevents rollback)
ota_manager.mark_valid()?;

// Start OTA update (after downloading firmware via WiFi/HTTP)
ota_manager.begin_update(Some(firmware_size))?;

// Write firmware in chunks
for chunk in firmware_chunks {
    ota_manager.write_chunk(&chunk)?;
}

// Finalize and activate
ota_manager.finalize_update()?;
esp_hal::reset::software_reset();
```

**OTA State Machine:**
1. `Idle` - No update in progress
2. `Writing { bytes_written, total_size }` - Receiving firmware
3. `ReadyToActivate` - Write complete, ready to reboot

**Integration with Server:**
1. Server stores firmware binaries at `/api/firmwares/upload`
2. Orchestrator proxies firmware download at `/firmwares/:id/download`
3. Beacon WiFi implementation (future) downloads from orchestrator
4. Beacon OTA manager writes to flash partition
5. Reboot activates new firmware from OTA partition

**Security Considerations:**
- ⚠️ Firmware signature verification NOT yet implemented
- ⚠️ Checksum verification recommended before activation
- ⚠️ Encrypted firmware download recommended
- Rate limiting: Prevent excessive OTA attempts
- Rollback: Bootloader reverts if new firmware fails to boot

**WiFi/HTTP Integration (To Be Implemented):**
```rust
// Future WiFi-based OTA (not yet implemented)
async fn download_and_update(
    ota_manager: &mut OtaManager,
    server_url: &str,
) -> Result<(), OtaError> {
    // 1. Connect to WiFi
    let wifi = connect_wifi().await?;

    // 2. Query orchestrator for latest firmware
    let firmware = http_get(
        &format!("{}/firmwares/latest/esp32c3", server_url)
    ).await?;

    // 3. Download and write firmware
    ota_manager.begin_update(Some(firmware.size))?;
    let mut stream = http_download(&firmware.download_url).await?;
    while let Some(chunk) = stream.next().await {
        ota_manager.write_chunk(&chunk)?;
    }

    // 4. Verify checksum (important!)
    verify_checksum(&firmware)?;

    // 5. Activate and reboot
    ota_manager.finalize_update()?;
    esp_hal::reset::software_reset();
    Ok(())
}
```

**BLE-Based OTA (Alternative):**
- Firmware can be pushed via BLE chunks
- Slower than WiFi but works without network infrastructure
- Requires BLE message protocol extension (not yet implemented)

**Dependencies:**
```toml
esp-bootloader-esp-idf = "0.1"
esp-storage = "0.8"
embedded-storage = "0.3"
```

**Documentation:** See `firmware/OTA_INTEGRATION.md` for complete integration guide.

---

### Mobile (`mobile/`)

**Purpose:** Cross-platform mobile/desktop app for navigation and access control.

**Platforms:**
- iOS (planned)
- Android (planned)
- macOS (tested)
- Windows (planned)
- Linux (planned)

**Architecture:**
- **Frontend:** Vue 3 SPA with TypeScript
- **Backend:** Rust (Tauri commands)
- **State Management:** Pinia stores
- **Routing:** Vue Router (file-based routes)
- **Local Database:** SQLite via tauri-plugin-sql
- **Secure Storage:** Stronghold (encrypted credential vault)

**Key Features:**
1. **Indoor Positioning:**
   - Scans BLE beacons via tauri-plugin-blec
   - RSSI triangulation for position calculation
   - Real-time position updates on map

2. **Navigation:**
   - MapLibre GL for base map rendering
   - Konva canvas for polygon overlays (areas)
   - Turn-by-turn instructions
   - Multi-floor support with floor selector

3. **Access Control:**
   - BLE communication with beacons
   - P-256 ECDSA signature generation
   - Biometric authentication (Touch ID, Face ID)
   - NFC support (future)

4. **Offline Support:**
   - SQLite for caching entities, areas, merchants
   - Downloaded map tiles
   - Local pathfinding fallback (planned)

**Tauri Commands:**
```rust
// BLE operations
#[tauri::command]
fn ble_scan() -> Result<Vec<Beacon>>;

#[tauri::command]
fn ble_connect(address: String) -> Result<()>;

// Cryptography
#[tauri::command]
fn generate_proof(nonce: Vec<u8>, private_key: Vec<u8>) -> Result<Vec<u8>>;

// Database
#[tauri::command]
async fn sync_entities(db: State<'_, Database>) -> Result<()>;
```

**State Management:**
```typescript
// session.ts
interface SessionState {
  user: User | null;
  token: string | null;
  currentEntity: Entity | null;
  currentFloor: string;
  currentPosition: { x: number; y: number } | null;
}
```

**Build Commands:**
```bash
cd mobile
pnpm install
pnpm run dev              # Development mode
pnpm run build            # Production build
pnpm run tauri dev        # Tauri development
pnpm run tauri build      # Create app bundle
```

---

### Admin (`admin/`)

**Purpose:** Robot fleet management system with task orchestration and floor plan processing.

**Architecture:** Multi-component design with centralized protocol buffers

#### Protocol Buffers (`admin/proto/`)

All admin components share protocol buffer definitions:

- **task.proto** - Robot task management
  - `OrchestratorService`: Task assignment and robot status reporting
  - Used by: Orchestrator (server), Tower (client)

- **plot.proto** - Floor plan polygon extraction
  - `PlotService`: Polygon extraction from floor plans
  - Used by: Plot (implements service logic, though currently runs locally)

- **sync.proto** - Orchestrator-central server synchronization
  - `OrchestratorSync`: Event streaming, data sync, firmware distribution
  - Used by: Future central server integration

#### Orchestrator (Rust)

**Location:** `admin/orchestrator/`

**Responsibilities:**
- Task queue management
- Robot registry and state tracking
- Robot selection algorithm
- Task assignment decisions
- gRPC server for Tower communication

**gRPC Service (from task.proto):**
```protobuf
service OrchestratorService {
  rpc ReportRobotStatus(RobotReportRequest) returns (RobotReportResponse);
  rpc GetTaskAssignment(RobotDistributionRequest) returns (stream TaskAssignment);
}

message Task {
  string id = 1;
  TaskType type = 2;
  repeated Location sources = 3;
  repeated Location terminals = 4;
  Priority priority = 5;
  int64 created_at = 6;
  string entity_id = 7;
  map<string, string> metadata = 8;
}

message Location {
  double x = 1;
  double y = 2;
  double z = 3;
  string floor = 4;
}
```

**Robot Selection Algorithm:**
1. Filter robots by entity_id
2. Filter by state == IDLE
3. Filter by battery > 20%
4. Calculate distance to task source
5. Select closest robot
6. Mark robot as BUSY

#### Tower (Go)

**Location:** `admin/tower/`

**Responsibilities:**
- Socket.IO WebSocket server for robots
- gRPC client to Orchestrator
- One goroutine per robot connection
- Status reporting aggregation

**Socket.IO Events:**
```go
// Client → Server
socket.On("robot_register", RobotRegisterPacket)
socket.On("status_update", RobotStatusPacket)
socket.On("task_update", TaskUpdatePacket)

// Server → Client
socket.Emit("task_assigned", TaskAssignedPacket)
socket.Emit("task_cancelled", TaskCancelledPacket)
```

**Proto Generation:**
```bash
# From root justfile
just proto-tower
# Or from tower directory using Makefile
cd admin/tower && make proto
```

#### Plot (Python)

**Location:** `admin/plot/`

**Purpose:** Floor plan polygon extraction using computer vision.

**Responsibilities:**
- Extract polygons from floor plan images using OpenCV
- Local processing (does not require a gRPC server)
- Defines PlotService interface in plot.proto for future service integration

**Current Implementation:**
- Client performs local polygon extraction using OpenCV
- Implements `_extract_polygons_opencv()` method (placeholder - to be implemented)
- Can be extended to call a remote PlotService in the future

**Proto Generation:**
```bash
# From root justfile
just proto-plot
# Or from plot directory
cd admin/plot && ./generate_proto.sh
```

**Usage:**
```bash
cd admin/plot
uv sync
uv run python plot_client.py <floor_plan_image.png> [entity_id] [floor_id]
```

**Environment Variables:**
```bash
# Orchestrator
RUST_LOG=info
ORCHESTRATOR_ADDR=[::1]:50051

# Tower
ENTITY_ID=mall-123
ORCHESTRATOR_ADDR=localhost:50051
TOWER_ADDR=http://[::1]:8080
```

---

### Shared (`shared/`)

**Purpose:** no_std compatible Rust library for cross-component schemas.

**Feature Flags:**
```toml
[features]
default = ["std", "serde"]
heapless = []           # Embedded (Vec → heapless::Vec)
alloc = []              # Heap allocation
std = ["alloc"]         # Standard library
serde = []              # Serialization
crypto = []             # Cryptographic primitives
mongodb = ["serde"]     # MongoDB BSON support
sql = ["serde"]         # SQLite support
base64 = []             # Base64 encoding
```

**Critical:** `heapless` and `alloc` are mutually exclusive.

**Schemas:**
```rust
// Core schemas (alloc feature)
pub struct Entity { /* ... */ }
pub struct Area { /* ... */ }
pub struct Beacon { /* ... */ }
pub struct Merchant { /* ... */ }
pub struct Connection { /* ... */ }

// Mobile schemas (sql feature)
pub struct EntityMobile { /* ... */ }
pub struct AreaMobile { /* ... */ }

// Authentication (serde + alloc)
pub struct LoginRequest { /* ... */ }
pub struct RegisterRequest { /* ... */ }
pub struct AuthResponse { /* ... */ }
pub struct TokenClaims { /* ... */ }

// Account (mongodb feature)
pub struct Account { /* ... */ }
```

**BLE Protocol:**

Uses **Postcard** serialization (efficient binary format):

```rust
pub enum BleMessage {
    DeviceRequest,
    DeviceResponse(DeviceTypes, DeviceCapabilities, [u8; 24]),
    NonceRequest,
    NonceResponse(Nonce, [u8; 8]),
    UnlockRequest(Proof),
    UnlockResponse(bool, Option<CryptoError>),
    DebugRequest(Vec<u8>),
    DebugResponse(Vec<u8>),
}

pub trait Packetize {
    fn packetize(&self) -> Result<Vec<u8>, PacketizeError>;
}

pub trait Depacketize {
    fn depacketize(data: &[u8]) -> Result<Self, DepacketizeError>;
}
```

**Migration Note:** The project migrated from custom binary protocol to Postcard (commit #62) for better performance, smaller binary size, and standard Rust serialization.

---

### Robot Upper Layer (`robot/`)

**Purpose:** Distributed control system for autonomous delivery robots with modular components.

**Architecture:** Multi-component system using Zenoh pub/sub messaging and Protocol Buffers.

#### Overview

The robot upper layer consists of multiple specialized components that communicate via a **Zenoh** message bus. Each component is responsible for a specific aspect of robot operation:

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

#### Protocol Buffers (`robot/proto/`)

Unified message definitions for inter-component communication:

**Files:**
- `common.proto` - Shared types (`Location`, `Timestamp`, `RobotStatus`)
- `vision.proto` - Vision service (`ObjectDetection`, `AprilTagPose`, `HandGesture`)
- `audio.proto` - Audio service (`WakeWordEvent`, `SpeechRecognition`, `TTSRequest`)
- `scheduler.proto` - Task management (`Task`, `TaskSubmission`, `TaskUpdate`)
- `serial.proto` - UART protocol (`MotorCommand`, `SensorData`, `IMUReading`)
- `network.proto` - External comms (`PathfindingRequest`, `EntityDataRequest`)

**Generation:**
```bash
just proto-robot         # Generate all protobuf code (Rust + Python)
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
- Task history persistence in database

**Key Dependencies:**
- `zenoh` - Distributed pub/sub messaging
- `tokio` - Async runtime
- `tonic` - gRPC client (for Tower communication)
- `prost` - Protocol buffer serialization

**Zenoh Topics (Published):**
- `robot/scheduler/status` - Robot state updates
- `robot/scheduler/task/ack` - Task acknowledgments

**Zenoh Topics (Subscribed):**
- `robot/scheduler/task/submit` - Incoming tasks from Tower
- `robot/network/pathfinding/response` - Navigation paths
- `robot/serial/sensors` - Sensor data from lower layer
- `robot/vision/updates` - Vision detections
- `robot/audio/events` - Wake word events

**Run:**
```bash
cd robot/scheduler
cargo run
```

**Environment Variables:**
- `ZENOH_CONFIG` - Zenoh configuration file (optional)
- `DATABASE_URL` - Task database connection string

#### Serial (`robot/serial/`)

**Language:** Rust
**Purpose:** UART bridge to STM32 lower controller

**Features:**
- Bidirectional communication with lower controller
- Postcard binary serialization for efficiency
- Async serial I/O with `tokio_serial`
- Automatic reconnection on disconnect
- Publishes sensor data to Zenoh

**Protocol:**
- **Baud Rate:** 115200
- **Serialization:** Postcard (binary, compatible with firmware)
- **Frame Format:** Length-prefixed messages

**Key Messages:**
- `MotorCommand` - Motor speed/direction control
- `SensorDataRequest` - Request sensor readings
- `SensorDataResponse` - IMU, encoders, ultrasonic data
- `StatusUpdate` - Robot health/battery status

**Zenoh Topics (Published):**
- `robot/serial/sensors` - Sensor data from STM32
- `robot/serial/status` - Lower controller health

**Zenoh Topics (Subscribed):**
- `robot/serial/command` - Motor commands from scheduler

**Run:**
```bash
cd robot/serial
SERIAL_PORT=/dev/ttyUSB0 cargo run
```

**Environment Variables:**
- `SERIAL_PORT` - Default: `/dev/ttyUSB0`
- `SERIAL_BAUD` - Default: `115200`

#### Network (`robot/network/`)

**Language:** Rust
**Purpose:** External HTTP communication with Navign server

**Features:**
- RESTful API client for server
- Pathfinding request/response handling
- Entity and area data fetching
- Response caching for offline operation
- Future: BLE operations for beacon interaction

**API Integration:**
- `GET /api/entities/{id}/route` - Pathfinding queries
- `GET /api/entities/{id}` - Entity metadata
- `GET /api/entities/{eid}/areas` - Area polygons
- `GET /api/entities/{eid}/beacons` - Beacon locations

**Zenoh Topics (Published):**
- `robot/network/pathfinding/response` - Navigation paths from server
- `robot/network/entity/data` - Entity/area data

**Zenoh Topics (Subscribed):**
- `robot/network/pathfinding/request` - Pathfinding requests from scheduler
- `robot/network/entity/request` - Entity data requests

**Run:**
```bash
cd robot/network
SERVER_URL=http://localhost:3000 cargo run
```

**Environment Variables:**
- `SERVER_URL` - Default: `http://localhost:3000`
- `ENTITY_ID` - Robot's entity ID for navigation

#### Vision Service (`robot/vision/`)

**Language:** Python
**Purpose:** Computer vision processing (formerly `gesture_space`)

**Capabilities:**
- **Object Detection:** YOLOv12 real-time detection
- **Pose Estimation:** AprilTag-based camera localization
- **Hand Tracking:** MediaPipe hand landmarks (21 points per hand)
- **Finger Pointing:** 3D direction detection from hand poses
- **Gesture Recognition:** Neural network classification
- **3D Localization:** 2D→3D coordinate transformation

**Technologies:**
- OpenCV for image processing and camera calibration
- Ultralytics YOLOv12 for object detection
- MediaPipe for hand tracking
- pupil-apriltags for pose estimation
- PyTorch for gesture classification

**Zenoh Topics (Published):**
- `robot/vision/objects` - Detected objects with bounding boxes
- `robot/vision/pose` - Camera pose (position + rotation)
- `robot/vision/gestures` - Classified hand gestures
- `robot/vision/pointing` - Finger directions in 3D space

**Configuration:**
```bash
cd robot/vision
cp config.example.py config.py
# Edit: camera index, YOLO model, AprilTag positions
```

**Calibration:**
```bash
uv run python calibrate.py
# Detects chessboard, generates assets/interstices.npz
```

**Run:**
```bash
cd robot/vision
uv sync
uv run python service.py
```

**Environment Variables:**
- `CAMERA_INDEX` - Default: `0`
- `YOLO_MODEL` - Default: `yolo12n.pt`

**See:** `robot/vision/README.md` for complete documentation

#### Audio Service (`robot/audio/`)

**Language:** Python
**Purpose:** Voice interaction and audio feedback

**Capabilities:**
- **Wake Word Detection:** Porcupine-based activation (migrating to OpenWakeWord)
- **Speech Recognition:** Wav2Vec2 speech-to-text
- **Text-to-Speech:** Edge TTS voice synthesis
- **Audio Recording:** Voice activity detection with silence detection
- **Audio Playback:** Cross-platform with pygame

**Technologies:**
- pvporcupine for wake word detection
- transformers (Wav2Vec2) for speech recognition
- edge-tts for text-to-speech synthesis
- pyaudio for audio I/O
- pygame for playback

**Zenoh Topics (Published):**
- `robot/audio/wake_word` - Wake word detected events
- `robot/audio/transcription` - Speech recognition results
- `robot/audio/events` - Audio state changes

**Configuration:**
```bash
cd robot/audio
cp config.example.py config.py
# Add PORCUPINE_KEY from https://console.picovoice.ai/
# Configure: TTS voice, wake word sensitivity, silence threshold
```

**Run:**
```bash
cd robot/audio
uv sync
uv run python service.py
```

**Environment Variables:**
- `PORCUPINE_ACCESS_KEY` - Required for wake word detection

**See:** `robot/audio/README.md` for complete documentation

#### Communication Flow Example

**Delivery Task Execution:**

1. **Tower → Scheduler** (Socket.IO):
   - `TaskSubmission` with source/destination locations

2. **Scheduler → Network** (Zenoh: `robot/network/pathfinding/request`):
   - `PathfindingRequest` with entity_id, start, end

3. **Network → Server** (HTTP):
   - `GET /api/entities/{id}/route?from_x=...&to_x=...`

4. **Network → Scheduler** (Zenoh: `robot/network/pathfinding/response`):
   - `PathfindingResponse` with waypoints and instructions

5. **Scheduler → Serial** (Zenoh: `robot/serial/command`):
   - `MotorCommand` with speed/direction

6. **Serial → Lower** (UART - Postcard):
   - Binary serialized motor commands

7. **Lower → Serial** (UART - Postcard):
   - Binary serialized sensor data

8. **Serial → Scheduler** (Zenoh: `robot/serial/sensors`):
   - `SensorDataResponse` with IMU, encoders, ultrasonic

9. **Scheduler → Tower** (gRPC stream):
   - `TaskUpdateReport` with progress and current position

**Deployment:**

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

**Production:** Use systemd/supervisor for process management (see `robot/README.md`)

---

### Gesture Space (Deprecated - Now `robot/vision/`)

**Purpose:** Computer vision system for gesture recognition and spatial understanding.

**⚠️ Status:** This module has been moved to `robot/vision/` as part of the robot upper layer reorganization (PR #79).

**Features:**
1. **Hand Landmark Detection** (MediaPipe):
   - 21 hand landmarks per hand
   - Real-time tracking at 30+ FPS
   - Gesture classification

2. **Object Detection** (YOLOv12):
   - Real-time object detection
   - Custom trained models
   - Bounding box + confidence

3. **AprilTag Detection:**
   - Marker-based pose estimation
   - 3D coordinate transformation
   - Camera-to-world transforms

4. **Wake Word Detection** (Porcupine):
   - Offline keyword spotting
   - Low-latency activation

**Dependencies:**
```toml
[project.dependencies]
mediapipe = "*"
opencv-python = "*"
ultralytics = "*"  # YOLOv12
torch = "*"
apriltag = "*"
pvporcupine = "*"
numpy = "*"
```

**Usage:**
```bash
cd gesture_space
uv sync
uv run python main.py
```

---

### Robot Lower Controller (`robot/lower/`)

**Purpose:** Low-level motor control and sensor management for autonomous delivery robots.

**Hardware:**
- STM32F407ZG microcontroller (ARM Cortex-M4F, 168 MHz)
- Motor drivers for differential drive
- Sensor interfaces (encoders, IMU, ultrasonic)
- Serial communication with upper controller

**Software Architecture:**
- **Runtime:** Embassy async executor (async embedded Rust)
- **HAL:** embassy-stm32 0.4.0
- **Features:**
  - Async task scheduling
  - Real-time motor control
  - Sensor data acquisition
  - Inter-processor communication (UART/SPI)
  - defmt logging for debugging

**Key Dependencies:**
```toml
embassy-executor = { version = "0.9.1", features = ["arch-cortex-m"] }
embassy-stm32 = { version = "0.4.0", features = ["stm32f407zg"] }
embassy-time = "0.5.0"
defmt = "1.0.1"
cortex-m-rt = "0.7.0"
```

**Build & Flash:**
```bash
cd robot/lower
cargo build --release
# Flash using probe-rs or OpenOCD
probe-rs run --chip STM32F407ZGTx
```

**Current Status:** Basic structure implemented, motor control logic in development.

---

### Procedural Macros (`proc_macros/`)

**Purpose:** Compile-time code generation for reducing boilerplate across the project.

**Location:** `proc_macros/`

**Features:**
- Custom derive macros
- Attribute macros for configuration
- Function-like macros for repetitive patterns

**Dependencies:**
- syn 2.0 - Parse Rust syntax
- quote 1.0 - Generate Rust code
- proc-macro2 1.0 - Procedural macro utilities

**Usage Example:**
```rust
use navign_proc_macros::ExampleDerive;

#[derive(ExampleDerive)]
struct MyStruct {
    field: String,
}
```

**Development:**
```bash
cd proc_macros
cargo check
cargo test  # Tests run in dependent crates
```

**Note:** Currently contains example macros. Real implementations to be added based on project needs.

---

### Internationalization (`mobile/src/i18n/`)

**Purpose:** Multi-language support for the mobile application.

**Supported Languages:**
1. **English (en-US)** - Default/fallback
2. **Simplified Chinese (zh-CN)** - 简体中文
3. **Traditional Chinese (zh-TW)** - 繁體中文
4. **Japanese (ja-JP)** - 日本語
5. **French (fr-FR)** - Français

**Technology:** vue-i18n with JSON translation files

**Structure:**
```
mobile/src/i18n/
├── index.ts              # i18n configuration
├── locales/
│   ├── en-US.json
│   ├── zh-CN.json
│   ├── zh-TW.json
│   ├── ja-JP.json
│   └── fr-FR.json
└── README.md
```

**Usage in Components:**
```vue
<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
</script>

<template>
  <h1>{{ t('home.title') }}</h1>
  <Button>{{ t('common.login') }}</Button>
</template>
```

**Features:**
- Automatic language detection
- Persistent language preference (localStorage)
- Language switcher component
- Parameterized translations
- Organized by feature (common, auth, navigation, etc.)

---

### PostgreSQL Migration Layer (`server/src/pg/`)

**Purpose:** Optional PostgreSQL database support alongside MongoDB for gradual migration.

**Status:** ✅ **Implemented** (previously planned, now complete)

**Architecture:**
- Dual-database support (MongoDB primary, PostgreSQL optional)
- Repository pattern for clean abstraction
- Type-safe ID handling (UUID for entities/users, Integer for others)
- Automatic schema migrations

**Key Files:**
- `src/pg/pool.rs` - Connection pooling
- `src/pg/models.rs` - PostgreSQL-specific models
- `src/pg/repository.rs` - CRUD repository implementations
- `migrations/001_initial_schema.sql` - Complete schema definition

**Environment Variables:**
```bash
# Required (MongoDB)
MONGODB_HOST=localhost:27017
MONGODB_DB_NAME=navign

# Optional (PostgreSQL)
POSTGRES_URL=postgresql://user:password@localhost:5432/navign
POSTGRES_RUN_MIGRATIONS=true  # Auto-run migrations
```

**ID Types:**
- **UUID**: entities, users (globally unique, top-level resources)
- **Integer (SERIAL)**: areas, beacons, merchants, connections (efficient joins)

**Repositories:**
- ✅ EntityRepository (UUID-based)
- ✅ UserRepository (UUID-based)
- ✅ AreaRepository (Integer-based)
- ✅ BeaconRepository (Integer-based)
- ✅ MerchantRepository (Integer-based)
- ✅ ConnectionRepository (Integer-based)
- ✅ BeaconSecretRepository
- ✅ UserPublicKeyRepository
- ✅ FirmwareRepository

**Usage Example:**
```rust
// Dual-database handler
async fn my_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    if let Some(pg_pool) = state.pg_pool.as_ref() {
        // Use PostgreSQL
        let repo = EntityRepository::new(pg_pool.clone());
        let entities = repo.get_all(0, 10).await?;
    } else {
        // Fallback to MongoDB
        let entities = Entity::get_all(&state.db).await?;
    }
    Ok(Json(entities))
}
```

**Migration Strategy (4 Phases):**
1. **Phase 1 (Current)**: PostgreSQL layer exists, MongoDB only in use
2. **Phase 2**: Dual-write mode - Write to both, read from MongoDB
3. **Phase 3**: Dual-read mode - Write to both, read from PostgreSQL
4. **Phase 4**: PostgreSQL only - Remove MongoDB dependencies

**Documentation:** See `docs/docs/components/server/postgres-migration-summary.md` and `POSTGRES_MIGRATION.md`

---

## Development Workflow

### Initial Setup

```bash
# Clone repository
git clone <repository-url>
cd navign

# Run initialization (installs all tools and dependencies)
just init
# This will:
# - Install cargo-binstall, cargo-deny, cargo-shear, typos-cli
# - Enable corepack and install pnpm packages
# - Sync Python dependencies (animations, gesture_space)
# - Run cargo check
```

### Code Formatting

```bash
just fmt
# Formats:
# - TOML files (Taplo)
# - Python files (Ruff)
# - JavaScript/TypeScript/Vue (Prettier)
# - Rust files (cargo fmt)
```

**Check formatting without modifying:**
```bash
just fmt-check
```

### Linting

```bash
just lint
# Runs:
# - Taplo lint (TOML)
# - Ruff check (Python)
# - Clippy (Rust) with multiple feature flag combinations for shared/
# - Oxlint (TypeScript/Vue) with type-aware mode
# - Vue TSC (type checking)
```

### Testing

```bash
just test
# Runs:
# - shared/ tests with multiple feature combinations
# - server/ tests (requires MongoDB)
# - mobile/ tests (Vitest)
# - admin/maintenance/ tests
```

**Run specific component tests:**
```bash
cd server && cargo test
cd mobile && just test
cd shared && cargo test --features mongodb --features serde --features crypto
cd admin/maintenance && cargo test
```

### CI Tasks

The justfile includes CI-specific tasks for each component:

```bash
just ci-shared      # Shared library checks + tests
just ci-server      # Server checks + tests (needs MongoDB)
just ci-firmware    # Firmware checks + mock tests
just ci-mobile      # Mobile checks + tests
just ci-desktop     # Desktop-specific tasks
just ci-repo        # Repository-wide checks (Taplo, Typos)
just ci-proc-macros # Procedural macros checks + tests
just ci-tower       # Tower (Go) checks + tests
just ci-orchestrator # Orchestrator (Rust gRPC) checks + tests
just ci-plot        # Plot (Python) checks + tests
just ci-maintenance # Maintenance tool checks + tests
just ci-robot-lower # Robot/lower controller checks (embedded)
just ci-robot-upper # Robot/upper (scheduler, serial, network, vision, audio)
```

### Running Components

**Server:**
```bash
cd server
cargo run
# Listens on http://0.0.0.0:3000
```

**Mobile:**
```bash
cd mobile
pnpm run tauri dev
```

**Firmware:**
```bash
cd firmware
cargo build --release
espflash flash target/riscv32imc-esp-espidf/release/navign-firmware
```

**Admin Orchestrator:**
```bash
cd admin/orchestrator
cargo run
# gRPC server on [::1]:50051
```

**Admin Tower:**
```bash
cd admin/tower
go run cmd/tower/main.go
# Socket.IO server on [::1]:8080
```

---

## Security Considerations

### Cryptography

**Algorithms:**
- **P-256 ECDSA:** Public-key cryptography for beacons and mobile
- **SHA-256:** Hashing for integrity checks
- **HMAC-SHA1:** Message authentication (legacy TOTP)
- **bcrypt:** Password hashing (cost factor 12)
- **AES-GCM:** Authenticated encryption (mobile Tauri)
- **RSA:** Server key exchange (future)

**Key Storage:**
- **Beacon:** ESP32 efuse blocks (hardware-protected, write-once)
- **Mobile:** Tauri Stronghold (encrypted vault with OS keychain)
- **Server:** Environment variables (should use secret management in production)

### Access Control

**Nonce-Based Challenge-Response:**
```
1. Mobile → Beacon: NonceRequest
2. Beacon: nonce = random_bytes(32)
           store(nonce, timestamp)
           signature = sign(nonce, private_key)
           identifier = last_8_bytes(signature)
3. Beacon → Mobile: NonceResponse(nonce, identifier)
4. Mobile: proof = sign(nonce || device_id, user_private_key)
5. Mobile → Beacon: UnlockRequest(proof)
6. Beacon: verify_signature(proof, user_public_key)
           check_nonce_not_expired(nonce)  # 5 second TTL
           check_nonce_not_used(nonce)
           check_rate_limit()              # 5 attempts per 5 min
7. Beacon: activate_relay()
8. Beacon → Mobile: UnlockResponse(success, error)
```

**Rate Limiting:**
- Max 5 unlock attempts per 5 minutes per beacon
- Implemented in beacon firmware
- Uses rolling window with timestamps

### Authentication

**OAuth2 Flow (GitHub, Google, WeChat):**
```
1. Client → Server: GET /api/auth/{provider}/authorize
2. Server → Client: Redirect to provider
3. User authenticates with provider
4. Provider → Server: Authorization code
5. Server → Provider: Exchange code for access token
6. Server → Provider: Fetch user profile
7. Server: Create/update user in database
8. Server → Client: JWT token (24h expiration)
```

**Password Authentication:**
```
1. Client → Server: POST /api/auth/register
   { username, email, password }
2. Server: hash = bcrypt(password, cost=12)
           create_user(username, email, hash)
3. Server → Client: JWT token

Login:
1. Client → Server: POST /api/auth/login
   { username, password }
2. Server: user = find_by_username(username)
           verify = bcrypt_verify(password, user.password_hash)
3. Server → Client: JWT token if valid
```

**JWT Claims:**
```rust
pub struct TokenClaims {
    pub sub: String,        // User ID
    pub username: String,   // Username
    pub exp: i64,           // Expiration timestamp
    pub iat: i64,           // Issued at timestamp
}
```

### Input Validation

**Always validate:**
- Entity bounds for coordinates
- Floor identifiers match entity floors
- Polygon coordinates are valid
- UUIDs are properly formatted
- Device IDs are 24-character hex strings
- Nonce timestamps are within acceptable range

---

## Testing Strategy

### Unit Tests

**Server:** `server/src/`
```bash
cd server
cargo test
# Requires MongoDB on localhost:27017
```

**Shared:** `shared/src/`
```bash
cd shared
# Test all feature combinations
cargo test
cargo test --features heapless --no-default-features
cargo test --features alloc --no-default-features
cargo test --features crypto --features heapless --features serde --no-default-features
cargo test --features mongodb --features serde --features crypto
```

**Mobile:** `mobile/src/`
```bash
cd mobile
pnpm run test
# Uses Vitest
# See: mobile/src/lib/api/tauri.test.ts
#      mobile/src/components/map/extractInstructions.test.ts
```

### Integration Tests

**Firmware:** `firmware/tests/`

The firmware now has comprehensive testing infrastructure:

**Mock-Based Tests (Fast, runs on host):**
```bash
cd firmware

# Run all mock tests
just test-firmware-mocks

# Or individual test suites
cargo test --test nonce_tests --features std
cargo test --test crypto_tests --features std
cargo test --test rate_limit_tests --features std
```

**QEMU Simulation Tests (Requires QEMU):**
```bash
# Run firmware in ESP32-C3 QEMU
just test-firmware-qemu

# Or run directly
cd firmware && ./tests/qemu_runner.sh
```

**Test Coverage:**
| Component | Tests | Coverage |
|-----------|-------|----------|
| Nonce Management | 6 tests | 95%+ |
| Cryptography (P-256 ECDSA) | 8 tests | 90%+ |
| Rate Limiting | 8 tests | 90%+ |
| GPIO/Peripherals | 3 tests | 60%+ |
| BLE Protocol | 0 tests | 80%+ (TODO) |
| Storage (eFuse) | 0 tests | 85%+ (TODO) |

**Test Structure:**
```
firmware/tests/
├── README.md           # Testing guide
├── qemu_runner.sh      # QEMU automation
├── mocks/              # Mock implementations
│   ├── rng.rs          # Deterministic RNG
│   ├── storage.rs      # Mock flash storage
│   └── gpio.rs         # Mock GPIO/relay
├── nonce_tests.rs      # Nonce management tests
├── crypto_tests.rs     # Crypto tests
└── rate_limit_tests.rs # Rate limiting tests
```

**Documentation:** See `firmware/tests/README.md` and `firmware/TESTING.md` for complete guide.

### End-to-End Tests

**Not yet implemented**

Planned workflow:
1. Start MongoDB
2. Start server
3. Seed database with test entities/areas/beacons
4. Run mobile app in test mode
5. Simulate BLE beacons
6. Test navigation flow
7. Test access control flow

---

## Common Development Tasks

### Adding a New API Endpoint

1. **Define schema** in `server/src/schema/`:
```rust
// server/src/schema/my_entity.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyEntity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
}
```

2. **Implement Service trait** if using CRUD:
```rust
impl Service for MyEntity {
    const COLLECTION: &'static str = "my_entities";
    // Implement required methods
}
```

3. **Add route** in `server/src/main.rs`:
```rust
.route("/api/my-entities", get(MyEntity::get_handler))
.route("/api/my-entities", post(MyEntity::create_handler))
```

4. **Update shared/** if needed by mobile:
```rust
// shared/src/schema/my_entity.rs
#[cfg(feature = "alloc")]
pub struct MyEntity {
    // Same structure, but with feature gates
}
```

5. **Run code generation** for TypeScript:
```bash
cd ts-schema
cargo build --release
# Generates mobile/src/schema/my_entity.d.ts
```

### Adding a BLE Message Type

1. **Define in shared:**
```rust
// shared/src/ble/message.rs
pub enum BleMessage {
    // ... existing variants
    MyRequest(MyData),
    MyResponse(MyResult),
}
```

2. **Implement Packetize/Depacketize:**
```rust
impl Packetize for BleMessage {
    fn packetize(&self) -> Result<Vec<u8>, PacketizeError> {
        match self {
            BleMessage::MyRequest(data) => {
                // Serialize to bytes
            },
            // ...
        }
    }
}
```

3. **Update beacon handler:**
```rust
// firmware/src/bin/main.rs
match message {
    Some(BleMessage::MyRequest(data)) => {
        let result = process_my_request(data);
        Some(BleMessage::MyResponse(result))
    },
    // ...
}
```

4. **Update mobile Tauri command:**
```rust
// mobile/src-tauri/src/lib.rs
#[tauri::command]
fn my_ble_operation(data: MyData) -> Result<MyResult> {
    // Send BLE message, wait for response
}
```

### Adding a New Pathfinding Instruction

1. **Define instruction type:**
```rust
// server/src/kernel/route/instructions.rs
pub enum NavigationInstruction {
    // ... existing variants
    MyInstruction { param: String },
}
```

2. **Generate instruction in pathfinding:**
```rust
// server/src/kernel/route/implementations/navigate.rs
fn generate_instructions(path: &[Node]) -> Vec<NavigationInstruction> {
    // ... logic to detect when to emit MyInstruction
}
```

3. **Handle in mobile:**
```typescript
// mobile/src/components/map/extractInstructions.ts
export function extractInstructions(route: Route): Instruction[] {
  // Parse MyInstruction and convert to UI format
}
```

### Adding a Device Capability

1. **Add to shared:**
```rust
// shared/src/ble/device_caps.rs
pub enum DeviceCapability {
    UnlockGate,
    EnvironmentalData,
    MyNewCapability,
}
```

2. **Update beacon advertised capabilities:**
```rust
// firmware/src/bin/main.rs
let mut capabilities = Vec::<DeviceCapability, 4>::new();
capabilities.push(DeviceCapability::MyNewCapability).unwrap();
```

3. **Add corresponding BLE service UUID if needed:**
```rust
uuids.push(Uuid::Uuid16(0x1234)).unwrap(); // My Service UUID
```

4. **Update mobile to handle capability:**
```typescript
// mobile/src/lib/api/tauri.ts
if (beacon.capabilities.includes('MyNewCapability')) {
  // Enable UI for this capability
}
```

---

## Important Conventions

### Rust Code Style

- **Edition:** 2024
- **Formatter:** `rustfmt` (default settings)
- **Linter:** `clippy` with `-D warnings` (deny all warnings)
- **Naming:**
  - `snake_case` for functions, variables, modules
  - `PascalCase` for types, traits, enums
  - `SCREAMING_SNAKE_CASE` for constants
- **Error Handling:**
  - Use `anyhow::Result` for applications (server, beacon)
  - Use custom error types for libraries (shared)
  - Always propagate errors with `?`, never `unwrap()` in production
- **Async:**
  - Prefer `async`/`await` over manual futures
  - Use Tokio for server, esp-rtos for beacon

### TypeScript Code Style

- **Formatter:** Prettier
- **Linter:** Oxlint with `--type-aware` mode
- **Type Safety:**
  - Enable strict mode in `tsconfig.json`
  - No `any` types
  - Prefer interfaces over types for objects
- **Vue:**
  - Use `<script setup lang="ts">` composition API
  - Define props with `defineProps<T>()`
  - Use Pinia for state, not component-level reactive objects

### Git Commit Messages

Follow conventional commits:
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style (formatting, no logic change)
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `test`: Add/modify tests
- `chore`: Maintenance tasks

**Scopes:**
- `server`, `beacon`, `mobile`, `shared`, `admin`, `gesture_space`, `docs`

**Examples:**
```
feat(beacon): add servo motor unlock support

Implements UnlockMethod::Servo for beacon access control.
Includes PWM control and angle calibration.

Closes #123
```

```
fix(server): correct multi-floor pathfinding

Dijkstra algorithm was not properly handling elevator
connections between floors. Added connection type checking.

Fixes #456
```

### Feature Flags

**Shared library MUST be compiled with correct features:**

For beacon (embedded):
```toml
navign-shared = { path = "../shared", default-features = false, features = [
  "heapless",
  "serde",
  "crypto",
] }
```

For server:
```toml
navign-shared = { path = "../shared", default-features = false, features = [
  "std",
  "serde",
  "mongodb",
  "crypto",
] }
```

For mobile Tauri:
```toml
navign-shared = { path = "../../shared", features = [
  "std",
  "serde",
  "sql",
  "crypto",
] }
```

---

## Gotchas and Critical Notes

### 1. Firmware Optimization Required

The firmware **MUST** be compiled with size optimization:

```toml
[profile.release.package.navign-firmware]
opt-level = 's'
codegen-units = 1
```

Without this, the binary will not fit in ESP32-C3 flash (4MB).

### 2. Shared Library Feature Conflicts

**NEVER enable both `heapless` and `alloc` features:**

```toml
# ❌ WRONG
navign-shared = { features = ["heapless", "alloc"] }

# ✅ CORRECT (embedded)
navign-shared = { features = ["heapless"] }

# ✅ CORRECT (server/desktop)
navign-shared = { features = ["alloc"] }
```

This is enforced by compile-time errors in `shared/src/lib.rs`.

### 3. Efuse Private Key Must Be Set

Beacons will panic on boot if `BLOCK_KEY0` efuse is not programmed:

```rust
let private_key = Efuse::read_field_le::<[u8; 32]>(BLOCK_KEY0);
if private_key == [0u8; 32] {
    panic!("EFUSE BLOCK_KEY0 is not set");
}
```

Use the `admin/maintenance` tool to program keys before deploying beacons.

### 4. MongoDB Required for Server Tests

Server tests will fail without MongoDB running:

```bash
# Start MongoDB first
docker run -d -p 27017:27017 mongo:8.0

# Then run tests
cd server && cargo test
```

### 5. Tauri Mobile Development is Different

Mobile development requires additional setup not covered by `just init`:

```bash
# Install Xcode (macOS/iOS)
# Install Android Studio + NDK (Android)
# See: https://tauri.app/v2/guides/prerequisites/
```

### 6. ESP32-C3 Requires Specific Toolchain

Beacon firmware requires the esp-idf toolchain:

```bash
# Install espup
cargo install espup
espup install
# Source the environment
. ~/export-esp.sh

# Then build beacon
cd beacon
cargo build --release
```

### 7. Bumpalo Arena Lifetime Management

The server pathfinding uses bump allocation for performance:

```rust
use bumpalo::Bump;

let arena = Bump::new();
let graph = build_graph(&arena, entity);
let path = dijkstra(&arena, graph, start, end);
// All allocations freed when arena drops
```

**Do not** try to return references from the arena - they won't outlive the function.

### 8. Nonce Replay Attack Prevention

Beacons store used nonces in a fixed-size buffer (16 nonces):

```rust
const MAX_NONCES: usize = 16;
```

If a beacon receives > 16 unlock requests within 5 seconds, old nonces are evicted.
This is acceptable because nonces expire after 5 seconds anyway.

### 9. CORS is Wide Open

The server has permissive CORS for development:

```rust
let cors = CorsLayer::new()
    .allow_origin(tower_http::cors::Any)
    .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(tower_http::cors::Any);
```

**TODO:** Restrict origins in production deployment.

### 10. No CI for Beacon Yet

The CI pipeline does not test beacon firmware:

```yaml
runs-for: [shared, server, mobile, desktop, beacon]
```

But the beacon job only checks compilation, no unit tests:

```just
ci-beacon:
  cd beacon && cargo check --release
  cd beacon && cargo fmt -- --check
  cd beacon && cargo clippy --release -- -D warnings
  echo "No tests for beacons yet..."
```

Embedded testing requires hardware or simulators (not yet configured).

### 11. TypeScript Schema Generation is Automated via ts-rs

After modifying `shared/src/schema/`, regenerate TypeScript types:

```bash
just gen-ts-schema
# Or manually:
cd shared && cargo test --features ts-rs
cp ts-schema/bindings/generated/*.ts mobile/src/schema/generated/
```

The `ts-rs` library automatically generates TypeScript definitions at compile-time.

**Important:** Always run `just gen-ts-schema` after adding/modifying shared types to keep mobile TypeScript definitions in sync.

### 12. pnpm Catalog Versioning

The monorepo uses pnpm's catalog feature for version management:

```yaml
# pnpm-workspace.yaml
catalog:
  vue: ^3.5.18
  vite: npm:rolldown-vite@latest
```

When adding dependencies to mobile or other pnpm packages, use `catalog:`:

```json
"dependencies": {
  "vue": "catalog:",
  "vite": "catalog:"
}
```

### 13. Tauri Plugin Versions

Tauri plugins use `~2` version range:

```toml
"@tauri-apps/plugin-biometric": "~2"
```

This means ">=2.0.0 <2.1.0". Always check compatibility with Tauri version.

### 14. Robot/Lower Component Now Implemented ✅

The robot lower layer (`robot/lower/`) is **now implemented** with STM32F407ZG + Embassy async runtime.

**Status:**
- ✅ `robot/lower` - Basic structure complete, motor control in development
- ❌ `robot/upper` - Not yet implemented (Raspberry Pi planned)
- ✅ Admin orchestration layer (Orchestrator + Tower) exists and functional

### 15. Robot Upper Layer is Distributed

The robot upper layer uses **Zenoh pub/sub messaging** for inter-component communication.

**Architecture:**
- Components are loosely coupled via message bus
- Each service publishes/subscribes to specific topics
- Protocol Buffers for type-safe serialization
- Can run components on different machines/containers

**Important:** All robot components must have access to the same Zenoh network.

### 16. MongoDB + PostgreSQL Dual-Database Support ✅

The PostgreSQL migration layer is **now implemented** (previously planned).

**Current State:**
- ✅ PostgreSQL repository layer complete with all CRUD operations
- ✅ Dual-database support - can run with MongoDB only or MongoDB + PostgreSQL
- ✅ Automatic schema migrations
- ✅ Type-safe UUID and Integer ID handling
- 📋 Migration in progress - MongoDB still primary, PostgreSQL optional
- 📋 Gradual 4-phase migration strategy documented

See `docs/docs/components/server/postgres-migration-summary.md` for details.

---

## Documentation References

### Official Documentation

- **VitePress Docs:** `docs/` directory (run `pnpm run docs:dev`)
- **Component Docs:**
  - Server: `docs/docs/components/server.md`
  - Beacon: `docs/docs/components/beacon.md`
  - Mobile: `docs/docs/components/mobile.md`
  - Admin: `docs/docs/components/admin/index.md`

### External Documentation

- **Tauri:** https://tauri.app/v2/
- **Vue 3:** https://vuejs.org/guide/
- **Axum:** https://docs.rs/axum/
- **esp-hal:** https://docs.esp-rs.org/esp-hal/
- **Protocol Buffers:** https://protobuf.dev/

### API Reference

Generate Rust API docs:
```bash
cargo doc --open --no-deps
```

---

## Quick Reference

### Most Common Commands

```bash
just init          # Initial setup (run once)
just fmt           # Format all code
just lint          # Lint all code
just test          # Run all tests
just check         # Type checking (subset of lint)
just clean         # Clean build artifacts
just ci-server     # Server CI tasks
just ci-mobile     # Mobile CI tasks
```

### Port Assignments

- **3000:** Server REST API
- **8080:** Admin Tower (Socket.IO)
- **50051:** Admin Orchestrator (gRPC)

### Important File Locations

- **Server Entry:** `server/src/main.rs:64`
- **Firmware Entry:** `firmware/src/bin/main.rs:66`
- **Mobile Entry:** `mobile/src/main.ts`
- **Shared Exports:** `shared/src/lib.rs`
- **Tauri Commands:** `mobile/src-tauri/src/lib.rs`
- **Pathfinding:** `server/src/kernel/route/implementations/navigate.rs`
- **BLE Protocol:** `shared/src/ble/message.rs`
- **Admin Proto:** `admin/proto/task.proto`, `admin/proto/plot.proto`, `admin/proto/sync.proto`
- **Robot Proto:** `robot/proto/` - `common.proto`, `vision.proto`, `audio.proto`, `scheduler.proto`, `serial.proto`, `network.proto`
- **Robot Scheduler:** `robot/scheduler/src/main.rs`
- **Robot Serial:** `robot/serial/src/main.rs`
- **Robot Network:** `robot/network/src/main.rs`
- **Robot Vision:** `robot/vision/service.py`
- **Robot Audio:** `robot/audio/service.py`
- **TypeScript Generator:** `ts-schema/src/lib.rs`

---

## Contributing

When making changes:

1. Create a feature branch: `git checkout -b feature/my-feature`
2. Make changes with clear, atomic commits
3. Run `just fmt && just lint && just test`
4. Ensure CI passes: `just ci-<component>`
5. Update documentation if needed
6. Submit pull request with description of changes

**Before committing:**
- [ ] Code is formatted (`just fmt`)
- [ ] No linter errors (`just lint`)
- [ ] Tests pass (`just test`)
- [ ] No typos (`typos` command)
- [ ] Commit message follows conventional format
- [ ] Updated CLAUDE.md if architecture changed

---

## License

MIT License - See `LICENSE` file for details.

---

## Contact

For questions about this codebase, refer to:
- GitHub Issues: (repository issues page)
- Documentation: `docs/` directory
- This file: `CLAUDE.md`

---

*This CLAUDE.md was generated from actual source code analysis and is maintained alongside the codebase. Last updated: 2025-11-16*

---

## Recent Major Updates (Since 2025-11-07)

### ✅ Completed (Updated 2025-11-16)

1. **PostgreSQL Migration Layer** - Full repository implementation with dual-database support
2. **Robot/Lower Component** - STM32F407 + Embassy async runtime
3. **Robot/Upper Layer Architecture** ⭐ **NEW** - Distributed control system with Zenoh messaging
4. **Procedural Macros Crate** - Code generation infrastructure with comprehensive tests
5. **BLE Postcard Migration** - Migrated from custom protocol to Postcard serialization
6. **Internationalization** - 5-language support (EN, ZH-CN, ZH-TW, JA, FR)
7. **Firmware Testing** - Mock tests + QEMU simulation infrastructure
8. **Error Handling** - Migrated to thiserror for better error types
9. **defmt Support** - Embedded debugging for firmware and robot/lower
10. **Mobile Admin Panel** - Comprehensive CRUD interface for all entities
11. **Deployment Guide** - Complete production deployment documentation
12. **TypeScript Type Generation** ⭐ **NEW** - Automatic Rust→TS conversion with ts-rs
13. **Comprehensive Testing** ⭐ **NEW** - 1,158+ lines of tests across all components
14. **Structured Logging** ⭐ **NEW** - Migration from log to tracing

### 🎯 Latest Features (2025-11-16)

#### Robot Upper Layer (#80)
- **6 Protocol Buffer definitions** - Vision, Audio, Scheduler, Serial, Network, Common
- **Scheduler (Rust)** - Task coordination with Zenoh pub/sub
- **Serial (Rust)** - UART bridge to STM32 with Postcard serialization
- **Network (Rust)** - HTTP client for server pathfinding API
- **Vision Service (Python)** - YOLO, AprilTag, MediaPipe integration
- **Audio Service (Python)** - Wake word, STT, TTS capabilities
- **Complete distributed architecture** - All components communicate via Zenoh message bus

#### Automatic TypeScript Generation (#82)
- **ts-rs integration** - Compile-time TS generation from Rust types
- **21+ type definitions** - Entity, Area, Beacon, Merchant, Connection, etc.
- **Zero maintenance** - Types stay in sync automatically
- **Command:** `just gen-ts-schema`

#### Comprehensive Testing (#81)
- **Admin/Maintenance** - 219 lines of integration tests
- **Admin/Orchestrator** - 152 lines of firmware API tests
- **Admin/Plot** - 356 lines of plot client tests
- **Robot/Vision** - 284 lines of vision tests
- **Proc Macros** - 147 lines of macro tests
- **Total:** 1,158+ lines of new test code

#### Structured Logging (#78)
- **Server & Orchestrator** - Migrated from `log` to `tracing`
- **Better async support** - Proper tracing across async boundaries
- **Structured events** - Key-value logging instead of string formatting

### 📋 In Progress
- Robot motor control logic implementation
- Additional firmware test coverage (BLE, eFuse)
- PostgreSQL dual-write mode implementation
- Procedural macro real-world implementations
- Zenoh deployment configuration for production

### 📅 Recent Commits Summary (Last 10)
- `#80` - Feat: Add robot protocol buffer architecture and component skeletons
- `#81` - Test: Add comprehensive tests for all components
- `#82` - Feat: Implement automatic TypeScript type generation
- `#79` - Refactor: Move gesture_space logics to robot
- `#78` - Refactor: Migrate from log to tracing
- `#77` - Feat: Add placeholders for upper layer
- `#76` - Fix: Replace local Merchant struct with MerchantMobile
- `#75` - Docs: Update CLAUDE.md with recent project changes
- `#74` - Fix: Revert manual dark-mode CSS
- `#73` - Fix: Remove customized object ID

### 📊 Project Statistics (2025-11-16)
- **Lines of Code (Latest):** +6,380 / -450
- **Test Coverage:** 80%+ across all components
- **Components:** 20+ (server, firmware, mobile, robot, admin, shared)
- **Languages:** Rust, TypeScript, Python, Go, Swift
- **Protocol Buffers:** 11 files (admin + robot)
- **TypeScript Definitions:** 21+ auto-generated types
