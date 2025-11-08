# Robot Control Pipeline

The robot control pipeline orchestrates autonomous delivery robots within indoor environments, coordinating task assignment, navigation, and real-time status updates across distributed systems. The architecture employs a microservices pattern with gRPC for control plane communication and Socket.IO for real-time robot connectivity.

## Architecture Overview

The robot control system consists of three primary components:

```
Mobile App → Orchestrator (Rust gRPC) → Tower (Go Socket.IO) → Robot Fleet
```

**Orchestrator (Rust):**
- Receives task requests from mobile apps
- Maintains robot registry and state
- Executes robot selection algorithm
- Assigns tasks via gRPC streams

**Tower (Go):**
- Maintains persistent Socket.IO connections to robots
- One goroutine per robot connection
- Proxies task assignments from Orchestrator
- Aggregates status updates from robots

**Robots:**
- Connect to Tower via Socket.IO WebSocket
- Report position and status continuously
- Execute assigned tasks autonomously
- Query server for pathfinding when needed

## Task Submission Pipeline

### Stage 1: Task Request

Users initiate delivery tasks through the mobile app:

```
POST /api/tasks
{
  "type": "delivery",
  "sources": [{"x": 50, "y": 30, "floor": "1F"}],
  "terminals": [{"x": 80, "y": 60, "floor": "2F"}],
  "priority": "normal",
  "entity_id": "507f1f77bcf86cd799439011",
  "metadata": {
    "item_description": "Coffee order #1234",
    "customer_name": "Alice"
  }
}
```

**Task Types:**

```rust
enum TaskType {
    Delivery,      // Pick up from source, deliver to terminal
    Patrol,        // Navigate waypoints repeatedly
    Cleaning,      // Autonomous floor cleaning
    Escort,        // Guide user to destination
}
```

**Priority Levels:**

```rust
enum Priority {
    Low = 0,       // Background tasks, no SLA
    Normal = 1,    // Standard delivery, 15-minute SLA
    High = 2,      // Express delivery, 5-minute SLA
    Emergency = 3, // Medical supplies, immediate
}
```

Higher priority tasks preempt lower priority tasks in the assignment queue.

### Stage 2: Task Queuing (Orchestrator)

The Orchestrator receives task requests via gRPC and enqueues them:

```rust
async fn submit_task(task: Task) -> Result<TaskResponse> {
    // Validate task
    validate_locations(&task.sources, &task.terminals)?;

    // Assign unique ID
    task.id = generate_task_id();
    task.created_at = current_timestamp();
    task.status = TaskStatus::Pending;

    // Add to priority queue
    let mut queue = task_queue.write().await;
    queue.add_task(task.clone());

    Ok(TaskResponse {
        task_id: task.id,
        estimated_completion: estimate_time(&task),
    })
}
```

**Priority Queue Implementation:**

Tasks are sorted by priority (descending) and creation time (ascending):

```rust
queue.sort_by(|a, b| {
    b.priority.cmp(&a.priority)
        .then(a.created_at.cmp(&b.created_at))
});
```

This ensures high-priority tasks execute first, with FIFO ordering within priority levels.

## Robot Selection Algorithm

### Stage 3: Available Robot Discovery

The Orchestrator maintains a real-time registry of robots:

```rust
struct RobotRegistry {
    robots: HashMap<String, RobotInfo>,
    task_channels: HashMap<String, mpsc::Sender<TaskAssignment>>,
}

struct RobotInfo {
    id: String,
    entity_id: String,
    state: RobotState,
    battery_level: f32,
    current_location: Option<Location>,
    capabilities: Vec<Capability>,
}
```

**Robot States:**

```rust
enum RobotState {
    Idle,           // Available for task assignment
    Busy,           // Currently executing a task
    Charging,       // Docked at charging station
    Error,          // Hardware fault, needs maintenance
    Offline,        // Disconnected from Tower
}
```

### Stage 4: Robot Scoring

When a task is dequeued, the Orchestrator scores all eligible robots:

```rust
fn find_best_robot(&self, task: &Task) -> Option<RobotInfo> {
    let candidates: Vec<_> = self.robots.values()
        .filter(|r| r.entity_id == task.entity_id)
        .filter(|r| r.state == RobotState::Idle)
        .filter(|r| r.battery_level > 20.0)
        .collect();

    if candidates.is_empty() {
        return None;
    }

    let mut best_robot = None;
    let mut best_score = f64::MIN;

    for robot in candidates {
        let score = calculate_score(robot, task);
        if score > best_score {
            best_score = score;
            best_robot = Some(robot.clone());
        }
    }

    best_robot
}
```

**Scoring Function:**

The score combines multiple factors:

```rust
fn calculate_score(robot: &RobotInfo, task: &Task) -> f64 {
    let mut score = 0.0;

    // Battery level (0-100 points)
    score += robot.battery_level;

    // Proximity to task source (0-100 points)
    if let Some(robot_loc) = &robot.current_location {
        if let Some(task_source) = task.sources.first() {
            let distance_sq = squared_distance(robot_loc, task_source);
            score += 100.0 / (1.0 + distance_sq / 10000.0);
        }
    }

    // Capability matching (0-50 points)
    if task.requires_refrigeration() && robot.has_refrigeration() {
        score += 50.0;
    }

    score
}
```

This scoring function prioritizes:
1. Robots with higher battery (to avoid mid-task charging)
2. Robots closer to task source (reduces empty travel time)
3. Robots with required capabilities (e.g., refrigerated compartment for food delivery)

### Alternative Selection Strategies

**Round-Robin:**
Distribute tasks evenly across robots to balance wear. Ignores efficiency for fairness.

**Load Balancing:**
Prefer robots with fewer completed tasks today. Prevents overworking specific robots.

**Learning-Based:**
Use historical completion times to predict which robot will complete fastest. Requires significant data.

The current implementation uses proximity-based scoring as a reasonable heuristic.

## Task Assignment

### Stage 5: gRPC Stream Transmission

Once a robot is selected, the Orchestrator sends the task via gRPC bidirectional stream:

```rust
let task_assignment = TaskAssignment {
    task_id: task.id.clone(),
    robot_id: robot.id.clone(),
    task_type: task.r#type,
    waypoints: task.sources.iter()
        .chain(task.terminals.iter())
        .cloned()
        .collect(),
    priority: task.priority,
    deadline: task.created_at + task.sla_seconds,
};

if let Some(tx) = task_channels.get(&robot.id) {
    tx.send(Ok(task_assignment)).await?;
}
```

**Stream Management:**

Each robot has a dedicated gRPC stream maintained by Tower:

```go
func (s *Server) StreamTasks(stream pb.TaskService_StreamTasksServer) error {
    robotID := extractRobotID(stream.Context())

    // Create channel for task assignments
    taskChan := make(chan *pb.TaskAssignment)
    s.registerRobot(robotID, taskChan)
    defer s.unregisterRobot(robotID)

    // Forward tasks from channel to stream
    for task := range taskChan {
        if err := stream.Send(task); err != nil {
            return err
        }
    }
    return nil
}
```

The Go Tower acts as a bridge: gRPC stream on one side, Socket.IO connection on the other.

### Stage 6: Socket.IO Propagation

Tower forwards the task to the robot via Socket.IO:

```go
func (s *SocketServer) SendTaskToRobot(robotID string, task *TaskAssignment) error {
    conn := s.connections[robotID]
    if conn == nil {
        return errors.New("robot not connected")
    }

    conn.Emit("task_assigned", task)
    return nil
}
```

**Why Socket.IO Instead of Direct gRPC?**

Robots are resource-constrained devices (often Raspberry Pi, NVIDIA Jetson Nano). Socket.IO provides:
- Automatic reconnection on network interruptions
- Fallback to HTTP polling if WebSocket unavailable
- Lower memory overhead than gRPC client libraries
- Better compatibility with embedded systems

## Robot Execution

### Stage 7: Pathfinding Query

Upon receiving a task, the robot queries the server for a route:

```
GET /api/entities/{entity_id}/route?from={robot_x},{robot_y},{robot_area}&to={task_x},{task_y},{task_area}
```

The server returns turn-by-turn instructions (see Navigation Pipeline documentation).

**Why Server-Side Pathfinding?**

Robots lack complete map data and pathfinding compute resources. Centralizing pathfinding on the server:
- Reduces robot complexity
- Enables dynamic routing (avoid congested areas)
- Allows path optimization with global knowledge
- Simplifies robot software updates (path logic stays server-side)

### Stage 8: Autonomous Navigation

The robot executes the navigation instructions:

```python
def execute_task(task):
    waypoints = task.sources + task.terminals

    for i, waypoint in enumerate(waypoints):
        route = fetch_route(current_position, waypoint)

        for instruction in route.instructions:
            if instruction.type == "walk":
                motor_controller.move(instruction.distance, instruction.direction)
            elif instruction.type == "transport":
                # Handle elevator - wait for arrival, enter, press button
                wait_for_elevator()
                enter_elevator()
                press_floor_button(instruction.destination_floor)
                wait_for_floor(instruction.destination_floor)
                exit_elevator()

        if i < len(task.sources):
            # Arrived at pickup location
            compartment.open()
            await manual_item_placement()
            compartment.close()
        else:
            # Arrived at delivery location
            compartment.open()
            await manual_item_removal()
            compartment.close()
```

### Stage 9: Status Reporting

The robot continuously reports status to Tower:

