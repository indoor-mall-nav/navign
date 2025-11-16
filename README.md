# Navign

> An elegant and comprehensive indoor mall navigation system with BLE beacon-based positioning, secure access control,
> gesture recognition, and robotics delivery.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.1.0-green.svg)

## 🌟 Overview

**Navign** is a complete indoor navigation and automation platform designed for shopping malls, transportation hubs,
schools, and hospitals. The system combines cutting-edge technologies including Bluetooth Low Energy (BLE) beacons,
cryptographic security, computer vision, gesture recognition, and robotics to provide intelligent wayfinding, secure
access control, and automated delivery services.

## 📦 Project Structure

This monorepo contains multiple interconnected components:

```
navign/
├── server/              # Rust backend server with pathfinding & API
├── mobile/              # Vue + Tauri cross-platform mobile app
├── firmware/            # ESP32-C3 BLE beacon firmware (Rust embedded)
├── miniapp/             # WeChat Mini Program
├── robot/               # Robot upper layer (Rust + Python distributed system)
│   ├── scheduler/       # Task coordination (Rust)
│   ├── serial/          # UART bridge to STM32 (Rust)
│   ├── network/         # Server communication (Rust)
│   ├── vision/          # Computer vision (Python - YOLO, AprilTag, MediaPipe)
│   ├── audio/           # Wake word & TTS (Python)
│   ├── intelligence/    # AI/LLM scene description (Python)
│   ├── lower/           # STM32F407 motor control (Rust Embassy)
│   └── firmware/        # Raspberry Pi firmware (Rust)
├── animations/          # Manim animations for presentations (Python)
├── vision/              # Apple Vision Pro spatial computing app (Swift)
├── admin/
│   ├── maintenance/     # ESP32-C3 key management CLI (Rust)
│   ├── orchestrator/    # Robot task orchestration (Rust gRPC)
│   ├── tower/           # Robot WebSocket server (Go)
│   └── plot/            # Floor plan polygon extraction (Python)
├── ts-schema/           # TypeScript schema generator (ts-rs)
├── shared/              # Shared Rust types (no_std compatible)
│   └── pathfinding/     # A* and triangulation-based routing
└── schematics/          # KiCad PCB designs for hardware
```

## 🏗️ System Architecture

### Core Components

#### 🖥️ **Server** (`server/`)

High-performance Rust backend providing:

- **Advanced Pathfinding**: Dijkstra + A* with triangulation for non-Manhattan polygons
- **Multi-floor Navigation**: Support for elevators, escalators, and stairs
- **Beacon Authentication**: TOTP-based secure access control with P-256 ECDSA signatures
- **RESTful API**: Full CRUD operations for entities, areas, merchants, beacons
- **OAuth2 Integration**: GitHub, Google, WeChat authentication
- **Dual Database**: MongoDB (primary) + PostgreSQL (optional migration layer)

**Tech Stack**: Axum, Tokio, MongoDB, JWT, P-256 ECDSA, TOTP, Bump allocation

#### 📱 **Mobile App** (`mobile/`)

Cross-platform indoor navigation app built with Vue.js and Tauri:

- **Indoor Positioning**: Real-time BLE beacon triangulation with RSSI-based distance calculation
- **Turn-by-Turn Navigation**: Visual route overlay on interactive maps
- **Secure Door Unlocking**: Cryptographically signed BLE communication (P256 ECDSA, AES-GCM)
- **Biometric Authentication**: Touch ID, Face ID, fingerprint support
- **Interactive Maps**: MapLibre GL + Konva canvas for polygon-based areas
- **Multi-platform**: iOS, Android, macOS, Windows, Linux

**Tech Stack**: Vue 3, TypeScript, Tauri 2.0, Reka UI, Tailwind CSS 4, Pinia, MapLibre GL, Konva, SQLite

#### 📡 **Firmware** (`firmware/`)

Secure BLE beacon firmware for ESP32-C3 microcontrollers:

- **BLE Advertising**: Multi-service broadcasting (Authorization, Location, Navigation, Environmental)
- **Cryptographic Security**: P-256 ECDSA signatures, nonce-based challenge-response
- **Device Types**: Merchant, Pathway, Connection, Turnstile
- **Hardware Control**: Relay, servo motor, IR transmitter, DHT11 sensor
- **Replay Attack Prevention**: Time-windowed nonce validation with rate limiting
- **Efuse Key Storage**: Hardware-secured private keys in ESP32-C3 efuse BLOCK_KEY0

**Tech Stack**: Rust (embedded), ESP32-C3, BLE (bleps), P-256 ECDSA, DHT11

#### 🤖 **Robot Upper Layer** (`robot/`)

Distributed control system for autonomous delivery robots:

