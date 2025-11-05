# Beacon

The Beacon component consists of ESP32-C3 based BLE beacons that facilitate indoor positioning within the Navign system. These beacons emit Bluetooth signals that are detected by mobile devices, allowing for accurate location tracking and navigation assistance indoors.

Language: Rust

# Features

- **ESP32-C3 Microcontroller**: Utilizes the ESP32-C3 for efficient BLE communication and low power consumption.
- **BLE Signal Emission**: Beacons broadcast Bluetooth Low Energy signals for real-time location tracking.
- **Authentication**: Implements security measures such as ECDSA signatures and TOTP for secure communication for access control.

# Functionality

- **Indoor Positioning**: Provides accurate indoor location data to mobile devices using RSSI-based triangulation.
- **Access Control**: Enables secure access to restricted areas through beacon authentication.
- **Environment Measurements**: Collects environmental data, including humidity and temperature, to enhance user experience and system performance.

# Security

- **Cryptographic Protection**: Utilizes P-256 ECDSA signatures, TOTP authentication, and nonce-based challenge-response mechanisms to ensure secure communication and prevent replay attacks.
- **Hardware Key Storage**: Leverages ESP32 efuse for secure storage of cryptographic keys.
- **Access Control**: Manages access to doors and merchant spaces through mobile app authentication and beacon verification.

# Libraries

- **esp-hal**: Hardware abstraction layer for ESP32-C3. Bare-metal programming without an operating system.
- **esp-radio**: Library for handling BLE radio functionalities on ESP32-C3.
- **bleps**: BLE protocol stack implementation for ESP32-C3.

# Roadmap

- [ ] Implement Over-the-Air (OTA) updates for beacons.
- [ ] Interact with local orchestration services for dynamic configuration and management.
- [ ] Enhance environmental sensing capabilities with additional sensors.
- [ ] Optimize power consumption for extended battery life.
