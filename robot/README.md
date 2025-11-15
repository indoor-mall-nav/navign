# Robot Upper Layer Components

This directory contains the high-level robot control system with modular components for autonomous navigation, sensing, and communication.

## Architecture

Multi-component distributed system using **Zenoh** pub/sub messaging:

```
┌─────────────┐  ┌─────────────┐
│   Vision    │  │    Audio    │  (Python)
│ (AprilTag,  │  │ (Wake Word, │
│   YOLO)     │  │     TTS)    │
└──────┬──────┘  └──────┬──────┘
       │                │
       └────────┬───────┘
                │
          [Zenoh Bus]
                │
       ┌────────┴────────┬────────┬────────┐
       │                 │        │        │
  ┌────▼────┐  ┌────────▼──┐  ┌──▼───┐ ┌──▼──────┐
  │Scheduler│  │  Network  │  │Serial│ │  Tower  │
  │  (Rust) │  │  (Rust)   │  │(Rust)│ │(Socket) │
  └────┬────┘  └───────────┘  └──┬───┘ └─────────┘
       │                          │
       │                          ▼
       │                    [Lower/STM32]
       │                    (Motors, Sensors)
       ▼
  [Task Database]
```

## Components

### Protocol Buffers (`proto/`)
Unified message format using Protocol Buffers for inter-component communication.

**Files:**
- `common.proto` - Shared types
- `vision.proto` - Vision service
- `audio.proto` - Audio service
- `scheduler.proto` - Task management
- `serial.proto` - UART protocol
- `network.proto` - External communication

**Generation:**
```bash
just proto-robot-python  # Python (vision/audio)
just proto-robot         # All proto generation
```

### Scheduler (`scheduler/`)
**Language:** Rust
**Purpose:** Central coordinator for robot operations

**Responsibilities:**
- Task queue management
- Inter-component coordination
- Robot state tracking
- Navigation decision-making

**Run:** `cd scheduler && cargo run`

### Serial (`serial/`)
**Language:** Rust
**Purpose:** UART bridge to lower controller (STM32)

**Protocol:** Postcard binary serialization
**Baud Rate:** 115200

**Run:** `cd serial && cargo run`

### Network (`network/`)
**Language:** Rust
**Purpose:** External communication (server API, BLE)

**Features:**
- Pathfinding requests
- Entity data fetching
- Future: BLE operations

**Run:** `cd network && cargo run`

### Vision (`vision/`)
**Language:** Python
**Purpose:** Computer vision (AprilTag, YOLO)

**Technologies:**
- AprilTags for pose estimation
- YOLOv8 for object detection
- OpenCV for image processing

**Run:** `cd vision && uv run python service.py`

### Audio (`audio/`)
**Language:** Python
**Purpose:** Wake word detection and TTS

**Technologies:**
- Porcupine for wake word (migrating to OpenWakeWord)
- Edge TTS for text-to-speech

**Run:** `cd audio && uv run python service.py`

### Firmware (`firmware/`)
**Language:** Rust (embedded)
**Purpose:** Upper controller firmware (Raspberry Pi)

**Status:** Planned/skeleton
**Target:** Linux-based SBC (Raspberry Pi 4/5)

## Communication Pattern

All components communicate via **Zenoh topics** using **Protocol Buffers**:

### Key Topics
- `robot/scheduler/task/submit` - Incoming tasks
- `robot/serial/sensors` - Sensor data from lower layer
- `robot/vision/updates` - Vision detections
- `robot/audio/events` - Wake word events
- `robot/network/pathfinding/request` - Navigation requests

### Message Flow (Example: Delivery Task)
1. Tower → Scheduler: `TaskSubmission`
2. Scheduler → Network: `PathfindingRequest`
3. Network → Scheduler: `PathfindingResponse`
4. Scheduler → Serial: `MotorCommand`
5. Serial → Lower: Postcard-encoded commands
6. Lower → Serial: Sensor data
7. Serial → Scheduler: `SensorDataResponse`
8. Scheduler → Tower: `TaskUpdateReport`

## Deployment

```bash
# Generate protobuf code
just proto-robot-python

# Start all components (use systemd/supervisor in production)
cd scheduler && cargo run &
cd serial && cargo run &
cd network && cargo run &
cd vision && uv run python service.py &
cd audio && uv run python service.py &
```

## Development

```bash
# Format code
just fmt

# Lint code
just lint

# Run CI checks for robot components
just ci-robot-upper
```

## Environment Variables

**Serial:**
- `SERIAL_PORT` - Default: `/dev/ttyUSB0`
- `SERIAL_BAUD` - Default: `115200`

**Network:**
- `SERVER_URL` - Default: `http://localhost:3000`

**Audio:**
- `PORCUPINE_ACCESS_KEY` - Required for wake word detection

## See Also
- `CLAUDE.md` - Full project documentation
- `robot/lower/` - STM32 lower controller
- `admin/tower/` - Robot fleet management
