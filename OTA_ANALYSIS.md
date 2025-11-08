# Navign Beacon OTA Functionality Analysis

## Executive Summary

The Navign codebase **does not currently have any OTA (Over-The-Air) firmware update functionality** for beacons. OTA updates are explicitly listed as a future roadmap item in the beacon documentation. Below is a comprehensive analysis of the current architecture and what would need to be implemented.

## Current Architecture

### 1. Admin Orchestrator (Rust/gRPC)
- **Location**: `admin/orchestrator/src/main.rs`
- **Responsibility**: Task scheduling and robot management
- **gRPC Services**:
  - `ReportRobotStatus()`: Receives robot status from Tower
  - `GetTaskAssignment()`: Streams task assignments to Tower
- **Limitation**: No firmware/artifact handling or distribution mechanism
- **Storage**: In-memory `HashMap<String, RobotInfo>` - no persistent artifact storage

### 2. Admin Tower (Go/Socket.IO)
- **Location**: `admin/tower/internal/socket_server/server.go`
- **Responsibility**: WebSocket bridge between Orchestrator and robots
- **Socket.IO Events Currently Supported**:
  - `connect` / `disconnect`: Connection lifecycle
  - `register`: Robot registration
  - `task_assigned`: Task distribution from orchestrator
  - `task_update`: Task progress updates
  - `status_update`: Robot status reporting
  - `keep_alive` / `ping` / `pong`: Connection heartbeat
- **Limitation**: No firmware distribution events or binary transfer mechanism

### 3. Beacon Firmware (Rust/ESP32-C3)
- **Location**: `beacon/src/bin/main.rs`
- **BLE Protocol**: Custom binary protocol with message types defined in `shared/src/ble/message.rs`
- **BLE Message Types** (Currently Supported):
  - `DeviceRequest` (0x01): Request device info
  - `DeviceResponse` (0x02): Device type, capabilities, 24-byte ID
  - `NonceRequest` (0x03): Request authentication nonce
  - `NonceResponse` (0x04): Nonce + signature verification
  - `UnlockRequest` (0x05): Signed proof for access control
  - `UnlockResponse` (0x06): Unlock success/failure
- **Storage**:
  - Efuse BLOCK_KEY0: 32-byte private key (write-once, read-protected)
  - No firmware update storage mechanism
  - No OTA capability field in device capabilities enum
- **No OTA dependencies**: `beacon/Cargo.toml` has no OTA-related crates

### 4. Shared Library (Rust)
- **Location**: `shared/src/ble/device_caps.rs`
- **Device Capabilities Enum**:
  ```rust
  pub enum DeviceCapability {
      UnlockGate,          // Physical unlock capability
      EnvironmentalData,   // Temperature/humidity sensor
      RssiCalibration,     // Position calibration
      // NO OTA capability defined
  }
  ```
- **No artifact or firmware version tracking**

### 5. Server (Rust/Axum)
- **Location**: `server/src/main.rs`
- **API Endpoints**: None for firmware/artifact handling
- **Database**: MongoDB with collections for entities, beacons, merchants, areas, connections
- **No schemas** for firmware artifacts, versions, or update history
- **No file storage** for binary artifacts

## What Exists

### Current File/Artifact Handling
- **None**: There is no existing infrastructure for storing, serving, or managing firmware binaries

### Current Firmware Version Management
- **None**: Beacons have no version tracking or update capability fields

### Current OTA Mechanism
- **None**: No WiFi-based OTA, no BLE firmware transfer, no firmware manifest system

## What Would Need to Be Implemented

### Phase 1: Infrastructure & Schemas

