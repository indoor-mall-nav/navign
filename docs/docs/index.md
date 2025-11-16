# Navign Project

An indoor navigation and access control system designed for large buildings such as malls, airports, hospitals, and schools. Navign combines BLE beacon-based positioning, real-time pathfinding, secure access control, and autonomous robot delivery coordination into a comprehensive indoor navigation platform.

## Overview

**License:** MIT
**Version:** 0.1.0
**Primary Language:** Rust (with TypeScript, Go, Python, Swift)

### Key Features

- **Indoor Navigation** - Turn-by-turn navigation with multi-floor support (elevators, escalators, stairs)
- **BLE Positioning** - Real-time location tracking using ESP32-C3 BLE beacons
- **Access Control** - Contactless door/gate unlocking with P-256 ECDSA cryptography
- **Robot Fleet Management** - Autonomous delivery robot coordination and task assignment
- **Cross-Platform Mobile** - Vue 3 + Tauri 2 app for iOS, Android, macOS, Windows, Linux
- **Environmental Monitoring** - Temperature and humidity sensing via DHT11 sensors
- **Gesture Recognition** - Spatial understanding and hand tracking for accessibility

## Architecture

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

## Quick Links

### Components
- **[Server](./components/server/)** - Axum REST API server with MongoDB/PostgreSQL
- **[Mobile](./components/mobile/)** - Cross-platform Vue 3 + Tauri mobile app
- **[Beacon](./components/beacon)** - ESP32-C3 BLE firmware for positioning and access control
- **[Robot](./components/robot/)** - Autonomous delivery robot system (upper + lower layers)
- **[Admin](./components/admin/)** - Fleet management and orchestration (Orchestrator + Tower)
- **[Shared](./components/shared)** - no_std Rust library shared across components

### Pipelines
- **[Navigation](./pipelines/navigation)** - Indoor pathfinding and turn-by-turn directions
- **[Localization](./pipelines/localization)** - BLE beacon triangulation for positioning
- **[Access Control](./pipelines/unlock)** - Cryptographic door/gate unlocking
- **[Robot Control](./pipelines/robot-control)** - Autonomous delivery task management
- **[OTA Updates](./pipelines/ota)** - Remote firmware updates for robots
- **[Firmware OTA](./pipelines/firmware-ota)** - Remote updates for ESP32-C3 beacons

### Development
- **[Testing](./testing/)** - Testing strategies and guides (unit, integration, simulation)
- **[Development](./development/)** - Development workflow, coding standards, best practices
- **[Critical TODOs](./development/critical-todos)** - High-priority tasks and known issues
- **[Refactoring Plan](./development/refactoring-plan)** - Long-term architectural improvements

## Technology Stack

### Backend (Rust)
- **Server:** Axum 0.8.6, Tokio 1.47.1, MongoDB 3.3.0, PostgreSQL (SQLx 0.8.6)
- **Cryptography:** p256 0.13.2, bcrypt 0.17.1, jsonwebtoken 10.0.0
- **Pathfinding:** Dijkstra with bump allocation (bumpalo 3.18)

### Embedded (Rust)
- **Firmware:** ESP-HAL 1.0.0-rc.1, bleps (BLE stack), p256 0.13.2
- **Hardware:** ESP32-C3 microcontroller (RISC-V, WiFi + BLE)

### Frontend (TypeScript/Vue)
- **Mobile:** Vue 3.5.18, Tauri 2.8.1, Pinia 3.0.3, MapLibre GL 5.6.2
- **UI:** Reka UI 2.4.1, Tailwind CSS 4.1.12, Konva 9.3.22

### Robot (Rust + Python)
- **Upper Layer:** Scheduler (Rust), Serial (Rust), Network (Rust), Vision (Python), Audio (Python), Intelligence (Python)
- **Lower Layer:** STM32F407 + Embassy async runtime
- **Messaging:** Zenoh pub/sub, Protocol Buffers

### Admin (Rust + Go)
- **Orchestrator:** Tonic 0.12 (gRPC server)
- **Tower:** Socket.IO 1.7.0 (Go WebSocket server)

## Getting Started

### Prerequisites
- Rust 1.83+ (nightly for some features)
- Node.js 23.11.0+ with pnpm 10.15.0+
- Python 3.13+ with uv package manager
- MongoDB 8.0+
- Just command runner

### Installation

```bash
# Clone repository
git clone <repository-url>
cd navign

# Run initialization (installs all tools and dependencies)
just init

# Format code
just fmt

# Run linters
just lint

# Run tests
just test
```

### Running Components

```bash
# Server
cd server && cargo run

# Mobile app
cd mobile && pnpm run tauri dev

# Firmware (requires esp-idf toolchain)
cd firmware && cargo build --release

# Admin orchestrator
cd admin/orchestrator && cargo run

# Admin tower
cd admin/tower && go run cmd/tower/main.go
```

## Documentation Structure

- **[Components](./components/)** - Individual component documentation
- **[Pipelines](./pipelines/)** - End-to-end data flow documentation
- **[Testing](./testing/)** - Testing strategies and guides
- **[Development](./development/)** - Development workflow and best practices

## License

MIT License - See LICENSE file for details.

---

*For comprehensive development guidance, see [CLAUDE.md](../../CLAUDE.md) in the repository root.*
