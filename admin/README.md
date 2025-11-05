# Admin - Robot Management System

This directory contains the robot management system for Navign, consisting of two main components:

## Architecture

```
┌─────────────────────────────────────────────────┐
│              Rust Orchestrator                  │
│         (gRPC Server - The Brain)               │
│                                                 │
│  • Task scheduling & assignment                 │
│  • Robot registry & tracking                    │
│  • Best robot selection (battery + proximity)   │
│  • Business logic & decision making             │
└─────────────┬───────────────────────────────────┘
              │ gRPC (bidirectional)
              │ • ReportRobotStatus (Go → Rust)
              │ • GetTaskAssignment stream (Rust → Go)
              │
┌─────────────▼───────────────────────────────────┐
│                Go Tower                         │
│   (gRPC Client + Socket.IO Server)              │
│                                                 │
│  • Socket.IO connection manager                 │
│  • One goroutine per robot (keep-alive)         │
│  • Forwards robot status to orchestrator        │
│  • Forwards task assignments to robots          │
└─────────────┬───────────────────────────────────┘
              │ Socket.IO (WebSocket)
              │ • Robot registration
              │ • Status updates
              │ • Task assignments
              │ • Keep-alive ping/pong
              │
     ┌────────┴────────┬────────────┐
     │                 │            │
┌────▼─────┐    ┌─────▼────┐  ┌───▼──────┐
│ Robot 1  │    │ Robot 2  │  │ Robot N  │
└──────────┘    └──────────┘  └──────────┘
```

## Components

### 1. Orchestrator (Rust) - `admin/orchestrator/`

The brain of the system, implemented in Rust for performance and reliability.

**Responsibilities:**
- Receives robot status reports from Go tower via gRPC
- Maintains a registry of all connected robots
- Implements task scheduling and robot selection algorithms
- Streams task assignments to Go tower
- Makes all business logic decisions

**Technologies:**
- Rust with Tokio async runtime
- Tonic for gRPC
- Protocol Buffers for serialization

**Running:**
```bash
cd admin/orchestrator
cargo run
# Listens on [::1]:50051
```

### 2. Tower (Go) - `admin/tower/`

The communication layer, implemented in Go to leverage goroutines for concurrent robot management.

**Responsibilities:**
- Acts as gRPC client to the Rust orchestrator
- Manages Socket.IO connections with robots
- Spawns one goroutine per robot for:
  - Keep-alive monitoring (5 second interval)
  - Periodic status reporting (10 second interval)
  - Stale robot cleanup (30 second timeout)
- Forwards task assignments from orchestrator to robots
- Reports robot status changes to orchestrator

**Technologies:**
- Go with goroutines
- gRPC client
- Socket.IO for real-time robot communication

**Running:**
```bash
cd admin/tower
go build -o tower ./cmd/tower
./tower --entity-id "mall-123" --grpc "localhost:50051" --tower "http://[::1]:8080"
```

**Arguments:**
- `--entity-id`: Required. The entity (mall/building) ID
- `--grpc`: Rust orchestrator gRPC address (default: `localhost:50051`)
- `--tower`: Socket.IO server address (default: `http://[::1]:8080`)

## Protocol Buffers Schema

The system uses Protocol Buffers for type-safe communication. The schema is defined in `admin/tower/proto/task.proto`:

### Key Message Types

**Task:**
- `id`: Unique task identifier
- `type`: DELIVERY, PATROL, RETURN_HOME, EMERGENCY
- `sources`: Starting locations
- `terminals`: Destination locations
- `priority`: LOW, NORMAL, HIGH, URGENT
- `metadata`: Additional parameters

**RobotInfo:**
- `id`: Robot identifier
- `state`: IDLE, BUSY, CHARGING, ERROR, OFFLINE
- `current_location`: Current position (x, y, z, floor)
- `battery_level`: 0-100%
- `current_task_id`: Currently executing task

### RPC Services

**OrchestratorService** (implemented by Rust):
- `ReportRobotStatus(RobotReportRequest) → RobotReportResponse`
  - Called by Go tower to report robot status