- **Vision Service**: YOLOv12 object detection, AprilTag pose estimation, MediaPipe hand tracking
- **Audio Service**: Porcupine wake word detection, Wav2Vec2 speech recognition, Edge TTS
- **Intelligence Service**: Hybrid local (Qwen3-0.6B) + remote (GPT-4o/DeepSeek) LLM for accessibility
- **Scheduler**: Task coordination and robot state management (Rust + Zenoh)
- **Network Client**: HTTP client for server pathfinding API (Rust)
- **Serial Bridge**: UART communication with STM32 lower controller (Rust + Postcard)
- **Lower Controller**: STM32F407 motor control with Embassy async runtime

**Tech Stack**: Rust (Tokio, Zenoh, Embassy), Python (MediaPipe, OpenCV, Transformers, OpenAI), Protocol Buffers

#### 🎬 **Animations** (`animations/`)

Professional presentation animations using Manim:

- **Project Demonstrations**: Beacon, intro, outro, localization, path, robot, unlock animations
- **Technical Illustrations**: Schema visualizations for documentation
- **Video Generation**: High-quality animation rendering for presentations

**Tech Stack**: Python, Manim, NumPy, SciPy

#### 🥽 **Vision** (`vision/`)

Apple Vision Pro spatial computing application:

- **Immersive Navigation**: AR-based indoor wayfinding
- **Spatial UI**: visionOS native interface with RealityKit
- **Video Playback**: Integrated AVPlayer for media content

**Tech Stack**: Swift, SwiftUI, RealityKit, visionOS

#### 📦 **WeChat Mini Program** (`miniapp/`)

Lightweight WeChat-based navigation experience:

- **WeChat Integration**: Native mini program for Chinese market
- **Cross-platform**: Runs within WeChat super app

**Tech Stack**: TypeScript, WeChat Mini Program SDK

#### 🔧 **Maintenance Tool** (`admin/maintenance/`)

ESP32-C3 key management and provisioning CLI:

- **Key Generation**: P-256 private/public key pair generation
- **Efuse Programming**: Secure key storage in ESP32-C3 hardware
- **Metadata Management**: Key tracking and certification

**Tech Stack**: Rust, Clap, P-256, esptool integration

#### 🔄 **TypeScript Schema** (`ts-schema/`)

Rust-to-TypeScript schema generator using ts-rs:

- **Type Safety**: Automatic TypeScript definitions from Rust types
- **Compile-time Generation**: Types generated during Rust compilation
- **Zero Runtime Cost**: Pure compile-time code generation

**Tech Stack**: Rust, ts-rs, TypeScript

#### 📚 **Shared** (`shared/`)

Common types and utilities shared across Rust components:

- **no_std Compatible**: Embedded-friendly with optional std support
- **Feature Flags**: Configurable dependencies (heapless, serde, crypto, mongodb, postgres, ts-rs)
- **Cryptographic Primitives**: Shared crypto types for beacon/server communication
- **Advanced Pathfinding**: A* inner-area routing, Dijkstra inter-area routing, triangulation for non-Manhattan polygons

**Tech Stack**: Rust (no_std), Serde, P-256, HMAC, SHA-2, Postcard

#### 🤖 **Admin** (`admin/`)

Robot management system with Rust orchestrator and Go tower:

- **Orchestrator (Rust)**: gRPC server for task scheduling, robot selection, and business logic
- **Tower (Go)**: gRPC client + Socket.IO server for robot connection management
- **One-Goroutine-Per-Robot**: Dedicated goroutines for keep-alive and status reporting
- **Streaming gRPC**: Real-time task assignments from Rust to Go
- **Protocol Buffers**: Type-safe communication schema

**Tech Stack**: Rust (Tokio, Tonic), Go (gRPC, Socket.IO), Protocol Buffers

#### ⚡ **Schematics** (`schematics/`)

KiCad PCB designs for custom beacon hardware:

- **Gerber Files**: Manufacturing-ready PCB designs
- **Multi-layer Boards**: F/B copper, mask, paste, silkscreen layers

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.86+ (for server, beacon, admin/maintenance, shared, ts-schema, admin/orchestrator)
- **Node.js** 18+ with **pnpm** (for mobile, miniapp, ts-schema)
- **Python** 3.12+ with **uv** (for robot/vision, robot/audio, robot/intelligence, animations)
- **Xcode** 16+ (for vision app, iOS/macOS builds)
- **just** command runner
- **ESP-IDF** (for beacon development)

### Quick Start

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd navign
   ```

2. **Install dependencies**:
   ```bash
   just init
   ```

   This will:
    - Install Rust tools (cargo-deny, cargo-shear, typos-cli)
    - Install pnpm dependencies for Node.js projects
    - Sync Python virtual environments with uv
    - Build all Rust components

3. **Format code**:
   ```bash
   just fmt
   ```

4. **Lint and check**:
   ```bash
   just lint
   ```

5. **Fix issues automatically**:
   ```bash
   just fix
   ```

### Running Individual Components

#### Server

```bash
cd server
cargo run --release
```

#### Mobile App

```bash
cd mobile
pnpm dev          # Development mode
pnpm tauri dev    # Tauri development with hot reload
pnpm tauri build  # Production build
```

#### Firmware (ESP32-C3)

```bash
cd firmware
cargo build --release
# Flash to ESP32-C3 using espflash
```

#### Robot (Vision/Audio/Intelligence Services)

```bash
cd robot/vision
uv run python service.py

