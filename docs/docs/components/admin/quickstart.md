# Quick Start Guide

Get the robot management system running in 5 minutes.

## Prerequisites

- Rust 1.86+ 
- Go 1.25+
- Protocol Buffers compiler (`protoc`)

## Step 1: Build Everything

```bash
# Build Rust orchestrator
cd admin/orchestrator
cargo build --release

# Build Go tower  
cd ../tower
make proto
go build -o tower ./cmd/tower
```

## Step 2: Start the Orchestrator

```bash
cd admin/orchestrator
RUST_LOG=info cargo run
```

Output:
```
Orchestrator gRPC server listening on [::1]:50051
```

## Step 3: Start the Tower

In a new terminal:

```bash
cd admin/tower
./tower --entity-id "mall-001" --grpc "localhost:50051"
```

Output:
```
Connected to orchestrator at localhost:50051
Starting Socket.IO server on http://[::1]:8080
Task assignment stream requested for entity: mall-001
Controller started successfully
```

## Step 4: Connect a Robot

Robots connect via Socket.IO. Example using Node.js:

```javascript
const io = require('socket.io-client');

const socket = io('http://[::1]:8080');

// Register robot
socket.on('connect', () => {
  console.log('Connected to tower');
  
  socket.emit('register', JSON.stringify({
    robot_id: 'robot-001',
    name: 'Delivery Bot 1',
    entity_id: 'mall-001',
    battery: 95.5,
    timestamp: Date.now()
  }));
});

// Receive task assignments
socket.on('task_assigned', (data) => {
  const task = JSON.parse(data);
  console.log('Received task:', task);
  
  // Report progress
  socket.emit('task_update', JSON.stringify({
    task_id: task.task_id,
    robot_id: 'robot-001',
    status: 'in_progress',
    progress: 50,
    timestamp: Date.now()
  }));
});

// Send periodic status
setInterval(() => {
  socket.emit('status_update', JSON.stringify({
    robot_id: 'robot-001',
    state: 'idle',
    current_location: { x: 100, y: 200, z: 0, floor: '1F' },
    battery: 95.5,
    timestamp: Date.now()
  }));
}, 10000);

// Respond to pings
socket.on('keep_alive', () => {
  socket.emit('ping', JSON.stringify({ timestamp: Date.now() }));
});
```

## Verify Everything Works

Check the logs:

**Orchestrator:**
```
Robot registered: robot-001 (Entity: mall-001, Battery: 95.5%, State: Idle)
Robot status updated: robot-001 - 1
```

**Tower:**
```
Robot registering: ID=robot-001, Name=Delivery Bot 1, Entity=mall-001, Battery=95.5%
Keep-alive loop started for robot: robot-001
Robot status reported: robot-001
```

## Next Steps

- Read `admin/README.md` for architecture details
- Read `admin/INTEGRATION.md` for integration with other components
- Modify task creation logic in orchestrator
- Implement actual robot navigation logic

## Common Issues

**Tower can't connect to orchestrator:**
- Ensure orchestrator is running
- Check `--grpc` address matches orchestrator's listen address
- Verify ports are not blocked

**Robot can't connect to tower:**
- Ensure tower is running
- Check Socket.IO URL matches `--tower` address
- Verify WebSocket connections are allowed

**Tasks not being assigned:**
- Ensure robot state is IDLE
- Check battery level > 0
- Verify entity_id matches between robot and task