- `GetTaskAssignment(RobotDistributionRequest) → stream TaskAssignment`
  - Streaming RPC for task assignments from Rust to Go

## Socket.IO Events

Communication between Go tower and robots uses Socket.IO with these events:

### From Robot to Tower:
- `register`: Robot connects and registers
  - Payload: `{ robot_id, name, entity_id, battery, timestamp }`
- `status_update`: Robot reports status change
  - Payload: `{ robot_id, state, current_location, battery, current_task_id, timestamp }`
- `task_update`: Robot reports task progress
  - Payload: `{ task_id, robot_id, status, progress, message, timestamp }`
- `ping`: Keep-alive from robot
  - Payload: `{ timestamp }`

### From Tower to Robot:
- `task_assigned`: Tower assigns a task
  - Payload: `{ task_id, type, sources[], terminals[], priority, metadata, assigned_at }`
- `keep_alive`: Periodic keep-alive check
  - Payload: `{ robot_id, timestamp }`
- `pong`: Response to ping
  - Payload: `{ timestamp }`

## Data Flow Examples

### Robot Registration Flow:
1. Robot connects to Go tower via Socket.IO
2. Robot sends `register` event with ID, battery, location
3. Go tower stores robot info and starts goroutine
4. Goroutine reports robot to Rust orchestrator via gRPC
5. Rust orchestrator adds robot to registry

### Task Assignment Flow:
1. External system creates a task (or orchestrator generates one)
2. Rust orchestrator finds best robot (highest battery + closest location)
3. Rust streams TaskAssignment to Go tower
4. Go tower receives assignment and finds robot's Socket.IO connection
5. Go tower sends `task_assigned` event to robot
6. Robot executes task and sends `task_update` events
7. Updates flow back through Go → Rust

### Status Reporting Flow:
1. Robot sends `status_update` event to Go tower
2. Go tower updates local robot state
3. Go tower calls orchestrator's `ReportRobotStatus` gRPC
4. Rust orchestrator updates robot registry
5. If robot becomes idle, it's eligible for new tasks

## Building

### Build Protobuf Code:

**Go:**
```bash
cd admin/tower
make proto
```

**Rust:**
```bash
cd admin/orchestrator
cargo build
# Proto code is generated automatically via build.rs
```

### Build Everything:
```bash
# Rust orchestrator
cd admin/orchestrator && cargo build --release

# Go tower
cd admin/tower && go build -o tower ./cmd/tower
```

## Development

### Prerequisites:
- Rust 1.86+ (for orchestrator)
- Go 1.25+ (for tower)
- Protocol Buffers compiler (`protoc`)
- `protoc-gen-go` and `protoc-gen-go-grpc` for Go
- `tonic-build` for Rust (added as build dependency)

### Testing Locally:

1. Start the Rust orchestrator:
```bash
cd admin/orchestrator
RUST_LOG=info cargo run
```

2. Start the Go tower:
```bash
cd admin/tower
./tower --entity-id "test-mall-001"
```

3. Connect a test robot to `http://[::1]:8080` via Socket.IO

## Design Decisions

### Why Rust for Orchestrator?
- Performance: Handles many robots efficiently
- Safety: Type system prevents bugs in critical scheduling logic
- Async: Tokio provides excellent async/await support
- Protobuf: First-class support via Tonic

### Why Go for Tower?
- Goroutines: Perfect for one-thread-per-robot pattern
- Simple: Easy to manage concurrent connections
- Socket.IO: Good library support for real-time communication
- Minimal logic: Just connection management and forwarding

### Why Split?
- **Separation of concerns**: Business logic (Rust) vs connection management (Go)
- **Leverage strengths**: Use Rust for algorithms, Go for concurrency
- **Scalability**: Can run multiple tower instances for different entities
- **Testability**: Can test scheduling logic independently of I/O

## Future Enhancements

- [ ] Add authentication for robot connections
- [ ] Implement task retry logic
- [ ] Add metrics and monitoring
- [ ] Support for robot capabilities/constraints
- [ ] Dynamic pathfinding integration
- [ ] Multi-entity orchestrator support
- [ ] Horizontal scaling for tower instances
