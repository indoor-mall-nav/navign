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
├── beacon/              # ESP32-C3 BLE beacon firmware (Rust embedded)
├── miniapp/             # WeChat Mini Program
├── gesture_space/       # Computer vision gesture recognition (Python)
├── animations/          # Manim animations for presentations (Python)
├── presentation/        # Slidev presentation for GestureSpace project
├── vision/              # Apple Vision Pro spatial computing app (Swift)
├── robot/               # Robotics delivery system (empty/planned)
├── maintenance-tool/    # ESP32-C3 key management CLI (Rust)
├── ts-schema/           # TypeScript schema generator (Rust NAPI)
├── shared/              # Shared Rust types (no_std compatible)
└── schematics/          # KiCad PCB designs for hardware
```

## 🏗️ System Architecture

### Core Components

#### 🖥️ **Server** (`server/`)

High-performance Rust backend providing:

- **Advanced Pathfinding**: Dijkstra-based algorithm with bump allocation for ultra-fast routing
- **Multi-floor Navigation**: Support for elevators, escalators, and stairs
- **Beacon Authentication**: TOTP-based secure access control with P-256 ECDSA signatures
- **RESTful API**: Full CRUD operations for entities, areas, merchants, beacons
- **OAuth2 Integration**: GitHub, Google, WeChat authentication
- **MongoDB Storage**: Document-based storage (planned PostgreSQL migration)

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

#### 📡 **Beacon** (`beacon/`)

Secure BLE beacon firmware for ESP32-C3 microcontrollers:

- **BLE Advertising**: Multi-service broadcasting (Authorization, Location, Navigation, Environmental)
- **Cryptographic Security**: P-256 ECDSA signatures, nonce-based challenge-response
- **Device Types**: Merchant, Pathway, Connection, Turnstile
- **Hardware Control**: Relay, servo motor, IR transmitter, DHT11 sensor
- **Replay Attack Prevention**: Time-windowed nonce validation with rate limiting
- **Efuse Key Storage**: Hardware-secured private keys in ESP32-C3 efuse BLOCK_KEY0

**Tech Stack**: Rust (embedded), ESP32-C3, BLE (bleps), P-256 ECDSA, DHT11

#### 🖐️ **Gesture Space** (`gesture_space/`)

Advanced computer vision system for gesture-based control:

- **Hand Gesture Recognition**: MediaPipe-based finger tracking and gesture detection
- **Object Detection**: YOLOv8 (Ultralytics) for environment understanding
- **Voice Wake Word**: Porcupine wake word detection
- **Speech Recognition**: Audio recording and recognition
- **3D Localization**: Camera pose estimation and 3D point mapping
- **AprilTag Detection**: Marker-based positioning and calibration

**Tech Stack**: Python, MediaPipe, OpenCV, PyTorch, Ultralytics (YOLOv8), Porcupine, AprilTags

#### 🎬 **Animations** (`animations/`)

Professional presentation animations using Manim:

- **Project Demonstrations**: Beacon, intro, outro, localization, path, robot, unlock animations
- **Technical Illustrations**: Schema visualizations for documentation
- **Video Generation**: High-quality animation rendering for presentations

**Tech Stack**: Python, Manim, NumPy, SciPy

#### 📊 **Presentation** (`presentation/`)

Interactive Slidev presentation for the GestureSpace project:

- **4.5-minute presentation** covering market situation, GestureSpace techniques, Navign integration, and robot system
- **Interactive slides** with code highlighting, diagrams, and animations
- **Export capabilities**: PDF, PNG, or host as web application
- **Responsive design** for presenting on any device

**Tech Stack**: Slidev, Vue, Markdown, Mermaid

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

#### 🔧 **Maintenance Tool** (`maintenance-tool/`)

ESP32-C3 key management and provisioning CLI:

- **Key Generation**: P-256 private/public key pair generation
- **Efuse Programming**: Secure key storage in ESP32-C3 hardware
- **Metadata Management**: Key tracking and certification

**Tech Stack**: Rust, Clap, P-256, esptool integration

#### 🔄 **TypeScript Schema** (`ts-schema/`)

Rust-to-TypeScript schema generator using N-API:

- **Type Safety**: Automatic TypeScript definitions from Rust types
- **Native Performance**: Rust-powered schema generation
- **Cross-platform**: Supports macOS, Windows, Linux, WASM

**Tech Stack**: Rust, NAPI-RS, TypeScript

#### 📚 **Shared** (`shared/`)

Common types and utilities shared across Rust components:

- **no_std Compatible**: Embedded-friendly with optional std support
- **Feature Flags**: Configurable dependencies (heapless, serde, crypto, base64)
- **Cryptographic Primitives**: Shared crypto types for beacon/server communication

**Tech Stack**: Rust (no_std), Serde, P-256, HMAC, SHA-2

#### 🤖 **Robot** (`robot/`)

Robotics delivery system (planned/in development):

- **Upper/Lower Components**: Modular robot design
- **Autonomous Delivery**: Integration with navigation system

**Status**: In development

#### ⚡ **Schematics** (`schematics/`)

KiCad PCB designs for custom beacon hardware:

- **Gerber Files**: Manufacturing-ready PCB designs
- **Multi-layer Boards**: F/B copper, mask, paste, silkscreen layers

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.86+ (for server, beacon, maintenance-tool, shared, ts-schema)
- **Node.js** 18+ with **pnpm** (for mobile, miniapp, ts-schema)
- **Python** 3.12+ with **uv** (for gesture_space, animations)
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

#### Beacon (ESP32-C3)

```bash
cd beacon
cargo build --release
# Flash to ESP32-C3 using espflash
```

#### Gesture Space

```bash
cd gesture_space
uv run python main.py
```

#### Animations

```bash
cd animations
uv run manim intro.py NavignIntro
```

#### Presentation (Slidev)

```bash
cd presentation
pnpm dev          # Start presentation in dev mode
pnpm build        # Build for production
pnpm export       # Export as PDF
```

#### Vision (Apple Vision Pro)

```bash
# Open vision/vision.xcodeproj in Xcode
# Build and run on Vision Pro simulator or device
```

#### Maintenance Tool

```bash
cd maintenance-tool
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

