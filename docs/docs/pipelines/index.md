# Data Flow Pipelines

This section documents the end-to-end data flow pipelines in the Navign system, showing how different components interact to achieve specific functionality.

## Overview

Navign uses multiple interconnected pipelines to provide indoor navigation, access control, robot coordination, and firmware management. Each pipeline involves multiple components working together to deliver a seamless user experience.

## Available Pipelines

### Navigation Pipeline
**[Navigation](./navigation.md)** - Complete flow for indoor pathfinding and turn-by-turn navigation

The navigation pipeline handles user requests for directions within buildings, calculating optimal routes across multiple floors and providing step-by-step instructions.

**Flow:** Mobile App → Server → Pathfinding Engine → Mobile App → User

---

### Localization Pipeline
**[Localization](./localization.md)** - Indoor positioning using BLE beacons

The localization pipeline enables real-time position tracking using BLE beacon triangulation, allowing the mobile app to determine the user's exact location within a building.

**Flow:** BLE Beacons → Mobile App → Triangulation Algorithm → Position Update

---

### Access Control Pipeline
**[Unlock](./unlock.md)** - Secure contactless door/gate access

The unlock pipeline provides cryptographically secure access control, allowing users to unlock doors, gates, and turnstiles using their mobile device without physical contact.

**Flow:** Mobile App → BLE Beacon → Signature Verification → Relay/Servo Activation

---

### Robot Control Pipeline
**[Robot Control](./robot-control.md)** - Autonomous delivery robot coordination

The robot control pipeline manages task assignment, pathfinding, and execution for delivery robots operating within buildings.

**Flow:** Mobile/API → Server → Orchestrator → Tower → Robot → Navigation → Task Completion

---

### OTA Update Pipeline
**[OTA Updates](./ota.md)** - Over-the-air firmware updates for robots

The OTA pipeline enables remote firmware updates for robot components without requiring physical access to the devices.

**Flow:** Firmware Upload → Server → Orchestrator → Robot → Download → Flash → Reboot

---

### Firmware OTA Pipeline
**[Firmware OTA](./firmware-ota.md)** - Over-the-air updates for ESP32-C3 beacons

The firmware OTA pipeline provides remote firmware updates for BLE beacon devices, enabling maintenance and feature updates without physical access.

**Flow:** Firmware Upload → Server → Beacon WiFi → Download → Flash Partition → Activate → Reboot

---

## Architecture Patterns

All pipelines follow common architectural patterns:

1. **Security First** - Cryptographic verification at each step
2. **Fault Tolerance** - Graceful error handling and rollback mechanisms
3. **Real-time Communication** - WebSocket, gRPC, and BLE for low-latency updates
4. **Event Logging** - Comprehensive audit trails for all operations
5. **Offline Support** - Local caching and deferred synchronization where applicable

## Integration Points

The pipelines share several key integration points:

- **MongoDB Database** - Central data persistence
- **REST API** - Primary interface for mobile and admin clients
- **gRPC** - High-performance communication between backend services
- **Zenoh Message Bus** - Pub/sub messaging for robot components
- **BLE Protocol** - Local wireless communication with beacons and robots

## See Also

- [Components Overview](../components/) - Individual component documentation
- [Testing](../testing/) - Testing strategies for each pipeline
- [Development](../development/) - Development guides and critical TODOs
