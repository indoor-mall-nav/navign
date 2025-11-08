# Beacon Firmware OTA Distribution System

This document describes the beacon firmware Over-The-Air (OTA) update distribution system implemented in Navign.

## Architecture

```
┌─────────────┐         ┌──────────────┐         ┌─────────────┐
│   Server    │◄────────┤ Orchestrator │◄────────┤   Tower     │
│  (Axum)     │  HTTP   │ (Axum+gRPC)  │  HTTP   │  (Go)       │
│             │         │              │         │             │
│ - Firmware  │         │ - Firmware   │         │ - Robot     │
│   Storage   │         │   Proxy API  │         │   Mgmt      │
│ - Upload    │         │ - Download   │         │ - Beacon    │
│ - Download  │         │   from       │         │   OTA       │
│ - Metadata  │         │   Server     │         │   (Future)  │
└─────────────┘         └──────────────┘         └─────────────┘
```

## Components

### 1. Server (Port 3000)

The main backend server stores and manages beacon firmware artifacts.

#### Firmware Storage

- **Location**: `./firmware_storage/` (configurable via `FIRMWARE_STORAGE_DIR`)
- **Naming**: `{device}-{version}-{timestamp}.bin`
- **Database**: MongoDB collection `firmwares`

#### REST API Endpoints

```
POST   /api/firmwares/upload              Upload new firmware (multipart)
GET    /api/firmwares                     List all firmwares (filterable)
GET    /api/firmwares/latest/:device      Get latest firmware metadata
GET    /api/firmwares/:id                 Get firmware metadata by ID
GET    /api/firmwares/:id/download        Download firmware binary
DELETE /api/firmwares/:id                 Delete firmware
```

#### Upload Firmware Example

```bash
curl -X POST http://localhost:3000/api/firmwares/upload \
  -F "version=1.0.0" \
  -F "device=esp32c3" \
  -F "description=Initial release" \
  -F "file=@beacon-firmware.bin" \
  -F "mark_latest=true"
```

#### Response

```json
{
  "id": "507f1f77bcf86cd799439011",
  "version": "1.0.0",
  "device": "esp32c3",
  "file_size": 524288,
  "checksum": "abc123...",
  "created_at": 1699564800000
}
```

#### List Firmwares Example

```bash
# Get all firmwares
curl http://localhost:3000/api/firmwares

# Filter by device
curl http://localhost:3000/api/firmwares?device=esp32c3

# Get latest only
curl http://localhost:3000/api/firmwares?latest_only=true
```

#### Download Firmware Example

```bash
curl -O http://localhost:3000/api/firmwares/507f1f77bcf86cd799439011/download
```

Response headers include:
- `X-Firmware-Version`: Semantic version
- `X-Firmware-Checksum`: SHA-256 checksum
- `X-Firmware-Device`: Target device type

### 2. Orchestrator (Ports 50051 + 8081)

The orchestrator acts as a firmware distribution proxy and task scheduler.

#### Services

1. **gRPC Server** (Port 50051): Robot task management
2. **HTTP Server** (Port 8081): Firmware distribution API

#### Configuration

Environment variables:
```bash
ORCHESTRATOR_GRPC_ADDR=[::1]:50051      # gRPC server address
ORCHESTRATOR_HTTP_ADDR=0.0.0.0:8081     # HTTP server address
SERVER_URL=http://localhost:3000        # Backend server URL
```

#### Firmware API Endpoints

```
GET /health                     Orchestrator health check
GET /firmwares                  List firmwares (proxied from server)
GET /firmwares/latest/:device   Get latest firmware metadata
GET /firmwares/:id              Get firmware metadata
GET /firmwares/:id/download     Download firmware binary
```

#### Example Usage

```bash
# Get latest firmware for ESP32-C3
curl http://localhost:8081/firmwares/latest/esp32c3

# Download firmware
curl -O http://localhost:8081/firmwares/507f1f77bcf86cd799439011/download
```

### 3. Tower (Port 8080)

The Tower component (Go) connects robots/beacons via Socket.IO for real-time communication.

**Note**: Beacon OTA functionality via Tower is not yet implemented. Current implementation provides the infrastructure for future OTA updates.

## Firmware Schema

### Firmware Document (MongoDB)

```typescript
{
  _id: ObjectId,
  version: string,              // Semantic version (e.g., "1.0.0")
  device: FirmwareDevice,       // Target device enum
  description?: string,         // Short description
  file_path: string,            // Storage filename
  file_size: number,            // Bytes
  checksum: string,             // SHA-256 hex string
  is_latest: boolean,           // Only one per device
  git_commit?: string,          // Git commit hash
  build_time: number,           // Milliseconds since epoch
  created_at: number,           // Upload timestamp
  release_notes?: string        // Markdown release notes
}
```

### FirmwareDevice Enum

```rust
pub enum FirmwareDevice {
    Esp32,      // "esp32"
    Esp32C3,    // "esp32c3"
    Esp32C5,    // "esp32c5"
    Esp32C6,    // "esp32c6"
    Esp32S3,    // "esp32s3"
}
```

