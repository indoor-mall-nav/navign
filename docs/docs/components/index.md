# Components

The Navign project is a polyglot monorepo composed of multiple interconnected components, each built with specialized technologies to achieve indoor navigation, access control, and autonomous robot coordination.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     Mobile App                          │
│              (Vue 3 + Tauri 2 + TypeScript)            │
│     Navigation, Access Control, Admin Panel             │
└───────────┬──────────────────────────────┬──────────────┘
            │                              │
            │                              │
     ┌──────▼──────┐              ┌───────▼────────┐
     │   Beacon    │              │     Server     │
     │  (ESP32-C3) │◄─────────────┤  (Axum/Rust)   │
     │ BLE Firmware│              │  REST API      │
     └─────────────┘              └───────┬────────┘
                                          │
                           ┌──────────────┼──────────────┐
                           │              │              │
                     ┌─────▼─────┐  ┌────▼────┐  ┌──────▼──────┐
                     │   Admin   │  │  Robot  │  │  Database   │
                     │Orchestrator│  │ System  │  │  (MongoDB)  │
                     │   (gRPC)  │  │(Zenoh)  │  │(PostgreSQL) │
                     └─────┬─────┘  └─────────┘  └─────────────┘
                           │
                      ┌────▼────┐
                      │  Tower  │
                      │(Socket) │
                      └─────────┘
