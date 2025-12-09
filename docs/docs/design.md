# Navign - Innovative Project Design Plan

## 1. Project Overview

### 1.1 Project Name
**Navign** - Intelligent Indoor Navigation and Automation Platform

### 1.2 Project Vision
To create a comprehensive indoor navigation and automation ecosystem that seamlessly integrates BLE beacon positioning, secure access control, autonomous robot delivery, and AI-powered voice interaction, providing an intelligent spatial experience for large-scale indoor environments.

### 1.3 Target Application Scenarios
- Shopping Malls
- Transportation Hubs (Airports, Train Stations)
- Hospitals
- Schools and Universities

---

## 2. Core Innovation Points

### 2.1 Multi-Modal Indoor Positioning
- **BLE Beacon Triangulation**: ESP32-C3 based beacons broadcast signals for real-time position calculation via RSSI triangulation
- **AprilTag Visual Localization**: Camera-based marker detection for robot precise positioning
- **Multi-floor Navigation**: Dijkstra algorithm with support for elevators, escalators, and stairs

### 2.2 Cryptographic Access Control
- **P-256 ECDSA Signatures**: Industry-standard elliptic curve cryptography
- **Nonce-based Challenge-Response**: Prevents replay attacks with 5-second TTL
- **Hardware Key Storage**: ESP32 eFuse provides tamper-resistant private key storage
- **Biometric Authentication**: Touch ID/Face ID integration on mobile devices

### 2.3 Autonomous Robot Delivery System
- **Distributed Architecture**: Zenoh pub/sub messaging for component decoupling
- **Intelligent Task Assignment**: Robot selection algorithm based on distance, battery, and workload
- **Multi-Layer Control**:
  - Upper Layer (Raspberry Pi): Scheduler, Vision, Audio, Network
  - Lower Layer (STM32): Motor control, sensor fusion

### 2.4 AI-Powered Accessibility
- **Scene Description for Visually Impaired**: 3D object coordinates converted to natural language
- **Hybrid LLM Architecture**: Local inference (Qwen3) with cloud fallback (GPT-4o/DeepSeek)
- **Voice Interaction**: Wake word detection, speech recognition, and TTS

---

## 3. System Architecture

### 3.1 Overall Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Mobile App (Vue 3 + Tauri 2)                  │
│         Navigation │ Access Control │ Admin Panel │ Robot Tracking   │
└────────────┬─────────────────────────────────────────┬───────────────┘
             │ HTTPS/REST                              │ BLE GATT
             ▼                                         ▼
┌────────────────────────┐                 ┌─────────────────────────┐
│    Central Server      │                 │    BLE Beacons          │
│    (Rust/Axum)         │                 │    (ESP32-C3)           │
│  ─────────────────────│                 │  ───────────────────────│
│  • REST API            │                 │  • Indoor Positioning   │
│  • Pathfinding         │◄───── SSE ─────►│  • Access Control       │
│  • OAuth2 Auth         │                 │  • Environmental Sensing│
│  • PostgreSQL          │                 │  • OTA Updates          │
└────────────┬───────────┘                 └─────────────────────────┘
             │ gRPC
             ▼
┌──────────────────────────────────────────────────────────────────────┐
│                    Admin System (Local Mall)                          │
├─────────────────────┬─────────────────────┬──────────────────────────┤
│    Orchestrator     │       Tower         │      Maintenance         │
│    (Rust/gRPC)      │    (Go/Socket.IO)   │    (Python CLI)          │
│  ─────────────────  │  ─────────────────  │  ──────────────────────  │
│  • Task Queue       │  • Robot WebSocket  │  • Key Generation        │
│  • Robot Selection  │  • Status Relay     │  • eFuse Programming     │
│  • Fleet Management │  • Task Streaming   │  • Beacon Registration   │
└─────────────────────┴──────────┬──────────┴──────────────────────────┘
                                 │ Socket.IO
                                 ▼
