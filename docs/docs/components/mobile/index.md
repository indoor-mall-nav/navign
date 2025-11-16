# Mobile Component

The Navign mobile application is a cross-platform app built with Vue 3 and Tauri 2, providing indoor navigation and access control functionality for users.

## Overview

**Location:** `mobile/`

**Technologies:**
- **Frontend:** Vue 3.5.18 (reactive UI)
- **Desktop/Mobile:** Tauri 2.8.1 (native wrapper)
- **State:** Pinia 3.0.3 (state management)
- **Router:** Vue Router 4.5.1
- **Maps:** MapLibre GL 5.6.2 + Konva 9.3.22
- **Styling:** Tailwind CSS 4.1.12

**Platforms:**
- macOS (tested)
- iOS (planned)
- Android (planned)
- Windows (planned)
- Linux (planned)

## Key Features

### 1. Indoor Navigation
- Real-time position tracking via BLE beacon triangulation
- Turn-by-turn navigation instructions
- Multi-floor support with floor selector
- MapLibre GL for base map rendering
- Konva canvas for polygon overlays

### 2. Access Control
- BLE communication with beacons
- P-256 ECDSA signature generation
- Biometric authentication (Touch ID, Face ID)
- Secure credential storage (Stronghold)

### 3. Offline Support
- SQLite for caching entities, areas, merchants
- Downloaded map tiles
- Local pathfinding fallback (planned)

### 4. Admin Panel
- CRUD interface for entities, areas, beacons, merchants
- Polygon drawing and editing
- Connection management
- User management

## Architecture

```
┌─────────────────────────────────────┐
│         Vue 3 Frontend             │
│  (MapLibre + Konva + Reka UI)      │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│      Tauri 2 Rust Backend          │
│  (BLE, Crypto, SQLite, Stronghold) │
└────────────┬────────────────────────┘
             │
       ┌─────┴─────┐
       ▼           ▼
  ┌────────┐  ┌────────┐
  │  BLE   │  │ Server │
  │Beacons │  │  API   │
  └────────┘  └────────┘
```

## Directory Structure

```
mobile/
├── src/                    # Vue frontend
│   ├── main.ts            # App entry point
│   ├── views/             # Vue pages
│   ├── components/        # UI components
│   │   ├── map/          # MapLibre + Konva
│   │   └── ui/           # Reka UI components
│   ├── lib/              # Utilities
│   │   └── api/tauri.ts  # Tauri commands
│   ├── schema/           # TypeScript types
│   ├── states/           # Pinia stores
│   └── i18n/             # Internationalization
├── src-tauri/            # Tauri Rust backend
│   ├── src/lib.rs        # Tauri commands
│   └── Cargo.toml        # Dependencies
├── package.json
└── justfile              # Mobile-specific tasks
```

## Documentation

### Admin Panel
**[Admin Panel Guide](./admin-panel.md)** - Comprehensive CRUD interface documentation

Learn how to use the mobile admin panel to manage entities, areas, beacons, merchants, and connections.

**Features:**
- Entity management with floor configuration
- Area polygon drawing and editing
- Beacon positioning and configuration
- Merchant metadata management
- Connection setup (elevators, stairs, escalators)

---

### gRPC-Web Integration
**[gRPC-Web Integration](./grpc-web-integration.md)** - Connecting to Orchestrator gRPC services

Guide for integrating gRPC-Web in the mobile app to communicate with the Admin Orchestrator for robot fleet management.

**Features:**
- gRPC-Web client setup
- Task submission and monitoring
- Robot status tracking
- Firmware management

---

## Tauri Commands

The mobile app uses Tauri commands to access native functionality:

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

## State Management

The app uses Pinia stores for state management:

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

## Internationalization

The mobile app supports 5 languages:
- English (en-US)
- Simplified Chinese (zh-CN)
- Traditional Chinese (zh-TW)
- Japanese (ja-JP)
- French (fr-FR)

See `mobile/src/i18n/` for translation files.

## Development

### Setup

```bash
cd mobile
pnpm install
```

### Running

```bash
# Development mode
pnpm run dev

# Tauri development
pnpm run tauri dev

# Production build
pnpm run build

# Create app bundle
pnpm run tauri build
```

### Testing

```bash
# Run tests
pnpm test

# Type checking
pnpm run type-check

# Linting
pnpm run lint
```

## Tauri Plugins

The mobile app uses several Tauri plugins:

- **tauri-plugin-blec** - BLE communication
- **tauri-plugin-sql** - SQLite local storage
- **tauri-plugin-biometric** - Touch ID/Face ID
- **tauri-plugin-stronghold** - Secure credential storage
- **tauri-plugin-nfc** - NFC support (planned)

## See Also

- [Main Mobile Documentation](../mobile.md) - Complete mobile component guide
- [Server API](../server.md) - REST API documentation
- [Beacon Protocol](../beacon.md) - BLE protocol specification
- [Shared Types](../shared.md) - TypeScript type definitions
