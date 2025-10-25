# Navign

A cross-platform indoor navigation system for shopping malls and large indoor spaces, built with Vue.js and Tauri.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.1.0-green.svg)

## Overview

Navign is an advanced indoor positioning and navigation application that leverages Bluetooth Low Energy (BLE) beacon technology to provide real-time location tracking and turn-by-turn navigation within indoor environments such as shopping malls, transportation hubs, schools, and hospitals. The application combines modern web technologies with native mobile capabilities to deliver a seamless cross-platform experience.

## Features

### Core Functionality

- **Indoor Positioning System**: Real-time location tracking using BLE beacon triangulation with RSSI-based distance calculation
- **Turn-by-Turn Navigation**: Step-by-step directions with visual route overlay on interactive maps
- **Interactive Maps**: Polygon-based area rendering with MapLibre GL and Konva canvas support
- **Merchant Discovery**: Search and browse stores within indoor spaces with detailed information
- **Area Management**: Dynamic area switching and multi-floor support

### Security & Authentication

- **Biometric Authentication**: Native biometric support (Touch ID, Face ID, fingerprint) for secure access
- **Secure Door Unlocking**: Cryptographically signed BLE communication for smart door access
  - P256 ECDSA for signature verification
  - AES-GCM encryption for secure message transmission
  - RSA key exchange for encrypted communication
- **Stronghold Integration**: Secure key storage using Tauri's Stronghold plugin
- **Token-based Authentication**: JWT token management with guest and registered user support

### Platform Support

- **iOS**: Native iOS app with full biometric and BLE support
- **Android**: Native Android app (minSdkVersion 26)
- **Desktop**: Development support for macOS, Windows, and Linux

### Localization

- **Multi-language Support**: Built-in i18n infrastructure
- **Supported Languages**: English (en-US), Chinese (zh-CN)

## Tech Stack

### Frontend

| Technology | Purpose |
|------------|---------|
| **Vue 3** | Progressive JavaScript framework with Composition API |
| **TypeScript** | Type-safe development |
| **Reka UI** | Accessible component primitives |
| **Tailwind CSS 4** | Utility-first CSS framework |
| **Pinia** | State management with persistence |
| **Vue Router** | Client-side routing |
| **Vee-Validate + Zod** | Form validation with schema validation |
| **MapLibre GL** | Map rendering library |
| **Konva** | 2D canvas library for custom map elements |
| **VueUse** | Collection of Vue composition utilities |
| **Motion-v** | Animation library |
| **Lucide Vue** | Icon library |
| **dayjs** | Date/time manipulation |

### Backend (Tauri)

| Technology | Purpose |
|------------|---------|
| **Tauri 2.0** | Cross-platform desktop/mobile runtime |
| **Rust** | Systems programming language |
| **SQLite + SQLx** | Local database with compile-time SQL verification |
| **P256** | ECDSA cryptography for signatures |
| **AES-GCM** | Authenticated encryption |
| **RSA** | Asymmetric encryption |
| **SHA-2** | Cryptographic hashing |
| **Tokio** | Async runtime |
| **Serde** | Serialization/deserialization |
| **Chrono** | Date/time handling |

### Tauri Plugins

- `tauri-plugin-blec`: BLE Central mode for beacon scanning
- `tauri-plugin-biometric`: Biometric authentication
- `tauri-plugin-geolocation`: GPS positioning
- `tauri-plugin-sql`: SQLite database
- `tauri-plugin-stronghold`: Secure credential storage
- `tauri-plugin-http`: HTTP client
- `tauri-plugin-nfc`: NFC support
- `tauri-plugin-notification`: Push notifications
- `tauri-plugin-fs`: File system access
- `tauri-plugin-log`: Application logging

## Project Structure

```
mobile/
├── src/                          # Vue.js frontend source
│   ├── assets/                   # Static assets
│   ├── components/               # Vue components
│   │   ├── cards/               # Entity, Merchant, Area cards
│   │   ├── map/                 # Map display and navigation components
│   │   └── ui/                  # Reusable UI components (shadcn-vue style)
│   ├── i18n/                    # Internationalization
│   │   └── locales/             # Translation files
│   ├── lib/                     # Utilities and libraries
│   │   ├── api/                 # Tauri command wrappers
│   │   ├── map/                 # Map utilities
│   │   └── structure/           # Data structures
│   ├── router/                  # Vue Router configuration
│   ├── schema/                  # TypeScript type definitions
│   ├── states/                  # Pinia stores
│   └── views/                   # Page components
│       └── authentication/      # Auth views
├── src-tauri/                   # Rust backend source
│   ├── src/
│   │   ├── api/                # API handlers
│   │   │   ├── login.rs        # Authentication
│   │   │   ├── map.rs          # Map data fetching
│   │   │   └── unlocker.rs     # Door unlocking
│   │   ├── locate/             # Indoor positioning
│   │   │   ├── locator.rs      # Triangulation algorithm
│   │   │   ├── beacon.rs       # Beacon management
│   │   │   └── scan.rs         # BLE scanning
│   │   ├── login/              # Authentication handlers
│   │   │   ├── handlers.rs     # Login/register/logout
│   │   │   └── handshake.rs    # Server binding
│   │   ├── unlocker/           # Secure unlocking module
│   │   └── utils/              # Shared utilities
│   ├── migrations/             # Database migrations
│   ├── capabilities/           # Tauri permission capabilities
│   └── gen/                    # Generated platform code
│       ├── android/            # Android project
│       └── apple/              # iOS/macOS project
├── public/                      # Public assets
├── package.json                # Node dependencies
├── Cargo.toml                  # Rust dependencies
├── tauri.conf.json             # Tauri configuration
├── vite.config.ts              # Vite configuration
├── tsconfig.json               # TypeScript configuration
└── justfile                    # Just command runner recipes
```