- **Intelligent Pathfinding**: Optimized Dijkstra algorithm with bump allocation
- **Multi-floor Support**: Elevator, escalator, and stair routing
- **Point-to-Point Navigation**: Coordinate-based and merchant-based routing
- **Real-time Positioning**: BLE RSSI triangulation for <2m accuracy
- **Area Connectivity Graph**: Dynamic graph generation for complex layouts
- **Agent Instance Pattern**: Smart handling of single-entry areas

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
| Server           | Rust             | Axum, Tokio, MongoDB, JWT, P-256     |
| Mobile           | TypeScript, Rust | Vue 3, Tauri 2.0, MapLibre GL, Konva |
| Beacon           | Rust             | ESP32-C3, BLE, embedded-hal          |
| Gesture Space    | Python           | MediaPipe, OpenCV, PyTorch, YOLOv8   |
| Animations       | Python           | Manim, NumPy, SciPy                  |
| Presentation     | Markdown, Vue    | Slidev, Mermaid                      |
| Vision           | Swift            | SwiftUI, RealityKit, visionOS        |
| Mini App         | TypeScript       | WeChat Mini Program SDK              |
| Maintenance Tool | Rust             | Clap, P-256, esptool                 |
| TS Schema        | Rust             | NAPI-RS                              |
| Shared           | Rust             | no_std, Serde, P-256                 |
| Docs             | Markdown         | VitePress                            |

## 📝 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

Copyright © 2025 Navign

## 🤝 Contributing

Contributions are welcome! Please follow the existing code style and run `just fmt` and `just lint` before submitting
pull requests.

## 📞 Support

For technical documentation, refer to individual component READMEs:

- [Server Documentation](server/README.md)
- [Mobile App Documentation](mobile/README.md)
- [Beacon Documentation](beacon/README.md)
- [TypeScript Schema Documentation](ts-schema/README.md)
- [GestureSpace Presentation](presentation/README.md)
- [Presentation Outline](PRESENTATION_OUTLINE.md)

## 🎯 Roadmap

- [ ] PostgreSQL migration (replace MongoDB)
- [ ] Complete robot delivery system
- [ ] Enhanced gesture recognition models
- [ ] Multi-language support expansion
- [ ] Advanced analytics dashboard
- [ ] Mesh network for beacons
- [ ] Offline-first mobile experience

---

**Built with ❤️ for the future of indoor navigation**
