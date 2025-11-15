# Admin

The `admin` component is the administrative interface *within an entity* (mall, hospital, etc.) for managing beacons, robots, and system settings. It consists of several subcomponents that handle different aspects of robot fleet management and communication.

This document explains how the robot management system integrates with other Navign components.

## Getting Started

- **[Quick Start Guide](./quickstart.md)** - Get the system running locally in 5 minutes for development
- **[Deployment Guide](./deployment.md)** - Production deployment with systemd, monitoring, and security

## Shared Schema Integration

The robot system uses the `shared` crate's schemas for consistency across the platform:

### Area & Entity Types

Robots operate within **Entities** (malls, hospitals, etc.) and navigate through **Areas**. The schemas are defined in `shared/src/schema/`:

- **Entity**: Building/complex identifier (from `shared/src/schema/entity.rs`)
  - Types: Mall, Transportation, School, Hospital
  - Contains location bounds, name, description

- **Area**: Physical zones within an entity (from `shared/src/schema/area.rs`)
  - Contains polygon coordinates, floor information
  - Beacon codes for positioning

- **Floor**: Floor numbering system (from `shared/src/schema/area.rs`)
  - Types: Level (UK: Ground, First), Floor (US: 1st, 2nd), Basement
  - Consistent with mobile app navigation

### Location Coordinates

The protobuf `Location` message matches the coordinate system used throughout Navign:

```protobuf
message Location {
  double x = 1;        // Longitude-derived x coordinate
  double y = 2;        // Latitude-derived y coordinate
  double z = 3;        // Altitude (optional)
  string floor = 4;    // Floor identifier (e.g., "1F", "B2", "L1")
}
```

**Coordinate System:**
- X, Y are derived from the entity's longitude/latitude ranges
- Floor string matches the Area floor format
- Z coordinate can be used for altitude/elevation

## Integration with Server

The Navign server (`server/`) provides:

- Entity and Area management APIs
- Beacon authentication
- Pathfinding algorithms

### Task Creation Flow

Tasks are typically created based on server events:

```
Example: User requests delivery via mobile app
1. Mobile app → Server: Create delivery request
2. Server → Orchestrator: Submit task via gRPC
3. Orchestrator: Assign to best robot
4. Orchestrator → Tower: Stream task assignment
5. Tower → Robot: Send task via Socket.IO
```

### Pathfinding Integration

Robots need navigation paths from the server's pathfinding system:

1. Task contains source and terminal locations
2. Robot can query server's pathfinding API
3. Server returns optimal path considering:
   - Multi-floor routing (elevators, escalators, stairs)
   - Area connectivity
   - Real-time beacon positioning

**Server API Endpoint:**

```http
POST /api/v1/pathfinding/route
{
  "start": { "area": "A001", "x": 100, "y": 200 },
  "end": { "area": "A023", "x": 500, "y": 600 },
  "entity": "mall-123"
}
```

## Integration with Mobile App

The mobile app can interact with the robot system:

### Track Robot Status

Mobile app can display robot locations and status:

```typescript
// Query orchestrator for robot distribution
// (Would need to add a public API endpoint)
const robots = await fetchRobotStatus(entityId);

// Display on map using MapLibre GL
robots.forEach(robot => {
  addRobotMarker(robot.current_location, robot.state);
});
```

### Request Delivery

Users can request robot delivery:

```typescript
// Submit delivery request
const task = {
  type: 'DELIVERY',
  pickup: currentLocation,
  dropoff: merchantLocation,
  priority: 'NORMAL'
};

// Send to server, which forwards to orchestrator
await api.post('/delivery/request', task);
```

## Integration with Beacon System

Robots use beacons for positioning:

### Robot Beacon Interaction

1. Robot scans for BLE beacons
2. Uses RSSI triangulation (like mobile app)
3. Reports location to tower via Socket.IO
4. Tower forwards to orchestrator

**Beacon Types for Robots:**
- **Pathway beacons**: For navigation
- **Merchant beacons**: For pickup/delivery verification
- **Connection beacons**: For elevator/stair access

### Location Reporting Format

```javascript
// Robot sends status update
socket.emit('status_update', {
  robot_id: 'robot-001',
  state: 'busy',
  current_location: {
    x: 125.5,
    y: 230.8,
    z: 0.0,
    floor: '2F'
  },
  battery: 85.5,
  current_task_id: 'task-456',
  timestamp: Date.now()
});
```

## Example: Complete Delivery Workflow

### 1. User Requests Delivery
```typescript
// Mobile app (Vue + Tauri)
const deliveryRequest = {
  merchant_id: 'merchant-456',
  pickup_area: 'A023',
  pickup_coords: { x: 500, y: 600 },
  dropoff_area: 'A001',
  dropoff_coords: { x: 100, y: 200 },
  entity_id: 'mall-123'
};

await api.post('/delivery/request', deliveryRequest);
```

### 2. Server Creates Task
```rust
// Server (Rust + Axum)
let task = Task {
    id: Uuid::new_v4().to_string(),
    r#type: TaskType::Delivery as i32,
    sources: vec![Location {
        x: request.pickup_coords.x,
        y: request.pickup_coords.y,
        z: 0.0,
        floor: get_area_floor(request.pickup_area).await?,
    }],
    terminals: vec![Location {
        x: request.dropoff_coords.x,
        y: request.dropoff_coords.y,
        z: 0.0,
        floor: get_area_floor(request.dropoff_area).await?,
    }],
    priority: Priority::Normal as i32,
    created_at: chrono::Utc::now().timestamp(),
    entity_id: request.entity_id,
    metadata: HashMap::new(),
};

// Send to orchestrator (would need gRPC client in server)
orchestrator_client.submit_task(task).await?;
```