## Getting Started

### Prerequisites

- **Node.js** (v18 or higher) and **pnpm**
- **Rust** (latest stable) via [rustup](https://rustup.rs/)
- **Tauri CLI**: `cargo install tauri-cli@^2.0.0`
- **Platform-specific requirements**:
  - **iOS**: Xcode 13+ (macOS only)
  - **Android**: Android Studio with NDK
  - **Desktop**: Platform-specific build tools

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd mobile
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Install Rust dependencies** (automatic on first build)

### Development

#### Desktop Development

```bash
# Run in development mode
pnpm tauri dev

# Or using just
just dev
```

#### Mobile Development

**iOS:**
```bash
pnpm tauri ios dev
```

**Android:**
```bash
pnpm tauri android dev
```

### Building

#### Desktop Build

```bash
pnpm tauri build
```

#### Mobile Build

**iOS:**
```bash
pnpm tauri ios build
```

**Android:**
```bash
pnpm tauri android build
```

## Database Schema

The application uses SQLite for local data caching:

### Tables

#### `active_areas`
Stores area/zone information with polygon boundaries.
- `id` (TEXT PRIMARY KEY): Unique area identifier
- `name` (TEXT): Area name
- `polygon` (TEXT): WKT polygon geometry
- `entity` (TEXT): Parent entity ID
- `updated_at` (INTEGER): Last update timestamp
- `stored_at` (INTEGER): Local storage timestamp

#### `beacons`
BLE beacon information for positioning.
- `id` (TEXT PRIMARY KEY): Beacon identifier
- `mac` (TEXT): MAC address
- `location` (TEXT): WKT point geometry
- `merchant` (TEXT): Associated merchant (FK)
- `area` (TEXT): Associated area (FK)
- `entity` (TEXT): Parent entity ID

#### `merchants`
Store/merchant information.
- `id` (TEXT PRIMARY KEY): Merchant identifier
- `name` (TEXT): Merchant name
- `entry` (TEXT): Entry point location

## Indoor Positioning Algorithm

Navign uses a sophisticated BLE-based triangulation system:

1. **Beacon Scanning**: Continuously scans for nearby BLE beacons
2. **RSSI to Distance Conversion**: Converts signal strength to distance using the formula:
   ```
   distance = 10^((TxPower - RSSI) / (10 * n))
   ```
   where `TxPower = -59 dBm` and `n = 2.0` (free space propagation)

3. **Triangulation**: Uses multiple beacon positions and distances to calculate user location
4. **Area Detection**: Automatically detects area changes based on beacon distribution
5. **Position Smoothing**: Filters out noise and provides stable positioning

## Security Architecture

### Door Unlocking Flow

1. **Key Generation**: ECDSA private key generated and stored in Stronghold
2. **Server Handshake**: Device binds with server using RSA-encrypted AES key
3. **Challenge-Response**: 
   - Server sends encrypted challenge
   - Device signs challenge with ECDSA key
   - Encrypted proof sent via BLE
4. **Verification**: Server verifies signature and grants access

### Biometric Integration

- Keys protected by device biometrics (iOS/Android)
- Stronghold vault encrypted with device-specific salt
- Automatic key derivation using Argon2

## API Integration

The app communicates with a backend server for:
- User authentication and registration
- Map data synchronization
- Merchant information
- Route calculation
- Door unlock authorization

Base URL configuration in `src-tauri/src/shared.rs`

## Development Commands

Using [Just](https://github.com/casey/just) command runner:

```bash
just fmt          # Format code (Prettier + Cargo fmt)
just lint         # Lint code (Oxlint + Clippy)
just fix          # Auto-fix linting issues
just test         # Run tests (Vitest + Cargo test)
just fmt-check    # Check formatting without changes
```

Traditional npm scripts:

```bash
pnpm dev          # Start dev server
pnpm build        # Build frontend
pnpm test         # Run frontend tests
pnpm lint         # Lint frontend
pnpm format       # Format code
```

## Testing

- **Frontend**: Vitest for unit tests
- **Backend**: Cargo's built-in test framework
- **Examples**: See `src/lib/utils.test.ts` and `src/lib/api/tauri.test.ts`

## Configuration

### Environment Variables

- `TAURI_DEV_HOST`: Custom development host for mobile HMR

### Tauri Configuration

Edit `src-tauri/tauri.conf.json` for:
- App identifier and version
- Window configuration
- Build settings
- Bundle targets

### Vite Configuration

Customize `vite.config.ts` for:
- Port settings (default: 1420)
- Path aliases
- Plugin configuration

## Contributing

1. Follow the existing code style
2. Run `just fmt` before committing
3. Ensure `just lint` passes
4. Add tests for new features
5. Update documentation as needed

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright (c) 2025 Ethan Wu

## Acknowledgments

- Built with [Tauri](https://tauri.app/)
- UI components inspired by [shadcn/ui](https://ui.shadcn.com/)
- Indoor positioning algorithms based on BLE beacon triangulation research

---

**Note**: This is a mobile-focused indoor navigation system. Desktop builds are primarily for development purposes. Production use should target iOS and Android platforms.