## Security

### Checksum Verification

All firmware files have SHA-256 checksums calculated on upload and stored in the database. Clients should verify checksums before flashing.

```rust
use sha2::{Sha256, Digest};

let mut hasher = Sha256::new();
hasher.update(&firmware_data);
let calculated = format!("{:x}", hasher.finalize());
assert_eq!(calculated, firmware.checksum);
```

### Storage Permissions

The firmware storage directory should have restricted permissions:

```bash
chmod 750 firmware_storage
chown navign:navign firmware_storage
```

### Rate Limiting

Server endpoints are protected by the global rate limiter (configurable via `RATE_LIMIT_PER_SECOND` and `RATE_LIMIT_BURST_SIZE`).

## Deployment

### Server Setup

```bash
# Set storage location (optional)
export FIRMWARE_STORAGE_DIR=/var/navign/firmwares

# Start server
cd server
cargo run --release
```

### Orchestrator Setup

```bash
# Configure endpoints
export ORCHESTRATOR_GRPC_ADDR=[::1]:50051
export ORCHESTRATOR_HTTP_ADDR=0.0.0.0:8081
export SERVER_URL=http://localhost:3000

# Install protobuf compiler
apt-get install protobuf-compiler

# Start orchestrator
cd admin/orchestrator
cargo run --release
```

### Docker Compose Example

```yaml
version: '3.8'
services:
  mongodb:
    image: mongo:8.0
    ports:
      - "27017:27017"
    volumes:
      - mongo-data:/data/db

  server:
    build: ./server
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=mongodb://mongodb:27017
      - FIRMWARE_STORAGE_DIR=/app/firmware_storage
    volumes:
      - firmware-data:/app/firmware_storage
    depends_on:
      - mongodb

  orchestrator:
    build: ./admin/orchestrator
    ports:
      - "50051:50051"
      - "8081:8081"
    environment:
      - SERVER_URL=http://server:3000
      - ORCHESTRATOR_GRPC_ADDR=[::]:50051
      - ORCHESTRATOR_HTTP_ADDR=0.0.0.0:8081

volumes:
  mongo-data:
  firmware-data:
```

## Future Enhancements

### Phase 1: Current Implementation ✅
- [x] Server firmware storage and REST API
- [x] Orchestrator firmware proxy API
- [x] SHA-256 checksum verification
- [x] Multiple device support
- [x] Version management

### Phase 2: Tower Integration (Planned)
- [ ] Socket.IO events for firmware updates
- [ ] Beacon firmware version reporting
- [ ] OTA update task creation
- [ ] Progress tracking

### Phase 3: Beacon OTA (Planned)
- [ ] ESP-IDF OTA partition support
- [ ] HTTP-based firmware download
- [ ] Rollback mechanism
- [ ] Signature verification

### Phase 4: Advanced Features (Planned)
- [ ] Staged rollouts (percentage-based)
- [ ] A/B testing
- [ ] Automatic version checking
- [ ] Firmware update scheduling
- [ ] Delta updates (binary diff)

## Troubleshooting

### Server Issues

**Problem**: Firmware upload fails with "Failed to create storage directory"

**Solution**: Ensure the server process has write permissions:
```bash
mkdir -p firmware_storage
chmod 750 firmware_storage
```

**Problem**: Download returns 404

**Solution**: Check that the firmware file exists on disk and the database record is correct.

### Orchestrator Issues

**Problem**: "protoc not found" error

**Solution**: Install protobuf compiler:
```bash
apt-get install protobuf-compiler
```

**Problem**: Cannot fetch firmware from server

**Solution**: Check `SERVER_URL` environment variable and ensure server is running:
```bash
curl http://localhost:3000/health
```

## Testing

### Manual Testing

```bash
# 1. Start MongoDB
docker run -d -p 27017:27017 mongo:8.0

# 2. Start server
cd server
cargo run

# 3. Upload firmware
curl -X POST http://localhost:3000/api/firmwares/upload \
  -F "version=1.0.0" \
  -F "device=esp32c3" \
  -F "file=@test-firmware.bin" \
  -F "mark_latest=true"

# 4. Start orchestrator
cd admin/orchestrator
cargo run

# 5. Test orchestrator API
curl http://localhost:8081/health
curl http://localhost:8081/firmwares/latest/esp32c3
```

### Integration Tests

```bash
# Server tests
cd server
cargo test

# Orchestrator tests
cd admin/orchestrator
cargo test
```

## References

- [ESP-IDF OTA Documentation](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/ota.html)
- [Axum Documentation](https://docs.rs/axum/)
- [MongoDB Rust Driver](https://docs.rs/mongodb/)
- [Tonic gRPC](https://docs.rs/tonic/)

## Support

For issues or questions:
- GitHub Issues: [navign/issues](https://github.com/your-org/navign/issues)
- Documentation: See `/docs` directory
- CLAUDE.md: Architecture overview
