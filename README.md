# Navign Beacon

A secure Bluetooth Low Energy (BLE) beacon system for indoor mall navigation and access control, designed for ESP32-C3 microcontrollers.

## Overview

**Navign** is an embedded IoT system that provides secure, cryptographically-verified access control for indoor environments like shopping malls, office buildings, and smart facilities. The beacon advertises as "NAVIGN-BEACON" and uses P-256 ECDSA cryptographic signatures to authenticate and authorize access requests from mobile clients.

This repository contains the firmware for the beacon devices that act as smart access points, providing both navigation services and physical access control capabilities.

## Features

### Core Functionality
- **BLE Advertising**: Broadcasts beacon presence with multiple service UUIDs:
  - `0x183D` - Authorization Control Service (for gate unlocking)
  - `0x1819` - Location and Navigation Service
  - `0x1821` - Indoor Positioning Service
  - `0x181A` - Environmental Sensing Service (optional)
- **Secure Authentication**: Uses P-256 ECDSA cryptographic signatures for secure access control
- **Nonce-based Challenge-Response**: Prevents replay attacks with time-windowed nonce validation
- **Device Capabilities**: Supports multiple device types and capabilities
- **Environmental Monitoring**: DHT11 temperature and humidity sensor integration
- **Hardware Integration**: Controls physical actuators (relays, servo motors, IR transmitters)

### Device Types
- **Merchant** (`0x01`): Commercial establishment access points
- **Pathway** (`0x02`): Navigation waypoints in corridors
- **Connection** (`0x03`): Junction points between different areas
- **Turnstile** (`0x04`): Access control gates and turnstiles

### Device Capabilities
- **UnlockGate** (`0x01`): Physical gate/door unlocking capability
- **EnvironmentalData** (`0x02`): Environmental sensor data collection (temperature, humidity)
- **RssiCalibration** (`0x04`): Signal strength calibration for precise indoor positioning

### Security Features
- **Nonce-based Authentication**: 16-byte random nonces prevent replay attacks
- **Challenge-Response Protocol**: Secure proof verification with server-signed challenges
- **Counter-based Protection**: Sequential request validation with monotonic counter
- **Efuse Private Key Storage**: Hardware-secured key storage in ESP32-C3 efuse BLOCK_KEY0
- **Rate Limiting**: Maximum 5 unlock attempts within 5-minute window
- **Time-Window Validation**: Nonces expire after 5 minutes
- **Dual Signature Verification**: Both server and device signatures validated

## Hardware Requirements

### Supported Chips
- **ESP32-C3** (primary target)
- ESP32, ESP32-S3, ESP32-C2, ESP32-C6, ESP32-H2

### GPIO Configuration (ESP32-C3)
```
GPIO1  - Human body sensor (PIR sensor, input)
GPIO3  - Button input (boot button)
GPIO4  - DHT11 temperature/humidity sensor (bidirectional)
GPIO7  - Relay control output (for electric locks)
GPIO8  - LED indicator output
```

### Memory Requirements
- **Heap Size**: 192KB allocated for dynamic memory
- **Flash**: ~200KB for firmware (varies with features)
- **Efuse**: BLOCK_KEY0 (256 bits) used for private key storage

### Hardware Components
- ESP32-C3-DevKitM-1 or similar board
- DHT11 temperature/humidity sensor
- PIR motion sensor (HC-SR501 or similar)
- 5V relay module (for electric lock control)
- LED indicator
- Push button (typically uses boot button)
- Power supply: 5V/1A minimum

## Software Architecture

### Module Structure

```
beacon/
├── src/bin/
│   ├── main.rs              # Main application entry point
│   ├── ble/                 # BLE protocol implementation
│   │   ├── manager.rs       # BLE connection management
│   │   ├── protocol.rs      # Protocol serialization/deserialization
│   │   └── mod.rs          # BLE message types
│   ├── crypto/              # Cryptographic primitives
│   │   ├── challenge.rs     # Challenge generation
│   │   ├── nonce.rs        # Nonce generation and validation
│   │   ├── proof.rs        # Proof verification and signing
│   │   ├── error.rs        # Crypto error types
│   │   └── mod.rs
│   ├── execute/             # Execution logic
│   │   └── mod.rs          # BeaconState and unlock methods
│   ├── storage/             # Persistent storage
│   │   ├── nonce_manager.rs # Nonce tracking and replay prevention
│   │   └── private_key.rs  # Private key management
│   ├── shared/              # Shared types and constants
│   │   ├── constants.rs    # Protocol constants
│   │   └── mod.rs          # Common types
│   └── internet/            # Network connectivity (future)
│       └── mod.rs
```

### Key Components

#### BeaconState
The central state machine that manages:
- Device authentication and proof verification
- Nonce generation and tracking
- Unlock method execution (relay, servo, IR remote)
- Human presence detection
- Rate limiting and security

#### ProofManager
Handles cryptographic operations:
- P-256 ECDSA signature verification
- Server signature validation
- Device signature generation
- Counter-based challenge validation