#### 1.1 Server Side (Rust/Axum)
**New Schemas** (`server/src/schema/`):
```rust
// Firmware artifact metadata
pub struct FirmwareArtifact {
    pub id: String,              // MongoDB ObjectId
    pub version: String,         // semver format: "1.2.3"
    pub target_chip: String,     // "esp32c3", "esp32s3", etc.
    pub release_date: DateTime,
    pub file_size: u64,          // bytes
    pub checksum_sha256: String, // hex-encoded SHA-256
    pub changelog: String,
    pub file_path: String,       // S3/local storage path
    pub is_prerelease: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

pub struct BeaconFirmwareState {
    pub beacon_id: String,       // MongoDB ObjectId
    pub current_version: String,
    pub last_check_time: DateTime,
    pub update_status: UpdateStatus, // Idle, Downloading, Installing, Complete, Failed
    pub error_message: Option<String>,
}

pub enum UpdateStatus {
    Idle,
    CheckingForUpdates,
    DownloadingFirmware,
    InstallingFirmware,
    UpdateComplete,
    UpdateFailed(String),
    RollingBack,
}
```

**New API Endpoints**:
```rust
// Artifact management (admin only)
POST   /api/admin/firmware/artifacts          // Upload firmware binary
GET    /api/admin/firmware/artifacts          // List artifacts
GET    /api/admin/firmware/artifacts/{id}     // Get artifact details
DELETE /api/admin/firmware/artifacts/{id}     // Delete artifact

// Beacon firmware state (admin/beacons)
GET    /api/entities/{eid}/beacons/{bid}/firmware/state     // Get beacon FW state
POST   /api/entities/{eid}/beacons/{bid}/firmware/check     // Check for updates
POST   /api/entities/{eid}/beacons/{bid}/firmware/update    // Trigger update
GET    /api/entities/{eid}/beacons/{bid}/firmware/progress  // Get update progress

// Public endpoints (for beacons)
GET    /api/firmware/latest                   // Get latest firmware manifest
GET    /api/firmware/{version}/download       // Download firmware binary
POST   /api/firmware/{version}/report         // Report update status
```

**Database Collections**:
```
- firmware_artifacts: Metadata for each firmware release
- beacon_firmware_state: Current update status per beacon
- firmware_update_log: Historical update records (audit trail)
```

#### 1.2 Shared Library (Rust)
**New BLE Message Types**:
```rust
// In shared/src/ble/message.rs
pub enum BleMessage {
    // ... existing variants ...
    FirmwareCheckRequest,                     // 0x10
    FirmwareCheckResponse(FirmwareCheckInfo), // 0x11
    FirmwareDownloadRequest {                 // 0x12
        version: [u8; 4],      // semver packed as 4 bytes
        chunk_index: u16,      // for chunked download
    },
    FirmwareDownloadResponse {                // 0x13
        version: [u8; 4],
        chunk_index: u16,
        chunk_data: [u8; 100], // 100-byte chunks over BLE
        total_chunks: u16,
        checksum: u32,         // CRC32 for this chunk
    },
    FirmwareInstallRequest {                  // 0x14
        version: [u8; 4],
        total_size: u32,
        checksum_sha256: [u8; 32],
    },
    FirmwareInstallResponse {                 // 0x15
        success: bool,
        error_code: u8,        // 0=OK, 1=Checksum failed, 2=Size mismatch, etc.
    },
    FirmwareStatusReport {                    // 0x16
        current_version: [u8; 4],
        update_status: u8,     // 0=Idle, 1=Downloading, 2=Installing, 3=Complete, 4=Failed
        progress_percent: u8,  // 0-100
        error_code: Option<u8>,
    },
}

pub struct FirmwareCheckInfo {
    pub latest_version: [u8; 4],
    pub current_version: [u8; 4],
    pub file_size: u32,
    pub checksum_sha256: [u8; 32],
}
```

**New Device Capability**:
```rust
pub enum DeviceCapability {
    UnlockGate,
    EnvironmentalData,
    RssiCalibration,
    OTAFirmwareUpdate,  // NEW: Supports firmware updates
}
```

