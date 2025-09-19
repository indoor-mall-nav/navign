# NAVIGN Beacon C3

A secure Bluetooth Low Energy (BLE) beacon system for indoor mall navigation and access control, designed for ESP32-C3 microcontrollers.

## Overview

This project implements a smart beacon system that provides secure access control and navigation services in indoor mall environments. The beacon advertises as "NAVIGN-BEACON" and uses cryptographic protocols to authenticate and authorize access requests from mobile clients.

## Features

### Core Functionality
- **BLE Advertising**: Broadcasts beacon presence with service UUID `0x1809`
- **Secure Authentication**: Uses P-256 ECDSA cryptographic signatures for secure access control
- **Device Capabilities**: Supports multiple device types and capabilities
- **WiFi Connectivity**: Integrated WiFi support for network communication
- **Hardware Integration**: Controls physical actuators (relays, gates, sensors)

### Device Types
- **Merchant**: Commercial establishment access points
- **Pathway**: Navigation waypoints
- **Connection**: Junction points between areas
- **Turnstile**: Access control gates

### Device Capabilities
- **UnlockGate**: Physical gate/door unlocking capability
- **EnvironmentalData**: Environmental sensor data collection
- **RssiCalibration**: Signal strength calibration for positioning

### Security Features
- **Nonce-based Authentication**: Prevents replay attacks
- **Challenge-Response Protocol**: Secure proof verification
- **Counter-based Protection**: Sequential request validation
- **Efuse Private Key Storage**: Hardware-secured key storage
- **Attempt Limiting**: Maximum 5 unlock attempts for security

## Hardware Requirements

### Supported Chips
- ESP32-C3 (primary target)
- ESP32, ESP32-S3, ESP32-C2, ESP32-C6, ESP32-H2

### GPIO Configuration
- **GPIO6**: Human body sensor (input)
- **GPIO7**: Trigger output
- **GPIO8**: LED indicator
- **GPIO9**: Button input (with pull-down)

### Memory Requirements
- **Heap Size**: 192KB allocated for dynamic memory
- **Flash**: Sufficient space for firmware and configuration
- **Efuse**: Block KEY0 used for private key storage

## Protocol Specification

### BLE Service
- **Service UUID**: `134b1d88-cd91-8134-3e94-5c4052743845`
- **Characteristic UUID**: `99d92823-9e38-72ff-6cf1-d2d593316af8`
- **Operations**: Read, Write, Notify

### Message Types
| Type | ID | Description |
|------|----|-----------| 
| Device Request | 0x01 | Request device information |
| Device Response | 0x02 | Device type, capabilities, and ID |
| Nonce Request | 0x03 | Request cryptographic nonce |
| Nonce Response | 0x04 | 16-byte random nonce |
| Unlock Request | 0x05 | Cryptographic proof for access |
| Unlock Response | 0x06 | Success/failure with reason |

### Packet Structure
- **Max Packet Size**: 256 bytes
- **Device ID**: 24-byte MongoDB ObjectId format
- **Nonce**: 16-byte random value
- **Signature**: 64-byte P-256 ECDSA signature
- **Timestamp**: 8-byte Unix timestamp
- **Counter**: 8-byte sequential counter

## Installation & Setup

### Prerequisites
1. **Rust Toolchain**: Stable channel with `rust-src` component
2. **Target**: `riscv32imc-unknown-none-elf`
3. **ESP-IDF**: Compatible version for ESP32-C3
4. **Hardware**: ESP32-C3 development board

### Building
```bash
# Clone the repository
git clone <repository-url>
cd beacon-c3

# Build the project
cargo build --release

# Flash to device
cargo run --release
```

### Configuration

#### WiFi Setup
Update the WiFi credentials in `main.rs`:
```rust
let wifi_config = Configuration::Client(esp_wifi::wifi::ClientConfiguration {
    ssid: "your_ssid".into(),
    password: "your_password".into(),
    auth_method: AuthMethod::WPAWPA2Personal,
    ..Default::default()
});
```

#### Device Configuration
Configure device properties:
```rust
let device_id = b"your_24_byte_device_id_here";
let device_type = DeviceType::Merchant; // or Pathway, Connection, Turnstile
let mut capabilities = Vec::<DeviceCapability, 3>::new();
capabilities.push(DeviceCapability::UnlockGate).unwrap();
```

#### Private Key Setup
Ensure the private key is programmed into efuse BLOCK_KEY0 before deployment.

## Architecture

### Module Structure
```
src/bin/
├── main.rs              # Main application entry point
├── ble/                 # Bluetooth Low Energy implementation
│   ├── mod.rs          # Message types and protocol definitions
│   ├── manager.rs      # BLE connection management
│   └── protocol.rs     # Protocol handler implementation
├── crypto/             # Cryptographic operations
│   ├── mod.rs          # Crypto module exports
│   ├── challenge.rs    # Challenge generation and management
│   ├── error.rs        # Crypto error types
│   ├── nonce.rs        # Nonce generation and validation
│   └── proof.rs        # Digital signature proof handling
├── execute/            # Beacon state management
│   └── mod.rs          # BeaconState and execution logic
├── internet/           # Network communication
│   └── mod.rs          # Internet connectivity features
├── shared/             # Shared types and constants
│   ├── mod.rs          # Device types and capabilities
│   └── constants.rs    # Protocol constants and packet sizes
└── storage/            # Persistent storage
    ├── mod.rs          # Storage module exports
    ├── nonce_manager.rs # Nonce storage and management
    └── private_key.rs   # Private key handling
```

### Security Model
1. **Device Registration**: Each beacon has a unique 24-byte device ID
2. **Key Management**: Private keys stored in hardware efuse
3. **Authentication Flow**:
   - Client requests device information
   - Client requests nonce for challenge
   - Client creates cryptographic proof using challenge
   - Beacon validates proof and grants/denies access
4. **Replay Protection**: Nonces and counters prevent replay attacks
5. **Rate Limiting**: Maximum attempts per session

## Usage

### Client Integration
Clients should implement the following flow:
1. Scan for "NAVIGN-BEACON" advertisements
2. Connect to BLE service
3. Request device information (capabilities, type, ID)
4. Request nonce for authentication challenge
5. Generate cryptographic proof with user credentials
6. Send unlock request with proof
7. Handle response (success/failure)

### Physical Integration
- Connect human body sensor to GPIO6 for presence detection
- Connect relay/actuator to GPIO7 for physical access control
- Connect status LED to GPIO8 for visual feedback
- Connect button to GPIO9 for manual operations

## Development

### Building for Different Targets
The project supports multiple ESP32 variants. Update `Cargo.toml` dependencies for your target chip.

### Debugging
- Serial output via `esp-println` for debugging
- LED status indicators for visual feedback
- Structured error handling with specific error types

### Testing
- Unit tests for cryptographic operations
- Integration tests for BLE protocol
- Hardware-in-the-loop testing recommended

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement changes with tests
4. Submit a pull request

## Security Considerations

- **Private Key Protection**: Never expose private keys in code
- **Nonce Uniqueness**: Ensure nonces are cryptographically random
- **Timestamp Validation**: Implement reasonable time windows for requests
- **Counter Synchronization**: Maintain counter state across reboots
- **Physical Security**: Secure hardware deployment environment

## Troubleshooting

### Common Issues
- **BLE Connection Failures**: Check advertising parameters and service UUIDs
- **WiFi Connection Issues**: Verify credentials and network availability
- **Crypto Errors**: Ensure private key is properly configured in efuse
- **Memory Issues**: Monitor heap usage, increase allocation if needed

### Debug Output
Enable debug logging by setting appropriate log levels in the environment.