#### NonceManager
Manages nonce lifecycle:
- Random nonce generation using TRNG
- Replay attack prevention
- Automatic expiry of old nonces (5-minute window)
- Capacity: up to 32 concurrent nonces

#### BleProtocolHandler
Implements the binary protocol:
- Message serialization/deserialization
- Buffer management (128-byte MTU)
- Fragmentation for larger messages

## BLE Protocol Specification

### Message Types

All messages start with a 1-byte identifier:

| Message Type | ID | Length | Description |
|-------------|-----|--------|-------------|
| DEVICE_REQUEST | 0x01 | 1 byte | Request device info |
| DEVICE_RESPONSE | 0x02 | 27 bytes | Device type, capabilities, ID |
| NONCE_REQUEST | 0x03 | 1 byte | Request authentication nonce |
| NONCE_RESPONSE | 0x04 | 21 bytes | Nonce + signature tail |
| UNLOCK_REQUEST | 0x05 | 105 bytes | Signed unlock proof |
| UNLOCK_RESPONSE | 0x06 | 3 bytes | Success/failure + reason |
| DEBUG_REQUEST | 0xFF | 1+ bytes | Debug command |
| DEBUG_RESPONSE | 0xFE | 1+ bytes | Debug response |

### Authentication Flow

```
Client                          Beacon                      Server
  |                               |                            |
  |----(1) DEVICE_REQUEST-------->|                            |
  |<---(2) DEVICE_RESPONSE--------|                            |
  |    (type, capabilities, ID)   |                            |
  |                               |                            |
  |----(3) NONCE_REQUEST--------->|                            |
  |<---(4) NONCE_RESPONSE---------|                            |
  |    (nonce, signature_tail)    |                            |
  |                               |                            |
  |-----------(5) Request Challenge from Server-------------->|
  |<----------(6) Signed Challenge (nonce, timestamp)---------|
  |                               |                            |
  |----(7) UNLOCK_REQUEST-------->|                            |
  |    (proof with server sig)    |                            |
  |        |                      |                            |
  |        | Verify server sig    |                            |
  |        | Verify device sig    |                            |
  |        | Check nonce validity |                            |
  |        | Execute unlock       |                            |
  |<---(8) UNLOCK_RESPONSE--------|                            |
  |    (success/failure)          |                            |
```

### Message Format Details

#### DEVICE_RESPONSE (27 bytes)
```
[0x02][device_type][capabilities][24-byte MongoDB ObjectId]
```

#### NONCE_RESPONSE (21 bytes)
```
[0x04][16-byte nonce][4-byte signature tail]
```
- Signature tail: Last 4 bytes of device signature on nonce

#### UNLOCK_REQUEST (105 bytes)
```
[0x05][16-byte nonce][8-byte device_bytes][8-byte verify_bytes]
      [8-byte timestamp][64-byte server_signature]
```

#### UNLOCK_RESPONSE (3 bytes)
```
[0x06][success: 0x00/0x01][error_code]
```

Error codes:
- `0x01` - Invalid signature
- `0x02` - Invalid key
- `0x03` - Server public key not set
- `0x04` - Verification failed
- `0x05` - Buffer full
- `0x06` - Rate limited
- `0x07` - Replay detected

## Setup and Installation

### Prerequisites

1. **Rust Toolchain**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **ESP-RS Toolchain**
   ```bash
   cargo install espup
   espup install
   source ~/export-esp.sh
   ```

3. **Build Tools**
   ```bash
   cargo install cargo-espflash
   cargo install espmonitor
   ```

### Building the Firmware

```bash
# Clone the repository
git clone https://github.com/yourusername/beacon.git
cd beacon

# Build for ESP32-C3
cargo build --release

# Or build and flash in one step
cargo espflash flash --release --monitor
```

### Setting the Private Key

The beacon requires a 32-byte private key stored in efuse BLOCK_KEY0:

```bash
# Generate a private key (do this securely!)
openssl ecparam -name prime256v1 -genkey -noout -out private_key.pem
openssl ec -in private_key.pem -outform DER -out private_key.der

# Extract raw 32-byte key from DER format
# (You'll need to parse the DER structure to extract the key)

# Burn to efuse (WARNING: This is irreversible!)
espefuse.py -p /dev/ttyUSB0 burn_key BLOCK_KEY0 private_key.bin KEYPURPOSE_USER

# Enable read protection
espefuse.py -p /dev/ttyUSB0 burn_key_digest
```

**⚠️ WARNING**: Burning efuses is permanent and irreversible. Test thoroughly before deployment!

### Configuration

Edit `src/bin/main.rs` to configure your beacon:

```rust
// Set your device ID (24-byte hex string, typically MongoDB ObjectId)
let device_id = b"68a84b6ebdfa76608b934b0a";

// Set device type
let device_type = DeviceType::Merchant; // or Pathway, Connection, Turnstile

// Add capabilities
let mut capabilities = Vec::<DeviceCapability, 3>::new();
capabilities.push(DeviceCapability::UnlockGate).unwrap();
capabilities.push(DeviceCapability::EnvironmentalData).unwrap();

// Set server public key (65 bytes, uncompressed P-256 public key)
let server_public_key = [
    0x04, // Uncompressed format indicator
    // ... 64 bytes of public key (32 bytes X, 32 bytes Y)
];
```

## Unlock Methods

The beacon supports three unlock methods:

### 1. Relay (Default)
Simple on/off control for electric strikes and magnetic locks:

```rust
let relay = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());
let method = UnlockMethod::Relay(relay);
```

**Behavior**:
- Unlocks: Relay HIGH for 5 seconds
- Human sensor active: Extends relay HIGH
- Closes: 10 seconds after last motion or unlock

### 2. Servo Motor
For mechanical gates using servo motors:

```rust
let method = UnlockMethod::Servo { channel, timer };
```

**Behavior**:
- Rotates servo to unlock position
- Holds for configured duration
- Returns to locked position

### 3. IR Remote
For IR-controlled gates (e.g., garage doors):

```rust
let method = UnlockMethod::remote(rmt, output, addr, cmd)?;
```

**Behavior**:
- Sends IR signal using RMT peripheral
- Supports custom address and command codes

## Environmental Monitoring

The beacon continuously monitors environmental conditions using the DHT11 sensor:

```rust
// Reading occurs every 50 seconds
Temperature: 24°C, Humidity: 60%
```

Data is logged to serial output and can be accessed via BLE (if EnvironmentalData capability is enabled).

## Development

### Building for Development

```bash
# Build with debug symbols
cargo build

# Flash and monitor
cargo espflash flash --monitor

# View logs
espmonitor /dev/ttyUSB0
```

### Testing

```bash
# Run unit tests (some tests require hardware)
cargo test --lib

# Run with verbose logging
RUST_LOG=debug cargo run
```

### Debugging

The firmware includes extensive logging:

```rust
println!("Request received: {:?}", message);
println!("Validation result: {:?}", result);
```

Use `espmonitor` to view logs in real-time.

## Security Considerations

### Best Practices

1. **Private Key Management**
   - Generate keys in a secure environment
   - Never commit private keys to version control
   - Use efuse read protection in production
   - Rotate keys periodically

2. **Server Configuration**
   - Validate all client requests server-side
   - Implement additional rate limiting
   - Log all access attempts
   - Monitor for suspicious patterns

3. **Physical Security**
   - Secure the beacon hardware enclosure
   - Use tamper-evident seals
   - Implement tamper detection (future feature)
   - Regular security audits

### Known Limitations

- No encrypted BLE connection (uses signed messages instead)
- Limited nonce storage (32 concurrent nonces)
- No persistent storage of unlock history
- Single server public key (no key rotation)

## Performance

- **Memory Usage**: ~120KB heap, ~80KB flash
- **Boot Time**: ~3 seconds to advertising
- **Response Time**: <100ms for unlock verification
- **BLE Range**: ~10-30 meters (depending on environment)
- **Power Consumption**: ~100mA active, ~10mA idle (with WiFi disabled)

## Troubleshooting

### Common Issues

**Beacon not advertising**
- Check BLE is enabled in sdkconfig
- Verify heap allocation (192KB required)
- Check for panic messages in serial output

**Authentication failures**
- Verify server public key matches server's private key
- Check device ID is correctly configured
- Ensure nonce hasn't expired (5-minute window)
- Verify system time is synchronized

**Relay not activating**
- Check GPIO pin configuration
- Verify relay module power supply
- Test relay with manual GPIO toggle
- Check for rate limiting (max 5 attempts)

**DHT11 read errors**
- Verify GPIO4 connection
- Check sensor power (3.3V or 5V)
- Ensure proper pull-up resistor (4.7kΩ)
- Allow sensor warm-up time (2 seconds)

## License

MIT License - see [LICENSE](LICENSE) file for details.

Copyright (c) 2023 Ethan Wu

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## Roadmap

- [ ] WiFi connectivity for remote management
- [ ] Over-the-air (OTA) firmware updates
- [ ] Persistent storage of access logs
- [ ] Multi-server public key support
- [ ] Encrypted BLE connections
- [ ] Battery power optimization
- [ ] Web dashboard for configuration
- [ ] Integration with cloud authentication services

## Support

For questions, issues, or feature requests:
- Open an issue on GitHub

## Acknowledgments

- Built with [esp-rs](https://github.com/esp-rs) ecosystem
- BLE implementation using [bleps](https://github.com/bjoernQ/bleps)
- Cryptography via [RustCrypto](https://github.com/RustCrypto)
- ESP32-C3 hardware platform by Espressif Systems

---

**Project**: Navign Beacon System  
**Repository**: `beacon`
**Version**: 0.1.0  
**MSRV**: 1.86+  
**Target**: ESP32-C3 RISC-V microcontroller