┌──────────────────────────────────────────────────────────────────────┐
│                    Autonomous Robot System                            │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────── Upper Layer (Raspberry Pi) ─────────────┐  │
│  │  ┌──────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ ┌──────────┐  │  │
│  │  │Scheduler │ │ Network │ │  Vision │ │ Audio  │ │Intelligence│ │  │
│  │  │  (Rust)  │ │ (Rust)  │ │  (C++)  │ │(Python)│ │  (Python) │  │  │
│  │  └────┬─────┘ └────┬────┘ └────┬────┘ └───┬────┘ └─────┬─────┘  │  │
│  │       └────────────┴──────────┴───────────┴────────────┘        │  │
│  │                          │ Zenoh Pub/Sub                        │  │
│  └──────────────────────────┼──────────────────────────────────────┘  │
│                             │ UART (Postcard)                         │
│  ┌─────────────────────── Lower Layer (STM32F407) ─────────────────┐  │
│  │              Motor Control │ Sensors │ Actuators                 │  │
│  │                      (Rust/Embassy Async)                        │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

### 3.2 Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Server | Rust (Axum) | REST API, Pathfinding, Authentication |
| Mobile | Vue 3 + Tauri 2 | Cross-platform App |
| Beacon Firmware | Rust (ESP-HAL) | BLE, Access Control, Sensors |
| Orchestrator | Rust (Tonic/gRPC) | Robot Fleet Management |
| Tower | Go (Socket.IO) | Robot Communication Hub |
| Robot Upper | Rust + C++ + Python | Vision, Audio, Scheduling |
| Robot Lower | Rust (Embassy) | Motor Control, Sensors |
| Shared Library | Rust (no_std) | Cross-component Types |
| Database | PostgreSQL | Primary Data Storage |

---

## 4. Core Module Design

### 4.1 Indoor Positioning Module

**Innovation**: Multi-source fusion positioning combining BLE RSSI and visual markers

**Technical Implementation**:
```
Position = α × BLE_Triangulation + β × AprilTag_Pose + γ × IMU_Dead_Reckoning
```

**Key Features**:
- 300+ beacon support per entity
- 1-3 meter positioning accuracy
- Multi-floor seamless handoff
- Offline positioning capability

### 4.2 Secure Access Control Module

**Innovation**: Hardware-backed cryptographic access with nonce-based replay prevention

**Security Flow**:
```
1. Mobile → Beacon: NonceRequest
2. Beacon: Generate 32-byte random nonce, store with timestamp
3. Beacon → Mobile: NonceResponse (nonce + signature_id)
4. Mobile: Create ECDSA proof = Sign(nonce || device_id, user_private_key)
5. Mobile → Beacon: UnlockRequest(proof)
6. Beacon: Verify signature, check nonce freshness (<5s), check rate limit
7. Beacon: Activate relay/servo if valid
8. Beacon → Mobile: UnlockResponse(success/error)
```

**Security Measures**:
- P-256 ECDSA cryptography
- 5-second nonce expiration
- 5 attempts per 5 minutes rate limiting
- Hardware eFuse key storage (write-once, read-protected)

### 4.3 Robot Delivery Module

**Innovation**: Distributed microservices architecture with intelligent task assignment

**Robot Selection Algorithm**:
```python
def select_robot(task, robots):
    candidates = [r for r in robots
                  if r.entity_id == task.entity_id
                  and r.state == IDLE
                  and r.battery > 20%]

    return min(candidates, key=lambda r:
        0.6 * distance(r.location, task.source) +
        0.3 * (100 - r.battery) +
        0.1 * r.queue_length
    )
```

**Component Communication**:
- Zenoh pub/sub for inter-service messaging
- Protocol Buffers for type-safe serialization
- gRPC streaming for real-time task updates

### 4.4 AI Accessibility Module

**Innovation**: Hybrid local/remote LLM for scene description

**Architecture**:
```python
def describe_scene(objects_3d, user_query):
    # Attempt local inference first (low latency)
    response = local_llm.generate(scene_prompt, objects_3d)

    if response == "<remote>":  # Complex query detected
        response = remote_llm.generate(scene_prompt, objects_3d)

    return response
```

**Use Case**:
Robot vocally describes surroundings to visually impaired users:
> "There is a coffee shop 3 meters ahead on your left. The elevator is 10 meters straight ahead. There are 2 people walking towards you from the right."

