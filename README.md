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

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.86+ (for server, beacon, admin/maintenance, shared, ts-schema, admin/orchestrator)
- **Node.js** 18+ with **pnpm** (for mobile, miniapp, ts-schema)
- **Python** 3.12+ with **uv** (for robot/vision, robot/audio, robot/intelligence, animations)
- **Go** 1.23+ (for admin/tower)
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

## 📝 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

Copyright © 2025 Navign