#### 1.3 Admin Orchestrator (Rust)
**New gRPC Messages** (`admin/orchestrator/proto/task.proto`):
```protobuf
message FirmwareManifest {
  string version = 1;
  int64 release_timestamp = 2;
  string target_chip = 3;
  string download_url = 4;
  string checksum_sha256 = 5;
  int64 file_size = 6;
}

message BeaconFirmwareRequest {
  string beacon_id = 1;
  string entity_id = 2;
}

message BeaconFirmwareState {
  string beacon_id = 1;
  string current_version = 2;
  string update_status = 3;  // IDLE, DOWNLOADING, INSTALLING, COMPLETE, FAILED
  int32 progress_percent = 4;
  string error_message = 5;
}

message FirmwareUpdateRequest {
  string beacon_id = 1;
  string entity_id = 2;
  string target_version = 3;
}

service OrchestratorService {
  // ... existing RPCs ...
  rpc GetFirmwareManifest(FirmwareManifest) returns (FirmwareManifest);
  rpc UpdateBeaconFirmware(FirmwareUpdateRequest) returns (FirmwareUpdateRequest);
  rpc ReportBeaconFirmwareStatus(BeaconFirmwareState) returns (google.protobuf.Empty);
}
```

**New Orchestrator Methods**:
```rust
pub struct OrchestratorServiceImpl {
    // ... existing fields ...
    firmware_manager: FirmwareManager,
}

impl OrchestratorServiceImpl {
    async fn get_firmware_manifest(&self, version: &str) -> Result<FirmwareManifest>;
    async fn queue_firmware_update(&self, beacon_id: &str, version: &str);
    async fn track_firmware_update_status(&self, beacon_id: &str, status: UpdateStatus);
    async fn rollback_firmware(&self, beacon_id: &str) -> Result<()>;
}
```

#### 1.4 Admin Tower (Go)
**New Socket.IO Events**:
```go
const (
    // ... existing events ...
    EventFirmwareCheckRequest  = "firmware_check_request"
    EventFirmwareCheckResponse = "firmware_check_response"
    EventFirmwareDownloadStart = "firmware_download_start"
    EventFirmwareChunkReceived = "firmware_chunk_received"
    EventFirmwareInstallStart  = "firmware_install_start"
    EventFirmwareInstallStatus = "firmware_install_status"
    EventFirmwareStatusReport  = "firmware_status_report"
)

type FirmwareCheckRequest struct {
    RobotID string `json:"robot_id"`
}

type FirmwareCheckResponse struct {
    LatestVersion string `json:"latest_version"`
    CurrentVersion string `json:"current_version"`
    UpdateAvailable bool `json:"update_available"`
    FileSize int64 `json:"file_size"`
}

type FirmwareChunkRequest struct {
    RobotID string `json:"robot_id"`
    Version string `json:"version"`
    ChunkIndex int `json:"chunk_index"`
}

type FirmwareChunkData struct {
    Version string `json:"version"`
    ChunkIndex int `json:"chunk_index"`
    Data []byte `json:"data"`
    TotalChunks int `json:"total_chunks"`
    CRC32 uint32 `json:"crc32"`
}
```

**New Tower Methods**:
```go
func (s *Server) handleFirmwareCheck(conn socketio.Conn, packet *models.FirmwareCheckRequest)
func (s *Server) sendFirmwareChunk(robotID string, chunk *models.FirmwareChunkData)
func (s *Server) monitorFirmwareUpdate(robotID string, version string)
func (s *Server) handleFirmwareStatus(packet *models.FirmwareStatusReport)
```

### Phase 2: Beacon Firmware (Rust/ESP32-C3)

#### 2.1 Firmware Storage & Updates
**New beacon modules**:
```
beacon/src/bin/
├── firmware/
│   ├── mod.rs             // Firmware update manager
│   ├── downloader.rs      // BLE-based chunked download
│   ├── installer.rs       // Flash partition management + rollback
│   ├── version.rs         // Version tracking
│   └── storage.rs         // Non-volatile state storage
```

**New storage requirements**:
- Current firmware version (stored in NVS - Non-Volatile Storage)
- Previous firmware version (for rollback)
- Update state (downloading, installing, complete, error)
- Downloaded firmware chunks (temporary, in PSRAM if available)