---

## 5. Communication Protocol Design

### 5.1 Orchestrator-Central Server Protocol

**Event-Driven Architecture**:
- Server-Sent Events (SSE) for real-time updates
- Firewall-friendly (orchestrator initiates all connections)
- Automatic reconnection with exponential backoff

**Event Types**:
| Event | Purpose |
|-------|---------|
| `data_update` | Entity/area/beacon data changes |
| `firmware_update` | New beacon firmware available |
| `task_create` | New delivery task |
| `config_update` | Robot selection parameters |
| `access_log_request` | Request access logs upload |

### 5.2 BLE Protocol

**Message Types** (Postcard binary serialization):
```rust
pub enum BleMessage {
    DeviceRequest,
    DeviceResponse(DeviceType, Capabilities, DeviceId),
    NonceRequest,
    NonceResponse(Nonce, SignatureId),
    UnlockRequest(Proof),
    UnlockResponse(bool, Option<Error>),
}
```

### 5.3 Robot Inter-Component Protocol

**Zenoh Topics**:
| Topic | Publisher | Subscriber | Purpose |
|-------|-----------|------------|---------|
| `robot/scheduler/status` | Scheduler | Tower | Robot state updates |
| `robot/vision/objects` | Vision | Scheduler | Detected objects |
| `robot/audio/wake_word` | Audio | Scheduler | Voice activation |
| `robot/serial/sensors` | Serial | Scheduler | Sensor data from STM32 |
| `robot/network/pathfinding/response` | Network | Scheduler | Navigation paths |

---

## 6. Data Model Design

### 6.1 Core Entities

```
Entity (Building)
├── id: UUID
├── name: String
├── type: Mall | Transportation | School | Hospital
├── location: (longitude, latitude)
└── floors: [Floor]

Area (Zone within Entity)
├── id: UUID
├── entity_id: UUID
├── name: String
├── floor: String
├── polygon: [Coordinate]
└── beacon_codes: [String]

Beacon (BLE Device)
├── id: UUID
├── entity_id: UUID
├── device_id: [u8; 24]
├── device_type: Merchant | Pathway | Connection | Turnstile
├── capabilities: [UnlockGate, EnvironmentalData, ...]
├── location: Coordinate
└── public_key: P256PublicKey

Connection (Inter-Area Link)
├── id: UUID
├── entity_id: UUID
├── from_area: UUID
├── to_area: UUID
├── type: Elevator | Escalator | Stairs
└── bidirectional: bool
```

### 6.2 Robot System Entities

```
Robot
├── id: String
├── entity_id: String
├── state: Idle | Busy | Charging | Offline
├── battery_level: f32
├── current_location: Location
├── current_task_id: Option<String>
└── capabilities: [String]

Task
├── id: String
├── type: Delivery | Patrol | Guide
├── sources: [Location]
├── terminals: [Location]
├── priority: Low | Normal | High | Urgent
├── status: Queued | Assigned | InProgress | Completed | Failed
└── assigned_robot_id: Option<String>
```

---

## 7. Implementation Roadmap

### Phase 1: Core Infrastructure (Weeks 1-4)
- [ ] Server REST API and PostgreSQL integration
- [ ] Beacon BLE advertising and GATT services
- [ ] Mobile app navigation UI with MapLibre
- [ ] Shared library with cross-platform types

### Phase 2: Access Control (Weeks 5-8)
- [ ] P-256 ECDSA signature implementation
- [ ] Nonce-based challenge-response protocol
- [ ] Biometric authentication integration
- [ ] eFuse key programming tools (maintenance CLI)

### Phase 3: Robot System (Weeks 9-12)
- [ ] Orchestrator gRPC server
- [ ] Tower Socket.IO relay
- [ ] Robot scheduler and serial bridge
- [ ] Vision service (AprilTag + YOLO)
- [ ] STM32 lower controller firmware

### Phase 4: AI Features (Weeks 13-16)
- [ ] Audio service (wake word, STT, TTS)
- [ ] Intelligence service (hybrid LLM)
- [ ] Scene description for accessibility
- [ ] Multi-language support (5 languages)

