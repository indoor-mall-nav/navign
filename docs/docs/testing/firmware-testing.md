# Firmware Testing Guide

This document provides comprehensive guidance for testing the ESP32-C3 firmware for the Navign indoor navigation system.

**⚠️ Current Status:** Mock-based unit tests are disabled pending library extraction from binary-only crate. QEMU simulation tests are available for manual execution.

## Table of Contents

1. [Current Limitations](#current-limitations)
2. [Testing Approaches](#testing-approaches)
3. [QEMU Simulation](#qemu-simulation)
4. [Wokwi Simulation](#wokwi-simulation)
5. [Mock-based Unit Tests](#mock-based-unit-tests)
6. [Hardware-in-the-Loop Testing](#hardware-in-the-loop-testing)
7. [CI/CD Integration](#cicd-integration)

---

## Current Limitations

**Mock-Based Tests Non-Functional (as of 2025-01-14):**
- Firmware is structured as binary-only crate (`src/bin/main.rs` only)
- Integration tests require library target (`src/lib.rs`)
- ESP-specific dependencies don't compile for x86_64 host target
- Test code in `tests/` directory is complete but can't execute

**Required Fixes:**
1. Extract testable modules to `src/lib.rs`
2. Make ESP dependencies conditional: `[target.'cfg(target_arch = "riscv32")'.dependencies]`
3. Re-enable tests in `justfile` ci-firmware target

**Currently Working:**
- QEMU simulation (manual execution only, requires setup)
- Hardware-in-the-loop testing (requires physical device)
- Code compilation checks (runs in CI)

---

## Testing Approaches

The firmware can be tested using multiple approaches, each with different tradeoffs:

| Approach | Speed | Accuracy | Setup | Use Case |
|----------|-------|----------|-------|----------|
| Mock Unit Tests | ⚡ Fast | Medium | Easy | Development, CI/CD (DISABLED) |
| QEMU | Fast | High | Medium | Security, crypto testing (MANUAL ONLY) |
| Wokwi | Medium | High | Easy | BLE, peripheral testing |
| Real Hardware | Slow | Perfect | Hard | Integration, final validation |

**Recommendation:** Use QEMU for security features validation (manual), Wokwi for BLE protocol testing, and real hardware for final acceptance testing.

---

## QEMU Simulation

QEMU provides the most comprehensive ESP32-C3 emulation with full security feature support.

### Installation

```bash
# Install Espressif's QEMU fork
git clone https://github.com/espressif/qemu.git
cd qemu
git checkout esp-develop-9.0.0
./configure --target-list=riscv32-softmmu --enable-gcrypt --enable-slirp --disable-capstone --disable-vnc --disable-sdl --disable-gtk
make -j$(nproc)
sudo make install
```

### Running Firmware in QEMU

```bash
# Build firmware
cd firmware
cargo build --release

# Convert to binary
esptool.py --chip esp32c3 elf2image --output build/firmware.bin target/riscv32imc-esp-espidf/release/navign-firmware

# Run in QEMU
idf.py qemu monitor
```

### Automated Testing with QEMU

Create `firmware/tests/qemu_integration_test.sh`:

```bash
#!/bin/bash
set -e

# Build firmware
cargo build --release

# Start QEMU in background
qemu-system-riscv32 \
  -nographic \
  -machine esp32c3 \
  -drive file=build/firmware.bin,if=mtd,format=raw \
  -serial mon:stdio \
  > qemu_output.log 2>&1 &

QEMU_PID=$!

# Wait for boot
sleep 5

# Send test commands via serial
echo "Running crypto tests..."
# TODO: Implement serial command protocol

# Check output for test results
if grep -q "All tests passed" qemu_output.log; then
  echo "✓ QEMU tests passed"
  kill $QEMU_PID
  exit 0
else
  echo "✗ QEMU tests failed"
  cat qemu_output.log
  kill $QEMU_PID
  exit 1
fi
```

### Testing Security Features

QEMU supports testing eFuse operations, secure boot, and flash encryption without modifying real hardware:

```rust
// firmware/tests/qemu_security_test.rs
#[cfg(test)]
mod qemu_security_tests {
    #[test]
    fn test_efuse_key_read() {
        // Test reading eFuse key in QEMU
        // Key can be set via QEMU command line
    }

    #[test]
    fn test_flash_encryption() {
        // Test flash encryption behavior
    }
}
```

---

## Wokwi Simulation

Wokwi provides an easy-to-use web-based simulator with excellent BLE support.

### Setup

1. Create `firmware/wokwi.toml`:

```toml
[wokwi]
version = 1
elf = "target/riscv32imc-esp-espidf/release/navign-firmware"
firmware = "target/riscv32imc-esp-espidf/release/navign-firmware.bin"

[[wokwi.parts]]
type = "wokwi-esp32-c3-devkit-m1"
id = "esp"
```

2. Create `firmware/diagram.json`:

```json
{
  "version": 1,
  "author": "Navign Team",
  "editor": "wokwi",
  "parts": [
    { "type": "wokwi-esp32-c3-devkit-m1", "id": "esp", "top": 0, "left": 0, "attrs": {} },
    { "type": "wokwi-relay-module", "id": "relay1", "top": 0, "left": 100, "attrs": {} },
    { "type": "wokwi-dht11", "id": "dht1", "top": 100, "left": 0, "attrs": {} },
    { "type": "wokwi-led", "id": "led1", "top": 100, "left": 100, "attrs": { "color": "blue" } }
  ],
  "connections": [
    [ "esp:GPIO7", "relay1:IN", "green", [ "v0" ] ],
    [ "esp:GPIO4", "dht1:SDA", "yellow", [ "v0" ] ],
    [ "esp:GPIO8", "led1:A", "red", [ "v0" ] ],
    [ "relay1:GND", "esp:GND", "black", [ "v0" ] ],
    [ "dht1:GND", "esp:GND", "black", [ "v0" ] ],
    [ "led1:C", "esp:GND", "black", [ "v0" ] ]
  ]
}
```

### Running in Wokwi

```bash
# Install Wokwi CLI
npm install -g wokwi-cli

# Or use VS Code extension
code --install-extension wokwi.wokwi-vscode

# Build and simulate
cargo build --release
wokwi-cli --diagram diagram.json --elf target/riscv32imc-esp-espidf/release/navign-firmware
```

### BLE Testing with Wokwi

Wokwi supports BLE simulation. You can test the BLE protocol using the virtual BLE scanner:

1. Start Wokwi simulation
2. Open the "Serial Monitor"
3. Use Wokwi's virtual BLE scanner to connect
4. Send GATT characteristic writes to test unlock protocol

---

## Mock-based Unit Tests

For fast, deterministic testing without hardware dependencies.

### Architecture

```
firmware/tests/
├── unit-tests.rs          # Entry point (already exists, empty)
├── mocks/
│   ├── mod.rs             # Mock module exports
│   ├── gpio.rs            # Mock GPIO
│   ├── ble.rs             # Mock BLE stack
│   ├── storage.rs         # Mock flash storage
│   └── rng.rs             # Mock RNG (deterministic)
└── crypto_tests.rs        # Crypto unit tests
└── ble_protocol_tests.rs  # BLE protocol tests
└── nonce_tests.rs         # Nonce management tests
└── rate_limit_tests.rs    # Rate limiting tests
```

### Example Mock Implementation

Create `firmware/tests/mocks/mod.rs`:

```rust
pub mod gpio;
pub mod ble;
pub mod storage;
pub mod rng;
```

Create `firmware/tests/mocks/rng.rs`:

```rust
/// Deterministic RNG for testing
pub struct MockRng {
    seed: u64,
}

impl MockRng {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn read(&mut self) -> u32 {
        // Simple LCG for deterministic randomness
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.seed / 65536) as u32
    }

    pub fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let val = self.read().to_le_bytes();
            chunk.copy_from_slice(&val[..chunk.len()]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut rng1 = MockRng::new(42);
        let mut rng2 = MockRng::new(42);

        assert_eq!(rng1.read(), rng2.read());
        assert_eq!(rng1.read(), rng2.read());
    }
}
```

Create `firmware/tests/mocks/storage.rs`:

```rust
use heapless::Vec;

/// Mock flash storage backed by RAM
pub struct MockStorage {
    efuse_data: [u8; 32],
    nonces: Vec<([u8; 16], u64), 16>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            efuse_data: [0u8; 32],
            nonces: Vec::new(),
        }
    }

    pub fn set_private_key(&mut self, key: [u8; 32]) {
        self.efuse_data = key;
    }

    pub fn read_private_key(&self) -> [u8; 32] {
        self.efuse_data
    }

    pub fn store_nonce(&mut self, nonce: [u8; 16], timestamp: u64) -> Result<(), ()> {
        if self.nonces.len() >= 16 {
            self.nonces.remove(0);
        }
        self.nonces.push((nonce, timestamp)).map_err(|_| ())
    }

    pub fn check_nonce_used(&self, nonce: &[u8; 16]) -> bool {
        self.nonces.iter().any(|(n, _)| n == nonce)
    }

    pub fn cleanup_expired_nonces(&mut self, current_time: u64, ttl: u64) {
        self.nonces.retain(|(_, timestamp)| current_time - timestamp < ttl);
    }
}
```

### Example Test Cases

Create `firmware/tests/nonce_tests.rs`:

```rust
#![cfg(test)]

use navign_shared::ble::Nonce;

mod mocks;
use mocks::storage::MockStorage;

#[test]
fn test_nonce_generation() {
    let mut rng = mocks::rng::MockRng::new(12345);
    let mut nonce_bytes = [0u8; 16];
    rng.fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::new(nonce_bytes);
    assert_eq!(nonce.bytes(), &nonce_bytes);
}

#[test]
fn test_nonce_replay_prevention() {
    let mut storage = MockStorage::new();
    let nonce = [42u8; 16];

    // Store nonce
    storage.store_nonce(nonce, 1000).unwrap();

    // Check it's marked as used
    assert!(storage.check_nonce_used(&nonce));

    // Different nonce should not be marked
    let different_nonce = [43u8; 16];
    assert!(!storage.check_nonce_used(&different_nonce));
}

#[test]
fn test_nonce_expiration() {
    let mut storage = MockStorage::new();

    // Store old nonces
    storage.store_nonce([1u8; 16], 1000).unwrap();
    storage.store_nonce([2u8; 16], 2000).unwrap();
    storage.store_nonce([3u8; 16], 5000).unwrap();

    // Clean up nonces older than 3 seconds
    storage.cleanup_expired_nonces(6000, 3000);

    // Old nonces should be removed
    assert!(!storage.check_nonce_used(&[1u8; 16]));
    assert!(!storage.check_nonce_used(&[2u8; 16]));

    // Recent nonce should still exist
    assert!(storage.check_nonce_used(&[3u8; 16]));
}

#[test]
fn test_nonce_buffer_overflow() {
    let mut storage = MockStorage::new();

    // Fill buffer to capacity (16 nonces)
    for i in 0..16 {
        let mut nonce = [0u8; 16];
        nonce[0] = i;
        storage.store_nonce(nonce, i as u64).unwrap();
    }

    // Add one more - should evict oldest
    let new_nonce = [99u8; 16];
    storage.store_nonce(new_nonce, 100).unwrap();

    // First nonce should be evicted
    assert!(!storage.check_nonce_used(&[0u8; 16]));

    // New nonce should be present
    assert!(storage.check_nonce_used(&new_nonce));
}
```

Create `firmware/tests/crypto_tests.rs`:

```rust
#![cfg(test)]

use p256::ecdsa::{SigningKey, VerifyingKey, Signature, signature::{Signer, Verifier}};
use sha2::{Sha256, Digest};

#[test]
fn test_signature_verification() {
    // Generate test keys
    let private_key = SigningKey::from_bytes(&[42u8; 32].into()).unwrap();
    let public_key = VerifyingKey::from(&private_key);

    // Sign a nonce
    let nonce = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let mut hasher = Sha256::new();
    hasher.update(&nonce);
    let hash = hasher.finalize();

    let signature: Signature = private_key.sign(&hash);

    // Verify signature
    assert!(public_key.verify(&hash, &signature).is_ok());
}

#[test]
fn test_invalid_signature_rejection() {
    let private_key1 = SigningKey::from_bytes(&[42u8; 32].into()).unwrap();
    let private_key2 = SigningKey::from_bytes(&[43u8; 32].into()).unwrap();
    let public_key1 = VerifyingKey::from(&private_key1);

    let nonce = [1u8; 16];
    let mut hasher = Sha256::new();
    hasher.update(&nonce);
    let hash = hasher.finalize();

    // Sign with key2
    let signature: Signature = private_key2.sign(&hash);

    // Verify with key1 should fail
    assert!(public_key1.verify(&hash, &signature).is_err());
}

#[test]
fn test_unlock_proof_format() {
    // Test the proof includes: nonce || device_id
    let nonce = [42u8; 16];
    let device_id = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24];

    let mut proof_data = [0u8; 40];
    proof_data[..16].copy_from_slice(&nonce);
    proof_data[16..].copy_from_slice(&device_id);

    // Hash and sign
    let private_key = SigningKey::from_bytes(&[99u8; 32].into()).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&proof_data);
    let hash = hasher.finalize();
    let signature: Signature = private_key.sign(&hash);

    // Signature should be 64 bytes (r + s)
    assert_eq!(signature.to_bytes().len(), 64);
}
```

Create `firmware/tests/rate_limit_tests.rs`:

```rust
#![cfg(test)]

mod mocks;

/// Rate limiter state for testing
struct RateLimiter {
    attempts: heapless::Vec<u64, 5>,
    max_attempts: usize,
    window_ms: u64,
}

impl RateLimiter {
    fn new(max_attempts: usize, window_ms: u64) -> Self {
        Self {
            attempts: heapless::Vec::new(),
            max_attempts,
            window_ms,
        }
    }

    fn check_and_record(&mut self, current_time: u64) -> bool {
        // Remove old attempts
        self.attempts.retain(|&t| current_time - t < self.window_ms);

        if self.attempts.len() >= self.max_attempts {
            return false; // Rate limited
        }

        self.attempts.push(current_time).ok();
        true
    }
}

#[test]
fn test_rate_limit_allows_within_limit() {
    let mut limiter = RateLimiter::new(5, 5000);

    // 5 attempts within window should succeed
    for i in 0..5 {
        assert!(limiter.check_and_record(1000 + i * 100));
    }
}

#[test]
fn test_rate_limit_blocks_excess() {
    let mut limiter = RateLimiter::new(5, 5000);

    // Fill up the limit
    for i in 0..5 {
        limiter.check_and_record(1000 + i);
    }

    // 6th attempt should be blocked
    assert!(!limiter.check_and_record(1005));
}

#[test]
fn test_rate_limit_resets_after_window() {
    let mut limiter = RateLimiter::new(5, 5000);

    // Fill limit
    for i in 0..5 {
        limiter.check_and_record(1000 + i);
    }

    // Wait beyond window
    assert!(limiter.check_and_record(7000));
}

#[test]
fn test_rate_limit_sliding_window() {
    let mut limiter = RateLimiter::new(5, 5000);

    // Attempts at: 1000, 2000, 3000, 4000, 5000
    for i in 0..5 {
        limiter.check_and_record(1000 + i * 1000);
    }

    // At time 6100, oldest (1000) should be expired
    assert!(limiter.check_and_record(6100));
}
```

### Running Mock Tests

```bash
cd firmware
cargo test --tests
```

---

## Hardware-in-the-Loop Testing

For final validation with real ESP32-C3 hardware.

### Test Harness Setup

You'll need:
- ESP32-C3 DevKit
- USB-to-Serial adapter for logging
- Another ESP32 or computer with BLE capability for testing unlock
- Oscilloscope/logic analyzer (optional, for timing verification)

### Automated HIL Testing

Create `firmware/tests/hil/unlock_test.py`:

```python
#!/usr/bin/env python3
"""Hardware-in-the-loop test for unlock functionality"""

import asyncio
from bleak import BleakScanner, BleakClient
import struct
import hashlib
from ecdsa import SigningKey, NIST256p

CHARACTERISTIC_UUID = "99d92823-9e38-72ff-6cf1-d2d593316af8"

async def test_unlock_flow():
    # Scan for beacon
    print("Scanning for Navign beacon...")
    device = await BleakScanner.find_device_by_name("Navign Beacon")

    if not device:
        print("✗ Beacon not found")
        return False

    print(f"✓ Found beacon: {device.address}")

    async with BleakClient(device) as client:
        # Request nonce
        nonce_request = bytes([0x02])  # NonceRequest message type
        await client.write_gatt_char(CHARACTERISTIC_UUID, nonce_request)

        # Read nonce response
        nonce_response = await client.read_gatt_char(CHARACTERISTIC_UUID)
        nonce = nonce_response[1:17]  # Extract nonce bytes

        print(f"✓ Received nonce: {nonce.hex()}")

        # Generate proof (sign nonce with user's private key)
        user_private_key = SigningKey.from_string(bytes.fromhex("0" * 64), curve=NIST256p)
        signature = user_private_key.sign(nonce, hashfunc=hashlib.sha256)

        # Send unlock request
        unlock_request = bytes([0x04]) + signature
        await client.write_gatt_char(CHARACTERISTIC_UUID, unlock_request)

        # Read unlock response
        unlock_response = await client.read_gatt_char(CHARACTERISTIC_UUID)
        success = unlock_response[1] == 1

        if success:
            print("✓ Unlock successful")
            return True
        else:
            print(f"✗ Unlock failed with error: {unlock_response[2]}")
            return False

if __name__ == "__main__":
    result = asyncio.run(test_unlock_flow())
    exit(0 if result else 1)
```

Run with:
```bash
cd firmware/tests/hil
python3 unlock_test.py
```

---

## CI/CD Integration

### GitHub Actions Workflow

Add to `.github/workflows/firmware-tests.yml`:

```yaml
name: Firmware Tests

on: [push, pull_request]

jobs:
  mock-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: riscv32imc-unknown-none-elf

      - name: Run mock-based tests
        run: |
          cd firmware
          cargo test --tests

  qemu-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install QEMU
        run: |
          # Install ESP-IDF QEMU
          # (Cached for performance)

      - name: Build firmware
        run: |
          cd firmware
          cargo build --release

      - name: Run QEMU integration tests
        run: |
          cd firmware/tests
          ./qemu_integration_test.sh
```

---

## Test Coverage Goals

| Component | Target Coverage | Priority |
|-----------|----------------|----------|
| Crypto (signing, verification) | 90% | High |
| Nonce management | 95% | High |
| Rate limiting | 90% | High |
| BLE protocol | 80% | High |
| Storage (eFuse, nonce) | 85% | Medium |
| GPIO/peripherals | 60% | Low |
| OTA | 70% | Medium |

---

## Troubleshooting

### QEMU doesn't start
- Ensure ESP-IDF QEMU fork is used, not mainline QEMU
- Check RISC-V toolchain is installed

### Wokwi simulation crashes
- Verify `wokwi.toml` paths are correct
- Check firmware size < 4MB

### Mock tests fail on real hardware
- Mock tests use deterministic RNG; real hardware uses TRNG
- Timing differences may cause race conditions

---

## Resources

- [Espressif QEMU Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/api-guides/tools/qemu.html)
- [Wokwi ESP32-C3 Documentation](https://docs.wokwi.com/guides/esp32)
- [embedded-test Documentation](https://docs.rs/embedded-test/)
- [ESP32-C3 BLE Examples](https://github.com/espressif/esp-idf/tree/master/examples/bluetooth)

---

**Last Updated:** 2025-11-14
**Maintained by:** Navign Team