**Flash partition layout**:
```
┌─────────────────────────┐
│    Bootloader           │ (0x0000 - 0x7FFF)
├─────────────────────────┤
│    Partition Table      │ (0x8000 - 0x8FFF)
├─────────────────────────┤
│    OTA App 0 (Active)   │ (0x10000 - 0xAFFFF) ~640KB
├─────────────────────────┤
│    OTA App 1 (Update)   │ (0xB0000 - 0x14FFFF) ~640KB
├─────────────────────────┤
│    NVS (Settings)       │ (0x150000 - 0x15FFFF) ~64KB
├─────────────────────────┤
│    SPIFFS (FS)          │ (0x160000 - 0x1FFFFF) ~640KB
└─────────────────────────┘
```

#### 2.2 Beacon BLE Protocol Additions
**Update check flow**:
```
Beacon (FW v1.0.0)              Server
    |                              |
    |---(1) FirmwareCheckRequest-->|
    |       (beacon_id, chip)      |
    |                              |
    |<--(2) FirmwareCheckResponse--|
    |       (latest: v1.1.0,       |
    |        size: 256KB,          |
    |        checksum_sha256)      |
    |                              |
    | [Compare versions]           |
    | [If update needed, proceed]  |
    |                              |
    |---(3) FirmwareDownloadRequest|
    |       (version, chunk 0)     |
    |<--(4) FirmwareDownloadResponse
    |       (100 bytes chunk data) |
    |                              |
    |---(5) repeat for each chunk  |
    |                              |
    |---(6) FirmwareInstallRequest |
    |       (checksum_sha256,      |
    |        total_size)           |
    |<--(7) FirmwareInstallResponse
    |       (success)              |
    |                              |
    | [Reboot into new FW]         |
```

**Update failure handling**:
- **Checksum mismatch**: Rollback to previous version
- **Timeout**: Pause download, resume on next check
- **Installation failure**: Rollback automatically
- **Partial download**: Resume from last complete chunk

#### 2.3 Bootloader Modifications
**Optional**: ESP-IDF OTA support
- Use ESP-IDF's built-in OTA system (already available)
- Requires `esp_ota_ops` library integration
- Manages partition switching and validation

### Phase 3: Mobile App Integration

#### 3.1 Tauri Backend Commands
```rust
// mobile/src-tauri/src/lib.rs

#[tauri::command]
async fn check_beacon_firmware_update(beacon_id: String) -> Result<FirmwareStatus>;

#[tauri::command]
async fn trigger_beacon_firmware_update(beacon_id: String, version: String) -> Result<()>;

#[tauri::command]
async fn get_firmware_update_progress(beacon_id: String) -> Result<UpdateProgress>;

#[tauri::command]
async fn cancel_firmware_update(beacon_id: String) -> Result<()>;
```

#### 3.2 Vue Frontend
**New UI Views**:
- Beacon firmware status panel
- Manual firmware update trigger
- Update progress indicator
- Rollback option

## Dependencies to Add

### Beacon (Cargo.toml)
```toml
[dependencies]
esp-idf-svc = { version = "0.49", features = ["ota", "nvs"] }
esp-idf-hal = "0.44"  # For OTA partition management
crc = "3.0"            # CRC32 for chunk validation
sha2 = "0.10"          # Already present, for SHA-256
```

### Server (Cargo.toml)
```toml
[dependencies]
tokio-util = { version = "0.7", features = ["io"] }  # Streaming utilities
sha2 = "0.10"          # SHA-256 hashing
hex = "0.4"            # Hex encoding for checksums
uuid = { version = "1.0", features = ["v4"] }
async-stream = "0.3"   # Async streaming
```

### Go Tower (go.mod)
```go
require (
    github.com/google/uuid v1.3.0
)
```

## Security Considerations

### 1. **Firmware Signing**
- All firmware artifacts must be **digitally signed** with server's private key
- Beacons verify signature before installation
- Prevents unauthorized/malicious firmware injection