cd robot/audio
uv run python service.py

cd robot/intelligence
uv run python service.py  # (Zenoh integration pending)
```

#### Animations

```bash
cd animations
uv run manim intro.py NavignIntro
```

#### Vision (Apple Vision Pro)

```bash
# Open vision/vision.xcodeproj in Xcode
# Build and run on Vision Pro simulator or device
```

#### Maintenance Tool

```bash
cd admin/maintenance
cargo run -- fuse-priv-key --output-dir ./keys --port /dev/ttyUSB0
```

## 🔐 Security Features

- **End-to-End Encryption**: P-256 ECDSA signatures for all beacon communications
- **Hardware Security**: ESP32-C3 efuse-based key storage
- **Replay Attack Prevention**: Nonce-based challenge-response with time windows
- **Biometric Authentication**: Native platform biometric support
- **Secure Storage**: Tauri Stronghold for credential management
- **Rate Limiting**: Configurable unlock attempt limits (5 attempts per 5 minutes)

## 🗺️ Navigation Features

- **Advanced Pathfinding**: Dijkstra inter-area + A* inner-area with triangulation for non-Manhattan polygons
- **Multi-floor Support**: Elevator, escalator, and stair routing
- **Point-to-Point Navigation**: Coordinate-based and merchant-based routing
- **Real-time Positioning**: BLE RSSI triangulation for <2m accuracy
- **Area Connectivity Graph**: Dynamic graph generation for complex layouts
- **Visibility Graph**: Optimal paths around irregular obstacles

## 🏪 Entity Management

Supports multiple entity types:

- **Malls**: Shopping centers with multiple floors and merchants
- **Transportation Hubs**: Airports, train stations, bus terminals
- **Schools**: Campus navigation with rooms and facilities
- **Hospitals**: Patient and visitor navigation

Merchant types include:

- Food (with cuisine categorization)
- Electronics
- Clothing
- Supermarket
- Health & Beauty
- Entertainment
- Facilities & Services

## 📊 Technology Stack Summary

| Component        | Languages        | Key Technologies                     |
|------------------|------------------|--------------------------------------|
| Server           | Rust             | Axum, Tokio, MongoDB, PostgreSQL, JWT, P-256 |
| Mobile           | TypeScript, Rust | Vue 3, Tauri 2.0, MapLibre GL, Konva |
| Firmware         | Rust             | ESP32-C3, BLE, embedded-hal, P-256   |
| Robot/Upper      | Rust, Python     | Tokio, Zenoh, Transformers, OpenAI, MediaPipe |
| Robot/Lower      | Rust             | STM32F407, Embassy, defmt            |
| Admin            | Rust, Go         | Tokio, Tonic, gRPC, Socket.IO, Protobuf |
| Animations       | Python           | Manim, NumPy, SciPy                  |
| Vision           | Swift            | SwiftUI, RealityKit, visionOS        |
| Mini App         | TypeScript       | WeChat Mini Program SDK              |
| Maintenance Tool | Rust             | Clap, P-256, esptool                 |
| TS Schema        | Rust             | ts-rs                                |
| Shared           | Rust             | no_std, Serde, P-256, Pathfinding    |
| Docs             | Markdown         | VitePress                            |

## 📝 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

Copyright © 2025 Navign

## 🤝 Contributing

Contributions are welcome! Please follow the existing code style and run `just fmt` and `just lint` before submitting
pull requests.

## 📞 Support

For technical documentation, refer to individual component READMEs:

- [CLAUDE.md - AI Assistant Development Guide](CLAUDE.md)
- [Server Documentation](server/README.md)
- [Mobile App Documentation](mobile/README.md)
- [Firmware Documentation](firmware/README.md)
- [Robot Documentation](robot/README.md)
- [TypeScript Schema Documentation](ts-schema/README.md)

## 🎯 Roadmap

- [x] PostgreSQL migration layer (dual-database support)
- [x] Robot upper layer components (Vision, Audio, Intelligence, Scheduler, Network, Serial)
- [x] Advanced pathfinding with triangulation
- [x] Intelligence service for accessibility (hybrid LLM)
- [ ] Zenoh integration for robot components
- [ ] Complete robot motor control logic
- [ ] Enhanced gesture recognition models
- [ ] Multi-language support expansion
- [ ] Advanced analytics dashboard
- [ ] Mesh network for beacons
- [ ] Offline-first mobile experience

---

**Built with ❤️ for the future of indoor navigation**
