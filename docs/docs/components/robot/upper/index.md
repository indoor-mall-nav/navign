# Robot Upper Layer

The robot upper layer is a distributed control system for autonomous delivery robots, using Zenoh pub/sub messaging and Protocol Buffers for inter-component communication.

## Architecture Overview

```
         Vision              Audio        (Python Services)
       (AprilTag,          (Wake Word,
         YOLO)                TTS)
            |                  |
            +--------+---------+
                     |
               [Zenoh Bus]
                     |
        +------------+------------+------------+
        |            |            |            |
    Scheduler     Network      Serial       Tower
     (Rust)        (Rust)      (Rust)     (Socket)
        |                         |
        |                         v
        |                  [Lower/STM32]
        |               (Motors, Sensors)
        v
  [Task Database]
```

## Components

### Rust Services

- **[Scheduler](scheduler.md)** - Central task coordinator and decision engine
- **[Serial](serial.md)** - UART bridge to STM32 lower controller
- **[Navign](navign.md)** - HTTP client for server API integration

### Python Services

- **[Vision](vision.md)** - Computer vision (YOLO, AprilTag, MediaPipe)
- **[Audio](audio.md)** - Voice interaction (wake word, STT, TTS)

### Communication

- **[Protocol Buffers](../../../pipelines/robot-control.md)** - Message schemas
- **Zenoh** - Distributed pub/sub messaging bus

## Quick Start

### Prerequisites

```bash
# Rust components
rustup default stable

# Python components (vision, audio)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Zenoh runtime (optional, embedded by default)
# https://zenoh.io/docs/getting-started/installation/
```

### Running All Components

```bash
# Terminal 1 - Scheduler
cd robot/scheduler && cargo run

# Terminal 2 - Serial
cd robot/serial && SERIAL_PORT=/dev/ttyUSB0 cargo run

# Terminal 3 - Network
cd robot/network && SERVER_URL=http://localhost:3000 cargo run

# Terminal 4 - Vision
cd robot/vision && uv run python service.py

# Terminal 5 - Audio
cd robot/audio && uv run python service.py
```

## Configuration

### Environment Variables

**Scheduler:**

- `ZENOH_CONFIG` - Zenoh configuration file (optional)
- `DATABASE_URL` - Task database connection

**Serial:**

- `SERIAL_PORT` - Default: `/dev/ttyUSB0`
- `SERIAL_BAUD` - Default: `115200`

**Network:**

- `SERVER_URL` - Default: `http://localhost:3000`
- `ENTITY_ID` - Robot's entity ID

**Vision:**

- `CAMERA_INDEX` - Default: `0`
- `YOLO_MODEL` - Default: `yolo12n.pt`

**Audio:**

- `PORCUPINE_ACCESS_KEY` - Required for wake word

## Message Flow Example

**Delivery Task Execution:**

1. Tower → Scheduler: `TaskSubmission` (Socket.IO)
2. Scheduler → Network: `PathfindingRequest` (Zenoh)
3. Network → Server: `GET /api/entities/{id}/route` (HTTP)
4. Network → Scheduler: `PathfindingResponse` (Zenoh)
5. Scheduler → Serial: `MotorCommand` (Zenoh)
6. Serial → Lower: Binary commands (UART/Postcard)
7. Lower → Serial: Sensor data (UART/Postcard)
8. Serial → Scheduler: `SensorDataResponse` (Zenoh)
9. Scheduler → Tower: `TaskUpdateReport` (gRPC)

## Development

```bash
# Format code
just fmt

# Lint all components
just lint

# Run tests
just ci-robot-upper

# Generate Protocol Buffers
just proto-robot
```

## Deployment

For production deployment with systemd, see `/robot/README.md`.

## See Also

- [Robot Lower Layer](../lower.md) - STM32 motor control
- [Admin Tower](../../admin/) - Fleet management
- [Server API](../../server.md) - Navigation backend