### 2. **Checksum Validation**
- SHA-256 checksum verification before and after download
- Chunk-level CRC32 for corrupted transfer detection
- Double validation on beacon side

### 3. **Rollback Protection**
- Keep previous firmware version available for 7 days
- Automatic rollback if new version fails to boot
- Prevents bricking devices

### 4. **Rate Limiting**
- Prevent rapid repeated update attempts
- Stagger updates across beacon fleet to avoid network congestion
- Max 1 update per beacon per hour (configurable)

### 5. **Authentication**
- Firmware updates only from authenticated beacons
- Use existing ECDSA challenge-response system
- Verify beacon's server certificate

## Implementation Roadmap

**Phase 1 (Weeks 1-2): Foundation**
- [ ] Add firmware artifact schemas to server
- [ ] Add storage (MongoDB or S3) for firmware binaries
- [ ] Implement firmware API endpoints
- [ ] Add OTA capability to beacon capabilities enum
- [ ] Design proto file for firmware messages

**Phase 2 (Weeks 3-4): Backend Infrastructure**
- [ ] Implement Orchestrator firmware manager
- [ ] Add firmware tracking to Robot registry
- [ ] Implement Tower Socket.IO firmware events
- [ ] Set up firmware signing pipeline
- [ ] Create firmware repository (S3/local storage)

**Phase 3 (Weeks 5-6): Beacon Firmware**
- [ ] Add OTA BLE message types to shared library
- [ ] Implement chunked download in beacon
- [ ] Add NVS storage for firmware state
- [ ] Implement flash partition switching
- [ ] Add rollback mechanism

**Phase 4 (Weeks 7-8): Integration & Testing**
- [ ] Integrate firmware updates into Tower
- [ ] Test end-to-end update flow
- [ ] Add firmware upload UI to admin panel
- [ ] Create firmware distribution policy
- [ ] Load testing with many beacons

**Phase 5 (Weeks 9-10): Mobile & Deployment**
- [ ] Add Tauri commands for firmware management
- [ ] Create Vue UI components
- [ ] Document update procedures
- [ ] Create rollback runbook
- [ ] Deploy to production

## Estimated Effort

- **Server/API**: 40-50 hours
- **Orchestrator**: 30-40 hours
- **Beacon Firmware**: 50-60 hours (complexity of embedded OTA)
- **Tower (Go)**: 20-30 hours
- **Mobile (Tauri/Vue)**: 20-30 hours
- **Testing & Documentation**: 40-50 hours
- **Total**: ~200-260 hours (~1.5 person-months)

## Alternative Approaches

### 1. **HTTP-based OTA** (Simpler but requires WiFi)
- Beacon downloads directly from server via WiFi/HTTP
- Uses ESP-IDF native OTA system
- Pros: Simple, proven, built-in rollback
- Cons: Requires WiFi, battery drain, larger binary
- **Recommendation**: Better for initial implementation

### 2. **BLE-based OTA** (Current design)
- Beacon downloads via BLE from Tower/mobile
- More complex, slower
- Pros: Works without WiFi, can use existing BLE connection
- Cons: Slower, requires more complex protocol
- **Recommendation**: Secondary implementation after HTTP works

### 3. **USB Update** (Simplest for testing)
- Connect beacon to laptop via USB
- Use `espflash` to upload new firmware
- Pros: Simple, no network needed
- Cons: Not scalable for fleet updates
- **Recommendation**: For development/testing only

## Conclusion

The Navign system is currently **production-ready for access control and navigation**, but **lacks OTA firmware update capability**. This is documented as a planned feature in the beacon roadmap. The architecture would support OTA through the existing admin orchestration layer (gRPC + Socket.IO), leveraging the established robot management infrastructure.

The most practical approach would be HTTP-based OTA using ESP-IDF's built-in system, with BLE-based chunking as a secondary option. Complete implementation would take approximately 1.5 person-months for a robust, production-ready system.
