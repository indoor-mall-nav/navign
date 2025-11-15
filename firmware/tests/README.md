# Firmware Testing Guide

This directory contains tests for the ESP32-C3 firmware. See `/firmware/TESTING.md` for comprehensive documentation.

## Quick Start

### Mock-based Tests (Fastest)
Run on host machine without hardware or simulators:

```bash
# Run all mock tests
just test-firmware-mocks

# Or run individual test suites
cargo test --test nonce_tests --features std
cargo test --test crypto_tests --features std
cargo test --test rate_limit_tests --features std
```

### QEMU Simulation Tests
Requires QEMU installation (see TESTING.md):

```bash
# Run QEMU tests
just test-firmware-qemu

# Or run directly
./tests/qemu_runner.sh
```

### All Tests
```bash
just test-firmware-all
```

## Test Structure

```
tests/
├── README.md           # This file
├── qemu_runner.sh      # QEMU test automation script
├── mocks/              # Mock implementations
│   ├── mod.rs
│   ├── rng.rs          # Mock RNG (deterministic)
│   ├── storage.rs      # Mock flash storage
│   └── gpio.rs         # Mock GPIO/relay
├── nonce_tests.rs      # Nonce management tests
├── crypto_tests.rs     # Cryptography tests
└── rate_limit_tests.rs # Rate limiting tests
```

## CI/CD

Tests run automatically in GitHub Actions:
- **Mock tests**: Run on every PR/push
- **QEMU tests**: Run on firmware changes
- **Hardware build**: Verify firmware compiles for real hardware

See `.github/workflows/firmware-tests.yml` for details.

## Coverage

Current test coverage:

| Component | Tests | Coverage Goal |
|-----------|-------|---------------|
| Nonce Management | 6 | 95%+ |
| Cryptography (P-256 ECDSA) | 8 | 90%+ |
| Rate Limiting | 8 | 90%+ |
| BLE Protocol | 0 | 80%+ (TODO) |
| Storage (eFuse) | 0 | 85%+ (TODO) |
| GPIO/Peripherals | 3 | 60%+ |

## Adding New Tests

### Mock-based Test

Create `tests/my_feature_tests.rs`:

```rust
#![cfg(test)]

mod mocks;
use mocks::storage::MockStorage;

#[test]
fn test_my_feature() {
    let storage = MockStorage::new();
    // ... test code
}
```

Run with:
```bash
cargo test --test my_feature_tests --features std
```

### QEMU Test

QEMU tests validate the firmware boots and runs correctly. The `qemu_runner.sh` script:

1. Checks firmware binary exists
2. Starts QEMU with ESP32-C3 machine
3. Monitors output for boot success
4. Checks for panics/errors
5. Reports test results

To add custom QEMU validation, edit `qemu_runner.sh`.

## Troubleshooting

### Mock tests fail
- Ensure you're using `--features std` flag
- Check Rust toolchain is up to date

### QEMU not found
```bash
# Install ESP32-C3 QEMU
git clone --depth 1 --branch esp-develop-9.0.0 https://github.com/espressif/qemu.git
cd qemu
./configure --target-list=riscv32-softmmu --enable-gcrypt --enable-slirp
make -j$(nproc)
sudo make install
```

### Firmware build fails
```bash
# Install espup
cargo install espup
espup install
source $HOME/export-esp.sh

# Build
cargo build --release
```

## Resources

- [Full Testing Documentation](../TESTING.md)
- [ESP32-C3 QEMU Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/api-guides/tools/qemu.html)
- [Wokwi Simulator](https://wokwi.com/esp32)