```

## Core Components

### [Server](./server/)
**Language:** Rust (Axum framework)
**Location:** `server/`

Centralized backend providing REST APIs for navigation, access control, and entity management.

**Key Features:**
- Multi-floor pathfinding with Dijkstra's algorithm
- MongoDB + PostgreSQL dual-database support
- OAuth2 authentication (GitHub, Google, WeChat)
- P-256 ECDSA cryptography for access control
- TOTP generation for beacons

**Subdocs:**
- [PostgreSQL Migration Guide](./server/postgres-migration.md)
- [Migration Summary](./server/postgres-migration-summary.md)

---

### [Mobile](./mobile/)
**Language:** TypeScript (Vue 3 + Tauri 2)
**Location:** `mobile/`

Cross-platform mobile and desktop application for users and administrators.

**Key Features:**
- Indoor navigation with MapLibre GL + Konva
- BLE beacon scanning and positioning
- Secure access control with biometric authentication
- Admin panel for entity/area/beacon management
- Offline support with SQLite caching

**Platforms:** macOS (tested), iOS, Android, Windows, Linux (planned)

**Subdocs:**
- [Admin Panel Guide](./mobile/admin-panel.md)
- [gRPC-Web Integration](./mobile/grpc-web-integration.md)

---

### [Beacon](./beacon)
**Language:** Rust (ESP-HAL)
**Location:** `firmware/`

ESP32-C3 firmware for BLE advertising, indoor positioning, and access control.

**Key Features:**
- BLE GATT server with custom protocol
- P-256 ECDSA signature verification
- Nonce-based challenge-response authentication
- Environmental sensing (DHT11)
- OTA firmware update support

**Hardware:**
- ESP32-C3 RISC-V microcontroller
- DHT11 temperature/humidity sensor
- GPIO relay/servo control
- PIR motion detection

---

### [Robot](./robot/)
**Language:** Rust (upper/lower) + Python (vision/audio/intelligence)
**Location:** `robot/`

Autonomous delivery robot system with distributed architecture.

**Architecture:**
- **[Upper Layer](./robot/upper/)** - Raspberry Pi running Scheduler, Serial, Network, Vision, Audio, Intelligence
- **[Lower Layer](./robot/lower)** - STM32F407 + Embassy async runtime for motor control
- **Messaging:** Zenoh pub/sub with Protocol Buffers

**Upper Components:**
- [Scheduler](./robot/upper/scheduler) - Task coordination
- [Serial](./robot/upper/serial) - UART bridge to STM32
- [Network](./robot/upper/navign) - HTTP client for server API
- [Vision](./robot/upper/vision) - YOLO, AprilTag, MediaPipe
- [Audio](./robot/upper/audio) - Wake word, STT, TTS
- [Bluetooth](./robot/upper/bluetooth) - BLE communication (planned)

---

### [Admin](./admin/)
**Language:** Rust (Orchestrator) + Go (Tower)
**Location:** `admin/`

Fleet management and coordination system for robots.

**Components:**
- **[Orchestrator](./admin/orchestrator)** - gRPC server for task assignment and robot selection
- **[Tower](./admin/tower)** - Socket.IO server for one-to-one robot communication
- **[Plot](./admin/client)** - Floor plan polygon extraction (Python)

**Features:**
- Robot registry and status tracking
- Task queue management with priority scheduling
- Firmware distribution and OTA updates
- gRPC streaming for real-time updates

**Subdocs:**
- [Quick Start Guide](./admin/quickstart.md)
- [Deployment Guide](./admin/deployment.md)
- [Protocol Documentation](./admin/protocol.md)
- [Implementation Guide](./admin/implementation-guide.md)
- [Vision Integration](./admin/vision.md)

---

### [Shared](./shared)
**Language:** Rust (no_std compatible)
**Location:** `shared/`

Cross-component library with feature flags for embedded and desktop targets.

**Key Features:**
- no_std compatibility for embedded systems
- Feature-gated exports (heapless, alloc, std, mongodb, sql, postgres)
- BLE message protocol (Postcard serialization)
- Cryptographic primitives (P-256 ECDSA)
- TypeScript type generation (ts-rs)

**Critical:** Never enable both `heapless` and `alloc` features simultaneously.

---

## Supporting Components

### Vision (Apple Vision Pro)
**Language:** Swift
**Location:** `vision/`

Apple Vision Pro application for gesture recognition and spatial understanding (planned).

---

### Miniapp (WeChat Mini Program)
**Language:** TypeScript
**Location:** `miniapp/`

WeChat Mini Program for user interaction and navigation assistance (planned).

---

## Component Communication

### Communication Patterns

**Mobile ↔ Server:**
- Protocol: REST API (HTTP/HTTPS)
- Format: JSON
- Authentication: JWT tokens

**Mobile ↔ Beacon:**
- Protocol: BLE GATT
- Format: Postcard binary serialization
- Security: P-256 ECDSA signatures

**Server ↔ Admin Orchestrator:**
- Protocol: gRPC (planned)
- Format: Protocol Buffers
- Use case: Server sync, firmware distribution

**Admin Orchestrator ↔ Tower:**
- Protocol: gRPC streaming
- Format: Protocol Buffers
- Use case: Task assignment, robot status

**Tower ↔ Robot:**
- Protocol: Socket.IO (WebSocket)
- Format: JSON
- Use case: Real-time task updates

**Robot Components (Upper Layer):**
- Protocol: Zenoh pub/sub
- Format: Protocol Buffers
- Use case: Inter-service messaging

**Robot Upper ↔ Lower:**
- Protocol: UART (serial)
- Format: Postcard binary serialization
- Use case: Motor commands, sensor data

---

## Technology Stack Summary

| Component | Primary Language | Framework/Runtime | Purpose |
|-----------|-----------------|-------------------|---------|
| Server | Rust | Axum + Tokio | REST API backend |
| Mobile | TypeScript | Vue 3 + Tauri 2 | Cross-platform app |
| Beacon | Rust | ESP-HAL | BLE firmware |
| Robot Upper | Rust + Python | Zenoh + Tokio | Robot coordination |
| Robot Lower | Rust | Embassy async | Motor control |
| Orchestrator | Rust | Tonic (gRPC) | Fleet management |
| Tower | Go | Socket.IO | Robot communication |
| Shared | Rust | no_std library | Cross-component types |

---

## Development Workflow

### Running All Components

```bash
# Terminal 1 - Server
cd server && cargo run

# Terminal 2 - Mobile
cd mobile && pnpm run tauri dev

# Terminal 3 - Orchestrator
cd admin/orchestrator && cargo run

# Terminal 4 - Tower
cd admin/tower && go run cmd/tower/main.go

# Terminal 5 - Robot (upper layer)
cd robot/scheduler && cargo run
```

### CI/CD Tasks

```bash
just ci-server
just ci-firmware
just ci-mobile
just ci-robot-upper
just ci-robot-lower
just ci-orchestrator
just ci-tower
```

---

## See Also

- [Pipelines](../pipelines/) - End-to-end data flow documentation
- [Testing](../testing/) - Testing strategies for each component
- [Development](../development/) - Development guides and best practices
- [CLAUDE.md](../../../CLAUDE.md) - Comprehensive development guide
