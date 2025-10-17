# Navign Mobile

A cross-platform indoor navigation mobile application for malls and other large indoor spaces, built with Vue.js and Tauri.

## Overview

Navign is an indoor navigation system that uses Bluetooth Low Energy (BLE) beacons to provide real-time location tracking and navigation within shopping malls, transportation hubs, schools, and hospitals. The app features biometric authentication, secure door unlocking, and interactive maps with merchant information.

## Features

- **Indoor Positioning**: Real-time location tracking using BLE beacon triangulation
- **Interactive Maps**: Visual representation of indoor spaces with polygon-based areas and merchant locations
- **Secure Authentication**: Biometric authentication support for secure access
- **Smart Door Unlocking**: Cryptographically secure door unlocking via BLE communication
- **Multi-language Support**: Internationalization (i18n) with English and Chinese locales
- **Merchant Discovery**: Browse and discover stores within the indoor space
- **Geolocation Integration**: Combined GPS and BLE positioning for accurate location tracking
- **Cross-platform**: Supports iOS, Android, and desktop platforms

## Tech Stack

### Frontend

- **Framework**: Vue 3 with TypeScript
- **UI Components**: Shadcn Vue
- **Styling**: Tailwind CSS 4
- **Maps**: MapLibre GL with Konva for canvas rendering
- **State Management**: Pinia with persistence
- **Routing**: Vue Router
- **Form Validation**: Vee-Validate with Zod schemas
- **Animations**: Motion-v

### Backend (Tauri)

- **Runtime**: Tauri 2.0
- **Language**: Rust
- **Database**: SQLite with SQLx
- **Bluetooth**: Custom BLE integration via tauri-plugin-blec
- **Cryptography**:
  - P256 ECDSA for signature verification
  - AES-GCM for encryption
  - RSA for key exchange
- **Plugins**:
  - Biometric authentication
  - Geolocation
  - HTTP client
  - NFC support
  - Notifications
  - SQL database

## Project Structure

```
mobile/
├── src/                          # Vue.js frontend source
│   ├── components/               # Vue components
│   │   ├── cards/               # Area, Entity, Merchant cards
│   │   ├── map/                 # Map-related components
│   │   └── ui/                  # Reusable UI components
│   ├── i18n/                    # Internationalization
│   ├── lib/                     # Utility libraries
│   │   ├── map/                 # Map utilities
│   │   ├── structure/           # Data structures
│   │   └── unlocker/            # Door unlock protocol
│   ├── router/                  # Vue Router configuration
│   ├── schema/                  # TypeScript type definitions
│   ├── states/                  # Pinia stores
│   └── views/                   # Page components
├── src-tauri/                   # Tauri backend (Rust)
│   └── src/
│       ├── api/                 # API communication layer
│       ├── locate/              # Indoor positioning logic
│       ├── login/               # Authentication
│       └── unlocker/            # Door unlock implementation
└── src-tauri/gen/               # Generated platform-specific code
    ├── android/                 # Android build configuration
    └── apple/                   # iOS/macOS build configuration
```

## Prerequisites

- **Node.js**: v18 or higher (22.14.0 recommended)
- **pnpm**: Package manager
- **Rust**: Latest stable version
- **Tauri CLI**: v2.0 or higher
- **Platform-specific tools**:
  - For Android: Android Studio, SDK, NDK
  - For iOS: Xcode (macOS only)
  - For desktop: Platform-specific system dependencies

## Installation

1. **Clone the repository**:

   ```bash
   git clone <repository-url>
   cd mobile
   ```

2. **Install dependencies**:

   ```bash
   pnpm install
   ```

3. **Set up environment variables**:
   Create a `.env` file in the root directory:

   ```env
   VITE_BASE_URL=http://your-server-url:3000
   ```

4. **Initialize the database**:
   The SQLite database will be created automatically on first run.

## Development

### Run in development mode

```bash
# Desktop development
pnpm tauri dev

# Mobile development (Android)
pnpm tauri android dev

# Mobile development (iOS, macOS only)
pnpm tauri ios dev
```

### Available scripts

