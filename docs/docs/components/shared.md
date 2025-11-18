# Shared Library

The `shared/` directory contains a `no_std` compatible Rust library that provides common types, schemas, and utilities shared across all components of the Navign system.

## Overview

The shared library is designed to work in multiple environments:
- Embedded systems (ESP32-C3 firmware) using `heapless` collections
- Server and desktop applications using standard library allocations
- Mobile applications with SQLite integration
- Cross-platform type safety with TypeScript generation

## Feature Flags

The library uses feature flags to enable/disable functionality based on the target environment:

- `heapless` - Embedded systems (mutually exclusive with `alloc`)
- `alloc` - Heap allocation (mutually exclusive with `heapless`)
- `std` - Standard library features
- `serde` - Serialization support
- `crypto` - Cryptographic primitives (P-256 ECDSA, SHA-256, HMAC)
- `sql` - SQLite integration for mobile
- `postgres` - PostgreSQL integration for server
- `base64` - Base64 encoding
- `postcard` - Efficient binary serialization (BLE protocol)
- `defmt` - Embedded debugging and logging
- `geo` - Geographic/geometric types for pathfinding
- `chrono` - Date and time handling
- `ts-rs` - TypeScript type generation (compile-time)

::: tip
See [Feature Flags Analysis](./shared/feature-flags-analysis.md) for a comprehensive analysis of all feature flags, their usage, and dependencies.
:::

## Key Modules

### BLE Protocol
Defines the Bluetooth Low Energy communication protocol used between mobile apps and beacons:
- Device discovery and capabilities
- Challenge-response authentication
- Access control messages
- Postcard binary serialization

### Schemas
Core data structures shared across components:
- `Entity` - Buildings (malls, hospitals, airports)
- `Area` - Polygonal zones within entities
- `Beacon` - BLE devices for positioning/access
- `Merchant` - Stores, restaurants, facilities
- `Connection` - Inter-area links (elevators, stairs)
- `Account` - User authentication data

### Crypto
Cryptographic utilities:
- P-256 ECDSA signature generation/verification
- Nonce management for replay attack prevention
- Proof structures for access control

### Pathfinding
Advanced pathfinding algorithms:
- Inner-area routing (A* within polygons)
- Inter-area routing (Dijkstra between areas)
- Triangulation for non-Manhattan geometries
- Visibility graph construction

## Usage Examples

### Firmware (Embedded)
```toml
[dependencies]
navign-shared = { path = "../shared", default-features = false, features = [
  "heapless",
  "serde",
  "crypto",
  "postcard",
  "defmt"
] }
```

### Server
```toml
[dependencies]
navign-shared = { path = "../shared", default-features = false, features = [
  "std",
  "serde",
  "geo",
  "postgres",
  "sql",
  "crypto"
] }
```

### Mobile (Tauri)
```toml
[dependencies]
navign-shared = { path = "../../shared", features = [
  "std",
  "serde",
  "sql",
  "crypto"
] }
```

## TypeScript Generation

The library can generate TypeScript definitions for mobile app type safety:

```bash
cd shared
cargo run --bin gen-ts-schema --features ts-rs
```

This generates `.d.ts` files in `mobile/src/schema/generated/` that match the Rust types exactly.

## Testing

Test different feature combinations:

```bash
# Default features
cargo test

# Embedded (heapless)
cargo test --features heapless,serde,crypto,postcard --no-default-features

# PostgreSQL
cargo test --features postgres,sql,serde,crypto

# Geometric types
cargo test --features geo,alloc,serde
```

## Resources

- [Feature Flags Analysis](./shared/feature-flags-analysis.md) - Detailed analysis of all features
- [Pathfinding Module](../../development/pathfinding.md) - Pathfinding algorithms guide
- [BLE Protocol Spec](../../protocols/ble.md) - BLE communication protocol
