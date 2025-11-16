# Testing Documentation

This section provides comprehensive testing guides and strategies for all Navign components.

## Overview

The Navign project employs multiple testing strategies across different components, from unit tests to integration tests and hardware-in-the-loop testing. Testing is critical to ensure reliability, security, and performance of the indoor navigation system.

## Testing Strategies by Component

### Firmware Testing
**[Firmware Testing Guide](./firmware-testing.md)** - Comprehensive testing for ESP32-C3 beacon firmware

The firmware testing guide covers both mock-based tests (fast, runs on host) and QEMU simulation tests for the ESP32-C3 BLE beacons.

**Coverage:**
- Nonce management (6 tests, 95%+ coverage)
- Cryptography - P-256 ECDSA (8 tests, 90%+ coverage)
- Rate limiting (8 tests, 90%+ coverage)
- GPIO/Peripherals (3 tests, 60%+ coverage)

**Test Methods:**
- Mock-based unit tests using `#[cfg(test)]` and `std` feature
- QEMU simulation for full integration testing
- Hardware-in-the-loop testing (planned)

---

## Testing Infrastructure

### Unit Tests

Unit tests are co-located with source code and run using `cargo test` or component-specific test commands.

**Rust Components:**
```bash
# Server
cd server && cargo test

# Shared library (multiple feature combinations)
cd shared && cargo test
cd shared && cargo test --features heapless --no-default-features
cd shared && cargo test --features crypto,serde,mongodb

# Firmware (mock tests)
cd firmware && just test-firmware-mocks

# Admin components
cd admin/orchestrator && cargo test
cd admin/maintenance && cargo test

# Robot components
cd robot/scheduler && cargo test
cd robot/serial && cargo test
cd robot/network && cargo test
```

**TypeScript Components:**
```bash
# Mobile app
cd mobile && pnpm test

# Documentation site
cd docs && pnpm test
```

**Python Components:**
```bash
# Robot vision
cd robot/vision && uv run pytest

# Robot audio
cd robot/audio && uv run pytest

# Admin plot
cd admin/plot && uv run pytest
```

---

### Integration Tests

Integration tests verify interactions between components:

- **BLE Protocol Tests** - Mobile ↔ Beacon communication
- **API Tests** - Mobile ↔ Server REST endpoints
- **gRPC Tests** - Orchestrator ↔ Tower communication
- **Zenoh Tests** - Robot component messaging
- **Database Tests** - Server ↔ MongoDB/PostgreSQL

---

### Simulation Testing

**QEMU Firmware Simulation:**
```bash
cd firmware && just test-firmware-qemu
```

**Robot Simulation:**
- Gazebo simulation (planned)
- Virtual beacon network (planned)

---

### End-to-End Tests

End-to-end tests verify complete user workflows:

- Navigation flow (planned)
- Access control flow (planned)
- Robot delivery flow (planned)
- Firmware OTA flow (planned)

---

## Test Coverage Goals

| Component | Current Coverage | Goal |
|-----------|-----------------|------|
| Server | 80%+ | 90%+ |
| Firmware | 80%+ | 90%+ |
| Mobile | 70%+ | 85%+ |
| Shared | 90%+ | 95%+ |
| Robot Components | 60%+ | 85%+ |
| Admin Components | 70%+ | 85%+ |

---

## Continuous Integration

All tests are automated via GitHub Actions CI/CD pipeline:

```bash
# Run CI checks locally
just ci-server
just ci-firmware
just ci-mobile
just ci-robot-upper
just ci-robot-lower
```

See `.github/workflows/ci.yml` for complete CI configuration.

---

## Best Practices

1. **Test Naming** - Use descriptive names: `test_nonce_expiration_after_5_seconds`
2. **Mock External Dependencies** - Database, network, hardware peripherals
3. **Deterministic Tests** - No random failures, consistent results
4. **Fast Feedback** - Unit tests should run in < 1 second
5. **Integration Tests** - Use Docker for database dependencies
6. **Feature Flag Testing** - Test all feature combinations for `shared/`

---

## Adding New Tests

### Rust Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = my_function(input);

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### TypeScript Test Template

```typescript
import { describe, it, expect } from 'vitest'
import { myFunction } from './myModule'

describe('myFunction', () => {
  it('should handle basic case', () => {
    // Arrange
    const input = 'test'

    // Act
    const result = myFunction(input)

    // Assert
    expect(result).toBe('expected')
  })
})
```

---

## See Also

- [Development Guides](../development/) - Development workflow and conventions
- [Pipelines](../pipelines/) - End-to-end data flow testing
- [Components](../components/) - Component-specific testing notes
