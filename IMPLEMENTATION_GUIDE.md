# Indoor Mall Navigation System

A comprehensive indoor navigation system built with Tauri, Vue 3, and Rust, featuring real-time location tracking using BLE beacons, map visualization, and secure device unlocking.

## 🚀 Features

### Backend (Rust)
- **Authentication System**: Login, registration, and guest access with JWT tokens
- **Location Services**: Real-time indoor positioning using BLE beacon triangulation
- **Map Generation**: Dynamic SVG map generation for areas with beacons and merchants
- **Secure Unlocking**: ECDSA-based cryptographic protocol for secure device unlocking
- **Database Integration**: SQLite database for caching area, beacon, and merchant data
- **BLE Communication**: Low-level Bluetooth communication with beacon devices

### Frontend (Vue 3 + TypeScript)
- **Interactive Map Display**: SVG-based map rendering with zoom and pan controls
- **Location Tracking**: Real-time position display on the map
- **Merchant Search**: Search and filter merchants by name and tags
- **Login/Registration**: User authentication with form validation
- **Beacon Scanning**: Discover and connect to nearby beacons
- **Session Management**: Persistent session state using Pinia

## 📋 Prerequisites

- **Node.js**: v18+ and pnpm
- **Rust**: 1.70+ (install via [rustup](https://rustup.rs/))
- **Tauri CLI**: Install with `cargo install tauri-cli`
- **Mobile Development** (optional):
  - Android: Android Studio with NDK
  - iOS: Xcode 14+

## 🛠️ Installation

### 1. Clone the Repository
```bash
git clone <repository-url>
cd indoor-mall/mobile
```

### 2. Install Frontend Dependencies
```bash
pnpm install
```

### 3. Build Rust Backend
```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

## 🏃 Running the Application

### Development Mode (Desktop)
```bash
# Start the development server
pnpm run tauri dev
```

### Build for Production
```bash
# Build desktop application
pnpm run tauri build
```

### Mobile Development

#### Android
```bash
# Add Android target
rustup target add aarch64-linux-android armv7-linux-androideabi

# Run on Android
pnpm run tauri android dev
```

#### iOS
```bash
# Add iOS target
rustup target add aarch64-apple-ios aarch64-apple-ios-sim

# Run on iOS simulator
pnpm run tauri ios dev
```

## 🏗️ Architecture

### Backend Structure (Rust)
```
src-tauri/src/
├── lib.rs                 # Main entry point, command registration
├── api/
│   ├── login.rs          # Authentication API
│   ├── map.rs            # Map data and SVG generation
│   └── page_results.rs   # Pagination helpers
├── locate/
│   ├── area.rs           # Area data models
│   ├── beacon.rs         # Beacon data models
│   ├── locator.rs        # Triangulation algorithm
│   ├── merchant.rs       # Merchant data models
│   └── scan.rs           # BLE scanning
├── login/
│   ├── handshake.rs      # Server key exchange
│   └── handlers.rs       # Login command handlers
└── unlocker/
    ├── challenge.rs      # Challenge generation
    ├── proof.rs          # Cryptographic proof
    ├── pipeline.rs       # Unlock workflow
    └── utils.rs          # BLE message protocol
```

### Frontend Structure (Vue)
```
src/
├── views/
│   ├── LoginView.vue     # Authentication page
│   ├── HomeView.vue      # Main navigation view
│   └── SplashScreen.vue  # Initial loading screen
├── components/
│   ├── map/
│   │   └── MapDisplay.vue # Interactive map component
│   └── ui/               # Reusable UI components
├── lib/
│   ├── api/
│   │   └── tauri.ts      # Tauri invoke wrappers
│   └── utils.ts          # Utility functions
└── states/
    └── session.ts        # Pinia session store
```

## 🔧 Configuration

### Backend Configuration
Edit `src-tauri/src/shared.rs` to set your backend URL:
```rust
pub const BASE_URL: &str = "http://your-server:3000/";
```

### Frontend Configuration
Edit `src/lib/shared.ts`:
```typescript
export const baseUrl = import.meta.env.VITE_BASE_URL || "http://your-server:3000";
```

## 📡 Tauri Commands (Backend API)

### Authentication
- `login_handler(email: String, password: String)` - User login
- `register_handler(email: String, username: String, password: String)` - User registration
- `guest_login_handler()` - Guest access
- `logout_handler(token: String)` - Logout
- `validate_token_handler(token: String)` - Token validation

### Location & Navigation
- `locate_handler(area: String, entity: String)` - Get current position
- `get_map_data_handler(entity: String, area: String)` - Fetch map data
- `generate_svg_map_handler(entity: String, area: String, width: u32, height: u32)` - Generate SVG map
- `search_merchants_handler(entity: String, query: String)` - Search merchants

### Device Unlocking
- `unlock_handler(entity: String, target: String)` - Unlock a device/gate
- `bind_with_server()` - Bind device with server

## 🔐 Security Features

### Cryptographic Protocol
- **ECDSA Signing**: P-256 curve for device authentication
- **RSA Encryption**: 2048-bit keys for AES key exchange
- **AES-256-GCM**: Symmetric encryption for payload
- **Stronghold Storage**: Secure key storage with biometric protection
- **Challenge-Response**: Prevents replay attacks

### Authentication Flow
1. Device generates ECDSA keypair (stored in Stronghold)
2. Public key exchange with server using RSA encryption
3. Server issues challenges with nonce
4. Device signs challenges and sends proof
5. Server validates and grants access

## 📊 Database Schema

The SQLite database (`navign.db`) stores:
- **active_areas**: Cached area polygons and metadata
- **beacons**: BLE beacon locations and properties
- **merchants**: Store/merchant information

## 🎨 UI Components

### Key Components
- **MapDisplay.vue**: Interactive SVG map with zoom/pan
- **LoginView.vue**: Authentication forms
- **HomeView.vue**: Main navigation interface
- **Badge, Button, Card, Input**: Shadcn-inspired UI components

## 🧪 Testing

```bash
# Run frontend tests
pnpm run test

# Run Rust tests
cargo test --manifest-path src-tauri/Cargo.toml
```

## 🔍 Troubleshooting

### Backend Issues
- **Compilation errors**: Ensure Rust 1.70+ is installed
- **Database errors**: Check `navign.db` permissions
- **BLE connection fails**: Verify Bluetooth permissions on mobile

### Frontend Issues
- **TypeScript errors**: Run `pnpm install` to update dependencies
- **Build fails**: Clear node_modules and reinstall
- **API connection fails**: Check `BASE_URL` configuration

### Mobile-Specific
- **Android**: Ensure all permissions are granted in AndroidManifest.xml
- **iOS**: Check Info.plist for Bluetooth and Location permissions

## 📱 Mobile Permissions Required

### Android
- `BLUETOOTH_SCAN`
- `BLUETOOTH_CONNECT`
- `ACCESS_FINE_LOCATION`
- `ACCESS_COARSE_LOCATION`

### iOS
- `NSBluetoothAlwaysUsageDescription`
- `NSLocationWhenInUseUsageDescription`
- `NSLocationAlwaysUsageDescription`

## 🚦 Development Workflow

1. **Start backend development**: Run `cargo watch -x check` for auto-compilation
2. **Start frontend development**: Run `pnpm run dev` for hot reload
3. **Test Tauri commands**: Use the browser console in dev mode
4. **Debug Rust code**: Use `println!` or attach debugger
5. **Inspect frontend**: Use Vue DevTools browser extension

## 📄 License

MIT License - See LICENSE file for details

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

## 📞 Support

For issues and questions, please open an issue on the repository.

---

**Note**: This application requires a backend server to function. Ensure your backend server is running and accessible at the configured `BASE_URL`.