```bash
# Frontend development server
pnpm dev

# Build frontend only
pnpm build

# Preview production build
pnpm preview

# Format code (frontend + backend)
just format

# Lint code
just lint

# Fix linting issues
just fix

# Run tests
just test
```

## Building for Production

### Desktop

```bash
pnpm tauri build
```

### Android

```bash
pnpm tauri android build
```

### iOS (macOS only)

```bash
pnpm tauri ios build
```

Build artifacts will be located in `src-tauri/target/release/` (desktop) or `src-tauri/gen/android/` and `src-tauri/gen/apple/` (mobile).

## Architecture

### Indoor Positioning System

The app uses a sophisticated BLE beacon-based positioning system:

1. **Beacon Scanning**: Continuously scans for nearby BLE beacons
2. **Signal Processing**: Calculates distance using RSSI (Received Signal Strength Indicator)
3. **Trilateration**: Determines precise location using multiple beacon signals
4. **Area Detection**: Identifies the current area within the indoor space
5. **Position Updates**: Real-time position updates as the user moves

### Door Unlock Protocol

The secure door unlocking feature uses a challenge-response protocol:

1. **Device Inquiry**: Discovers BLE-enabled door locks
2. **Challenge Request**: Requests a cryptographic challenge from the lock
3. **Biometric Authentication**: User authenticates with biometrics
4. **Signature Generation**: Creates a cryptographic signature using the device's private key
5. **Challenge Response**: Sends the signed response to unlock the door
6. **Verification**: Lock verifies the signature and grants access

### Data Schema

The application works with several key entities:

- **Entity**: Top-level container (mall, transportation hub, etc.)
- **Area**: Defined spaces within an entity (floors, zones)
- **Beacon**: BLE devices for positioning
- **Merchant**: Stores or points of interest
- **Connection**: Pathways between areas

## Configuration

### Tauri Configuration

Main configuration file: `src-tauri/tauri.conf.json`

Key settings:

- **Product Name**: Navign
- **Bundle Identifier**: com.ethan.mallnav
- **Minimum Android SDK**: 26
- **Window Dimensions**: 800x600 (desktop)

### Vite Configuration

Located in `vite.config.ts`:

- Development server runs on port 1420
- HMR on port 1421
- Path alias `@` points to `./src`

## Testing

```bash
# Run all tests
just test

# Frontend tests only
pnpm test

# Backend tests only
cd src-tauri && cargo test
```

## Code Quality

### Formatting

```bash
# Format all code
just format

# Frontend only (Prettier)
pnpm run format

# Backend only (rustfmt)
cd src-tauri && cargo fmt
```

### Linting

```bash
# Lint all code
just lint

# Frontend only (oxlint)
pnpm run lint

# Backend only (clippy)
cd src-tauri && cargo clippy
```

## Troubleshooting

### Common Issues

1. **BLE not working**: Ensure Bluetooth permissions are granted in device settings
2. **Database errors**: Delete `navign.db` and restart the app to reinitialize
3. **Build failures**: Clean build artifacts with `cargo clean` and rebuild
4. **Connection issues**: Verify `VITE_BASE_URL` environment variable is set correctly

### Debug Mode

The app includes VConsole for mobile debugging. Access it by tapping the debug icon in development mode.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Follow the existing code style
- Run `just format` before committing
- Ensure `just lint` passes without errors
- Write tests for new features

## Security

- All cryptographic operations use industry-standard algorithms
- Private keys are stored securely using Tauri's stronghold plugin
- Network communications should use HTTPS in production
- Biometric data never leaves the device

## License

MIT License - Copyright (c) 2025 Ethan Wu

See [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Tauri](https://tauri.app/)
- UI components based on [Shadcn Vue](https://shadcn-vue.com/)
- State management with [Pinia](https://pinia.vuejs.org/)
- Maps powered by [MapLibre GL](https://maplibre.org/)
- BLE integration via [tauri-plugin-blec](https://github.com/mnlphlp/tauri-plugin-blec)

## Contact

For questions, issues, or contributions, please open an issue on the repository.
