# Beacon Components

This directory contains components for beacon management and configuration.

## BluFi Provisioning

**Status:** Placeholder (Not Yet Implemented)

The `BluFiProvisioning.vue` component is a placeholder for future BluFi (Bluetooth + WiFi) provisioning functionality. It will allow users to configure WiFi credentials on ESP32 beacons over Bluetooth.

### Architecture

Following the Navign architecture pattern:

- **Rust Core** (`mobile/src-tauri/src/blufi/`): All business logic
  - BLE communication
  - BluFi protocol implementation
  - WiFi scanning through beacon
  - Credential provisioning
  - Orchestrator configuration
  - Connection verification

- **TypeScript API** (`mobile/src/lib/api/blufi.ts`): Minimal wrappers
  - Type-safe wrappers for Rust commands
  - No business logic

- **Vue UI** (`mobile/src/components/beacon/BluFiProvisioning.vue`): UI only
  - User interface
  - State management for UI
  - Calls Rust commands via TypeScript API

### Workflow

1. **Scan Beacons** - Discover BLE beacons in provisioning mode
2. **Connect** - Establish BLE connection to selected beacon
3. **Scan WiFi** - Scan for WiFi networks through the beacon
4. **Configure** - Send WiFi credentials + orchestrator settings
5. **Provision** - Beacon connects to WiFi and orchestrator
6. **Verify** - Confirm connection and display status

### Implementation Status

- [x] Rust BluFi module structure (`mobile/src-tauri/src/blufi/mod.rs`)
- [x] Tauri commands (`mobile/src-tauri/src/blufi/commands.rs`)
- [x] Shared schemas (`shared/src/schema/blufi.rs`)
- [x] TypeScript API wrapper (`mobile/src/lib/api/blufi.ts`)
- [x] Placeholder UI component (`BluFiProvisioning.vue`)
- [ ] BluFi protocol implementation
- [ ] BLE communication handlers
- [ ] WiFi scanning implementation
- [ ] Credential provisioning
- [ ] Orchestrator configuration
- [ ] Full UI implementation

### Future Work

When implementing BluFi:

1. **Rust Implementation**:
   - Implement Espressif BluFi protocol
   - Add BLE service discovery
   - Implement encryption/security layer
   - Add WiFi scanning via beacon
   - Implement credential transmission
   - Add connection verification

2. **UI Implementation**:
   - Beacon scanning list with signal strength
   - Beacon connection status indicator
   - WiFi network selection with security indicators
   - WiFi credential input form
   - Orchestrator configuration fields
   - Beacon metadata input (name, location)
   - Provisioning progress stepper
   - Success/failure result display

3. **Testing**:
   - Unit tests for Rust BluFi module
   - Integration tests with ESP32
   - UI component tests
   - End-to-end provisioning tests

### Related Files

- `shared/src/schema/blufi.rs` - BluFi type definitions
- `mobile/src-tauri/src/blufi/` - Rust BluFi implementation
- `mobile/src/lib/api/blufi.ts` - TypeScript API
- `mobile/src/components/beacon/BluFiProvisioning.vue` - UI component

### References

- [Espressif BluFi Documentation](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/blufi.html)
- [ESP32 BluFi Protocol](https://github.com/espressif/esp-idf/tree/master/examples/bluetooth/blufi)