### 3. Orchestrator Assigns Task
```rust
// Orchestrator (Rust)
let best_robot = registry.find_best_robot(&task).await
    .ok_or("No robots available")?;

let assignment = TaskAssignment {
    robot_id: best_robot.id.clone(),
    task: Some(task.clone()),
};

// Send to tower via streaming gRPC
task_channel.send(Ok(assignment)).await?;
```

### 4. Tower Forwards to Robot
```go
// Tower (Go)
assignment, err := taskStream.Recv()
if err != nil {
    return err
}

// Find robot's Socket.IO connection
robotConn := findRobotConnection(assignment.RobotId)

// Send task
packet := TaskAssignedPacket{
    TaskID:    assignment.Task.Id,
    Type:      "delivery",
    Sources:   convertLocations(assignment.Task.Sources),
    Terminals: convertLocations(assignment.Task.Terminals),
    Priority:  "normal",
}

robotConn.Emit("task_assigned", packet)
```

### 5. Robot Executes Task
```javascript
// Robot (Node.js or embedded system)
socket.on('task_assigned', (task) => {
  console.log('Received task:', task.task_id);
  
  // 1. Get path from server
  const path = await getNavigationPath(task.sources[0], task.terminals[0]);
  
  // 2. Start navigation
  await navigatePath(path);
  
  // 3. Report progress
  socket.emit('task_update', {
    task_id: task.task_id,
    robot_id: 'robot-001',
    status: 'in_progress',
    progress: 50,
    timestamp: Date.now()
  });
  
  // 4. Complete task
  socket.emit('task_update', {
    task_id: task.task_id,
    robot_id: 'robot-001',
    status: 'completed',
    progress: 100,
    timestamp: Date.now()
  });
});
```

## Environment Variables

Configure integration points via environment variables:

**Orchestrator (Rust):**
```bash
RUST_LOG=info                    # Log level
ORCHESTRATOR_ADDR=[::1]:50051    # gRPC listen address
```

**Tower (Go):**
```bash
ENTITY_ID=mall-123                      # Required: Entity identifier
ORCHESTRATOR_ADDR=localhost:50051       # Orchestrator gRPC address
TOWER_ADDR=http://[::1]:8080            # Socket.IO server address
```

**Robot:**
```bash
TOWER_URL=http://tower.example.com:8080  # Tower Socket.IO endpoint
ROBOT_ID=robot-001                       # Unique robot identifier
ENTITY_ID=mall-123                       # Entity to operate in
```

## Security Considerations

### Authentication

**Robot Authentication:**
- Robots should authenticate with tower using tokens
- Consider using JWT or API keys
- Validate robot_id matches authenticated identity

**gRPC Security:**
- Use TLS for production gRPC connections
- Implement mTLS for orchestrator ↔ tower communication
- Add authorization checks for sensitive RPCs

### Data Validation

- Validate all location coordinates within entity bounds
- Verify floor identifiers match entity's floors
- Check battery levels are within valid range (0-100)
- Sanitize task metadata

### Rate Limiting

- Limit status update frequency per robot
- Throttle task assignment requests
- Implement exponential backoff for retries

## Monitoring & Observability

### Metrics to Track

**Orchestrator:**
- Active robot count by state
- Task assignment latency
- Task completion rate
- Robot selection algorithm performance

**Tower:**
- Socket.IO connection count
- gRPC stream health
- Status report frequency
- Failed message deliveries

**Integration:**
- End-to-end task latency (request → completion)
- Robot location accuracy
- Path deviation rate
- Battery drain analysis

### Logging

Use structured logging for easier debugging:

```rust
// Orchestrator
log::info!(
    "Task assigned: task_id={}, robot_id={}, battery={:.1}%",
    task.id,
    robot_id,
    robot.battery_level
);
```

```go
// Tower
log.Printf("Robot registered: id=%s, entity=%s, battery=%.1f%%",
    robot.ID, robot.EntityID, robot.Battery)
```

## Testing

### Unit Tests

Test individual components:
- Task assignment algorithm
- Robot selection logic
- Location coordinate conversions

### Integration Tests

Test component interactions:
- Orchestrator ↔ Tower gRPC communication
- Tower ↔ Robot Socket.IO messaging
- Task lifecycle (creation → assignment → completion)

### End-to-End Tests

Simulate complete workflows:
1. Create test entity and areas
2. Register mock robots
3. Submit tasks
4. Verify assignments
5. Simulate task execution
6. Verify status updates

## Troubleshooting

### Robot Not Receiving Tasks

1. Check robot is registered: `status_update` sent?
2. Verify robot state is IDLE
3. Check battery level > 20%
4. Confirm entity_id matches task's entity_id
5. Check Socket.IO connection is active

### Orchestrator Can't Reach Tower

1. Verify tower is listening on expected address
2. Check network connectivity
3. Ensure gRPC ports are not blocked
4. Verify protobuf versions match

### Location Coordinates Incorrect

1. Verify entity's longitude/latitude ranges are set
2. Check coordinate system consistency
3. Ensure floor identifiers match area floors
4. Validate beacon positioning data

## Future Enhancements

- [ ] WebSocket fallback for Socket.IO
- [ ] Multi-region orchestrator deployment
- [ ] Advanced robot capabilities matching
- [ ] Dynamic pathfinding during task execution
- [ ] Robot-to-robot communication
- [ ] Task dependency graphs
- [ ] Predictive robot positioning
