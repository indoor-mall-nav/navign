# Quick Start Guide

## 🎯 Implementation Summary

I've successfully implemented comprehensive backend functionality in Rust and frontend integration for your indoor mall navigation system. Here's what was added:

## ✅ Completed Features

### Backend (Rust) - All functionality in Tauri commands

#### 1. Authentication System (`src-tauri/src/api/login.rs` + `src-tauri/src/login/handlers.rs`)
- ✅ `login_handler` - User login with email/password
- ✅ `register_handler` - New user registration
- ✅ `guest_login_handler` - Guest access without credentials
- ✅ `logout_handler` - Session termination
- ✅ `validate_token_handler` - JWT token validation

#### 2. Map Display System (`src-tauri/src/api/map.rs`)
- ✅ `get_map_data_handler` - Fetch area, beacons, and merchants
- ✅ `generate_svg_map_handler` - Generate SVG maps with proper scaling
- ✅ `search_merchants_handler` - Search merchants by name
`- ✅ Dynamic SVG generation with:
  - Area polygons with boundaries
  - Beacon markers (red circles)
  - Merchant polygons (blue)
  - Labels and interactive elements
`
#### 3. Location Services (Already existed, enhanced)
- ✅ `locate_handler` - BLE-based triangulation
- ✅ Beacon scanning and caching
- ✅ Database integration for offline support

#### 4. Device Unlocking (Already existed)
- ✅ `unlock_handler` - Secure cryptographic unlocking
- ✅ `bind_with_server` - Device registration

### Frontend (Vue 3 + TypeScript) - Minimal UI-focused components

#### 1. Authentication UI (`src/views/LoginView.vue`)
- ✅ Login form with validation
- ✅ Registration form with terms acceptance
- ✅ Guest login button
- ✅ Error handling and loading states
- ✅ Social login placeholders (Google, GitHub, WeChat)

#### 2. Map Display Component (`src/components/map/MapDisplay.vue`)
- ✅ SVG map rendering with zoom controls
- ✅ Beacon and merchant filtering
- ✅ Search functionality
- ✅ Interactive click handlers
- ✅ Loading states and error handling

#### 3. Home View (`src/views/HomeView.vue`)
- ✅ Location status display
- ✅ "Locate Me" button with backend integration
- ✅ Nearby merchants sidebar
- ✅ Quick actions panel
- ✅ Map integration

#### 4. API Integration Layer (`src/lib/api/tauri.ts`)
- ✅ Type-safe Tauri invoke wrappers
- ✅ Consistent error handling
- ✅ Response parsing utilities

#### 5. Session Management (`src/states/session.ts`)
- ✅ Pinia store with persistence
- ✅ User authentication state
- ✅ Location tracking
- ✅ Entity and area management

#### 6. UI Components
- ✅ Badge component for tags
- ✅ Skeleton component for loading states
- ✅ All existing Shadcn components integrated

## 🏃 How to Run

### 1. First Time Setup
```bash
cd /Users/ethangoh/Developer/indoor-mall/mobile

# Install dependencies
pnpm install

# Verify backend compiles
cargo check --manifest-path src-tauri/Cargo.toml
```

### 2. Development Mode
```bash
# Start Tauri development server (this compiles Rust + starts Vue dev server)
pnpm run tauri dev
```

### 3. Mobile Development
```bash
# Android
pnpm run tauri android dev

# iOS
pnpm run tauri ios dev
```

## 🔌 Backend Integration

All heavy lifting is done in Rust. The frontend only needs to:

1. **Call Tauri commands** via `invoke()`
2. **Display data** in Vue components
3. **Handle user interactions**

Example usage:
```typescript
import { login, getMapData, locateDevice } from '@/lib/api/tauri'

// Login
const result = await login('user@example.com', 'password')
if (result.status === 'success') {
  console.log('Token:', result.token)
}

// Get map
const map = await getMapData('entityId', 'areaId')
console.log('SVG:', map.svg)

// Locate device
const position = await locateDevice('areaId', 'entityId')
console.log('Position:', position.x, position.y)
```

## 📁 Key Files Modified/Created

### Backend (Rust)
- ✅ `src-tauri/src/lib.rs` - Registered all new commands
- ✅ `src-tauri/src/api/login.rs` - Authentication logic
- ✅ `src-tauri/src/api/map.rs` - Map generation and data
- ✅ `src-tauri/src/login/handlers.rs` - Command handlers
- ✅ `src-tauri/src/locate/mod.rs` - Made modules public

### Frontend (Vue)
- ✅ `src/views/LoginView.vue` - Complete authentication UI
- ✅ `src/views/HomeView.vue` - Main navigation interface
- ✅ `src/components/map/MapDisplay.vue` - Interactive map
- ✅ `src/lib/api/tauri.ts` - Tauri integration layer
- ✅ `src/states/session.ts` - Enhanced session management
- ✅ `src/main.ts` - Added Pinia persistence

### Configuration
- ✅ `tsconfig.json` - Fixed JSX configuration
- ✅ `src/components/ui/badge/` - Created Badge component
- ✅ `src/components/ui/skeleton/` - Created Skeleton component

## 🎨 UI Flow

```
1. App loads → Check authentication (session store)
   ↓
2. Not logged in → Redirect to LoginView
   ↓
3. Login/Register/Guest → Call backend → Store token
   ↓
4. Redirect to HomeView
   ↓
5. Select Entity/Area (or use existing from session)
   ↓
6. Load Map → Call get_map_data_handler → Render SVG
   ↓
7. "Locate Me" → Call locate_handler → Show position
   ↓
8. Click Beacon/Merchant → Show details or unlock
```

## 🔧 Configuration Required

Before running, update these files:

1. **Backend URL** - `src-tauri/src/shared.rs`:
```rust
pub const BASE_URL: &str = "http://YOUR_SERVER:3000/";
```

2. **Frontend URL** - `src/lib/shared.ts`:
```typescript
export const baseUrl = "http://YOUR_SERVER:3000";
```

## ⚠️ Important Notes

1. **Backend Server Required**: The app needs a running backend server at `BASE_URL`
2. **Database**: SQLite `navign.db` is created automatically
3. **Mobile Permissions**: Grant Bluetooth and Location permissions
4. **Biometric**: On mobile, biometric authentication is used for secure key access
5. **Guest Mode**: Works without server authentication for testing

## 🐛 Current Status

✅ **Rust Backend**: Compiles successfully without errors
✅ **Frontend**: All TypeScript types resolved
✅ **Integration**: Tauri commands properly registered
✅ **Components**: All UI components created

## 📝 Next Steps (Optional Enhancements)

1. Add route navigation between beacons
2. Implement pathfinding algorithm
3. Add real-time position updates
4. Enhance map styling and themes
5. Add merchant details modal
6. Implement QR code scanning for quick access
7. Add multi-language support (i18n already configured)
8. Progressive Web App (PWA) support

## 🆘 Troubleshooting

**Build fails?**
```bash
# Clean and rebuild
rm -rf node_modules src-tauri/target
pnpm install
cargo clean --manifest-path src-tauri/Cargo.toml
```

**Type errors?**
```bash
# Regenerate types
npx vue-tsc --noEmit
```

**Database locked?**
```bash
# Remove database
rm navign.db
# It will be recreated on next run
```

---

**Your app is ready to run!** Execute `pnpm run tauri dev` to start development.