### Phase 5: Production Hardening (Weeks 17-20)
- [ ] OTA firmware update system
- [ ] Comprehensive security audit
- [ ] Load testing and optimization
- [ ] Documentation and deployment guides

---

## 8. Expected Outcomes

### 8.1 Technical Metrics

| Metric | Target |
|--------|--------|
| Positioning Accuracy | 1-3 meters |
| Access Control Latency | < 500ms |
| Robot Task Assignment | < 2 seconds |
| Vision Processing | > 30 FPS |
| Speech Recognition | > 95% accuracy |
| System Uptime | > 99.9% |
| Concurrent Users | 10,000+ per entity |

### 8.2 Capacity Planning

| Resource | Specification |
|----------|---------------|
| Beacons per Entity | Up to 500 |
| Robots per Entity | Up to 50 |
| Concurrent Tasks | Up to 1,000 |
| Areas per Entity | Up to 150 |
| Merchants per Entity | Up to 200 |

### 8.3 Business Value
- **Enhanced User Experience**: Seamless indoor navigation and contactless access
- **Operational Efficiency**: Automated delivery reduces labor costs by 40%
- **Accessibility**: AI-powered assistance for visually impaired users
- **Scalability**: Supports multiple entities with centralized management
- **Security**: Enterprise-grade cryptographic protection

---

## 9. Risk Analysis and Mitigation

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| BLE signal interference | High | Medium | Multi-beacon triangulation, visual backup |
| Network connectivity loss | High | Low | Offline operation mode, local caching |
| Robot hardware failure | Medium | Medium | Redundant fleet, automatic task reassignment |
| Security breach attempt | High | Low | Hardware key storage, rate limiting, audit logs |
| LLM API unavailability | Medium | Low | Local LLM fallback, graceful degradation |

---

## 10. Innovation Summary

| Innovation Area | Technical Approach | Competitive Advantage |
|-----------------|-------------------|----------------------|
| **Indoor Positioning** | BLE + Visual marker fusion | Higher accuracy than BLE-only solutions |
| **Access Control** | Hardware-backed ECDSA | Enterprise-grade security |
| **Robot Delivery** | Distributed microservices | Scalable and fault-tolerant |
| **AI Accessibility** | Hybrid local/remote LLM | Low latency with cloud fallback |
| **Cross-Platform** | Rust + Tauri + Vue | Single codebase for all platforms |
| **Embedded Systems** | Rust no_std + Embassy | Memory-safe, real-time capable |
| **Protocol Design** | Event-driven SSE + gRPC | Firewall-friendly, real-time updates |

---

## 11. Project Repository Structure

```
navign/
├── server/              # Rust backend server (Axum)
├── mobile/              # Vue + Tauri cross-platform app
├── firmware/            # ESP32-C3 BLE beacon firmware
├── robot/               # Robot distributed system
│   ├── scheduler/       # Task coordination (Rust)
│   ├── serial/          # UART bridge (Rust)
│   ├── network/         # Server communication (Rust)
│   ├── vision/          # Computer vision (C++)
│   ├── audio/           # Voice interaction (Python)
│   ├── intelligence/    # AI/LLM (Python)
│   └── lower/           # STM32 motor control (Rust)
├── admin/
│   ├── orchestrator/    # Robot fleet management (Rust gRPC)
│   ├── tower/           # Robot WebSocket server (Go)
│   ├── maintenance/     # Key management CLI (Python)
│   └── proto/           # Protocol Buffer definitions
├── shared/              # Cross-platform Rust library (no_std)
├── docs/                # VitePress documentation
└── schematics/          # KiCad PCB designs
```

---

## 12. Conclusion

Navign represents a comprehensive solution for intelligent indoor environments, combining cutting-edge technologies in positioning, security, robotics, and AI. The polyglot architecture leverages the strengths of multiple programming languages while maintaining type safety and performance through shared Rust libraries and Protocol Buffers.

The project's modular design enables independent development and deployment of components, while the event-driven communication protocols ensure real-time responsiveness and system resilience. With a focus on accessibility, security, and user experience, Navign aims to transform how people navigate and interact with large indoor spaces.