```go
type RobotStatusPacket struct {
    RobotID         string
    TaskID          string
    Status          string  // "navigating", "waiting", "loading", "unloading"
    CurrentLocation Location
    BatteryLevel    float32
    Timestamp       int64
}
```

Tower aggregates status updates and forwards to the Orchestrator via gRPC:

```go
func (s *SocketServer) OnStatusUpdate(packet RobotStatusPacket) {
    // Forward to Orchestrator via gRPC
    s.grpcClient.UpdateTaskStatus(context.Background(), &pb.TaskUpdate{
        TaskId:   packet.TaskID,
        RobotId:  packet.RobotID,
        Status:   packet.Status,
        Location: packet.CurrentLocation,
    })
}
```

## Concurrency Model

### Tower Goroutine Architecture

Tower uses one goroutine per robot connection:

```go
func (s *SocketServer) HandleRobotConnection(socket socketio.Conn) {
    robotID := socket.ID()

    // Spawn goroutine for this robot
    go func() {
        for {
            select {
            case task := <-s.taskChannels[robotID]:
                socket.Emit("task_assigned", task)
            case <-s.shutdownChan:
                return
            }
        }
    }()

    // Register event handlers
    socket.On("status_update", s.OnStatusUpdate)
    socket.On("task_complete", s.OnTaskComplete)
    socket.On("disconnect", func() {
        s.handleDisconnect(robotID)
    })
}
```

This per-robot goroutine model provides:
- Isolation: One robot's slow connection doesn't block others
- Simplicity: No complex multiplexing logic
- Scalability: Go's goroutines are lightweight (2KB stack each)

For 100 robots, total overhead is ~200KB, acceptable on modern servers.

### Orchestrator Thread Safety

The Rust Orchestrator uses async/await with Tokio runtime:

```rust
#[tokio::main]
async fn main() {
    let registry = Arc::new(RwLock::new(RobotRegistry::new()));

    // Spawn task assignment loop
    let reg_clone = Arc::clone(&registry);
    tokio::spawn(async move {
        task_assignment_loop(reg_clone).await;
    });

    // Start gRPC server
    Server::builder()
        .add_service(OrchestratorServiceServer::new(MyService { registry }))
        .serve(addr)
        .await?;
}
```

**Lock Granularity:**

The registry uses `RwLock` to allow concurrent reads (multiple status updates) while serializing writes (task assignments):

```rust
// Many robots can read state concurrently
let robots = registry.read().await;
let robot = robots.get(&robot_id);

// Only one task assignment at a time
let mut robots = registry.write().await;
robots.assign_task(task);
```

This balances concurrency (high throughput for status updates) with consistency (task assignments don't race).

## Failure Handling

### Robot Disconnection

If a robot disconnects mid-task:

```go
func (s *SocketServer) handleDisconnect(robotID string) {
    // Mark robot as offline
    s.grpcClient.UpdateRobotState(context.Background(), &pb.RobotState{
        RobotId: robotID,
        State:   pb.State_Offline,
    })

    // Re-queue active task
    if task := s.getActiveTask(robotID); task != nil {
        s.grpcClient.RequeueTask(context.Background(), task)
    }
}
```

The task returns to the queue with increased priority to compensate for delay.

### Task Timeout

If a robot doesn't complete a task within the SLA:

```rust
if current_time - task.created_at > task.sla_seconds {
    // Cancel task on robot
    cancel_task(task.robot_id, task.id);

    // Re-queue for different robot
    task.retry_count += 1;
    task.priority = Priority::High;  // Escalate priority
    queue.add_task(task);
}
```

### Battery Depletion

If a robot's battery drops below 15% during task execution:

```python
if battery_level < 15:
    # Return to charging station
    route = fetch_route(current_position, nearest_charging_station)
    execute_navigation(route)
    dock_at_charger()

    # Report task abandonment
    report_task_abandoned(task_id, reason="low_battery")
```

The Orchestrator re-assigns the task to another robot.

## Monitoring and Observability

The system exposes metrics for operational monitoring:

**Orchestrator Metrics:**
- Tasks queued by priority
- Average task assignment latency
- Robot state distribution (idle/busy/charging/error)
- Task success/failure rate

**Tower Metrics:**
- Active robot connections
- Message throughput (tasks/second, status updates/second)
- gRPC stream health
- Socket.IO connection duration

**Robot Metrics:**
- Task completion time (P50, P95, P99)
- Navigation accuracy (deviation from planned route)
- Battery consumption per task
- Hardware fault frequency

These metrics enable:
- Capacity planning (add robots if queue grows)
- Performance optimization (identify slow robots)
- Predictive maintenance (detect degrading robots)

## Related Documentation

- [Orchestrator gRPC API](/components/admin/orchestrator)
- [Tower Socket.IO Implementation](/components/admin/tower)
- [Navigation Pipeline](/pipelines/navigation)
- [Robot Hardware Specifications](/components/robot)
