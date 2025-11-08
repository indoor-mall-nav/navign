# Navigation Pipeline

The navigation pipeline transforms a user's start and end positions into actionable turn-by-turn instructions, handling the complexity of multi-floor indoor environments with elevators, stairs, and escalators. Unlike outdoor navigation which operates on pre-defined road networks, indoor navigation must reason about arbitrary polygon geometries, vertical transitions, and accessibility constraints.

## Pipeline Architecture

The navigation system employs a hybrid approach: computationally intensive pathfinding executes on the server, while the mobile app handles instruction interpretation and visual rendering. This division reflects practical constraints—mobile devices have limited battery for sustained CPU-intensive operations, while the server can dedicate resources to complex graph algorithms.

```
User Request → Position Resolution → Server Pathfinding → Instruction Generation → Mobile Rendering
```

## Stage 1: Position Resolution

Navigation requests can specify locations in two formats:

**Explicit Coordinates:**
```
from: "45.5,67.3,507f191e810c19729de860ea"
to: "23.1,89.4,507f191e810c19729de860eb"
```
Format: `<x>,<y>,<area_id>`

**Merchant References:**
```
from: "507f1f77bcf86cd799439011"  (current_user_position special value)
to: "507f1f77bcf86cd799439012"    (merchant ObjectId)
```

The mobile resolves merchant IDs to coordinates via local cache:
```sql
SELECT location_x, location_y, area FROM merchants WHERE id = ?
```

This two-format system balances flexibility (users can navigate to any coordinate) with usability (users typically navigate to named destinations like "Starbucks" rather than abstract coordinates).

**Current Position Handling:**

The special value `"current"` or user position from localization pipeline resolves to:
```
(localization.x, localization.y, localization.area)
```

If localization is unavailable (no beacons detected), navigation fails with "position unavailable" error.

## Stage 2: Server Pathfinding Request

The mobile constructs a RESTful request to the server:

```
GET /api/entities/{entity_id}/route?from={source}&to={destination}&disallow={constraints}
```

**Query Parameters:**

`from`: Source position (format: `x,y,area_id` or `merchant_id`)
`to`: Destination position (same format)
`disallow`: Optional string encoding disabled connection types
  - `'e'`: Disable elevators
  - `'s'`: Disable stairs
  - `'c'`: Disable escalators
  - Example: `"esc"` disables all vertical transitions (same-floor only)

The `disallow` parameter enables accessibility routing. A wheelchair user would set `disallow="sc"` (stairs and escalators disabled), forcing the router to use only elevators and same-floor paths.

**Request Example:**
```
GET /api/entities/507f1f77bcf86cd799439011/route?from=10.5,20.3,507f191e810c19729de860ea&to=507f191e810c19729de860eb&disallow=c
```

This requests a route avoiding escalators.

## Stage 3: Graph Construction (Server)

The server begins by loading all areas and connections for the entity from MongoDB:

```javascript
areas = db.areas.find({ entity: entity_id })
connections = db.connections.find({ entity: entity_id })
merchants = db.merchants.find({ entity: entity_id })
```

These documents are converted into a connectivity graph where:
- **Nodes**: Areas (each floor's rooms, hallways, zones)
- **Edges**: Connections (elevators, stairs, escalators linking areas)

**Connection Types:**

Connections represent vertical transitions between floors or horizontal passages between separated areas:

```rust
enum ConnectionType {
    Elevator,
    Stairs,
    Escalator,
}
```

Each connection specifies entry/exit points in connected areas:

```rust
struct ConnectedArea {
    area: ObjectId,
    x: f64,           // Entry point x coordinate
    y: f64,           // Entry point y coordinate
    floor: String,    // Floor identifier (e.g., "1F", "B2")
}
```

For example, an elevator connecting Floor 1 to Floor 2 might specify:
- Area A (Floor 1): entry at (50, 30)
- Area B (Floor 2): exit at (51, 31)

The slight coordinate difference reflects the elevator's physical layout—you enter from one side and exit from another.

## Stage 4: Dijkstra's Algorithm

With the graph constructed, the server applies Dijkstra's shortest path algorithm to find the optimal route from source area to destination area.

**Connectivity Limits:**

The `disallow` parameter filters edges during graph construction:

```rust
let limits = ConnectivityLimits {
    elevator: !disallow.contains('e'),
    stairs: !disallow.contains('s'),
    escalator: !disallow.contains('c'),
};
```

Connections of disabled types are excluded from the graph, making them effectively unreachable.

**Edge Weights:**

Each connection has a weight representing traversal cost. The current implementation uses uniform weights (all connections cost 1), but future enhancements could incorporate:
- Physical distance between area centers
- Elevator wait times (elevators cost more than stairs)
- Stair difficulty (multiple floors cost more than single floor)
- Accessibility preferences (prefer ramps over stairs for certain users)

**Algorithm Execution:**

Dijkstra's maintains:
- Priority queue of unexplored nodes (ordered by cumulative distance from source)
- Distance table mapping each node to shortest known distance from source
- Predecessor table tracking optimal path

The algorithm iteratively:
1. Extract minimum-distance node from priority queue
2. For each neighbor, calculate distance via current node
3. If shorter than known distance, update distance and predecessor
4. Add neighbor to priority queue

Termination occurs when the destination node is extracted from the priority queue, guaranteeing an optimal path has been found.

**Bump Allocation:**

The server uses `bumpalo` bump allocator for graph nodes and edges. This arena-based allocation strategy eliminates per-node malloc/free overhead:

```rust
let arena = Bump::new();
let graph = build_graph(&arena, entity, areas, connections);
let path = dijkstra(&arena, graph, source, destination);
```

All allocations come from a contiguous memory region. When the arena drops, all memory is freed at once. This reduces pathfinding time by ~30% compared to standard allocations.

**Asymptotic Complexity:**

For V areas and E connections:
- Time: O((V + E) log V) (priority queue operations)
- Space: O(V + E) (graph storage)

Typical indoor entities have V ~ 50-200 areas and E ~ 100-500 connections, yielding sub-10ms pathfinding times.

## Stage 5: Path Decomposition

Dijkstra's returns a sequence of areas:
```
[area_1, area_2, area_3, ..., area_n]
```

The server decomposes this into segments, where each segment represents movement within a single area followed by a connection transition.

**Segment Structure:**

For each adjacent pair `(area_i, area_i+1)`:
1. Find the connection linking them
2. Extract entry point in `area_i` from connection metadata
3. Extract exit point (entry point in `area_i+1`)
4. Find path within `area_i` from current position to connection entry point
5. Add connection transition instruction
6. Update current position to connection exit point in `area_i+1`

**Within-Area Pathfinding:**

Each area has a polygon boundary representing navigable space. The server finds obstacle-avoiding paths using one of two methods:

**Visibility Graph (Not Yet Implemented):**
Construct a graph where nodes are polygon vertices and edges are visible (unobstructed) segments. This produces optimal paths but requires O(V²) visibility checks for V vertices.

**Polygon-Based Displacement:**
Current implementation uses a simpler approach: the polygon is converted to a grid-based or bounded block representation, and A* or similar search finds a path. This is faster to compute but produces sub-optimal (longer) paths.

Future implementation will use visibility graphs for better path quality.

## Stage 6: Instruction Generation

The server converts the geometric path into human-readable instructions:

```rust
enum InstructionType {
    Walk { distance: f64, direction: String },
    Transport {
        connection_id: String,
        destination_area: String,
        transport_type: ConnectionType,
    },
    EnterArea { area_id: String, name: String },
    ExitArea { area_id: String },
}
```

**Walk Instructions:**

Generated for movement within areas. The distance is Euclidean distance between waypoints, and direction is cardinal (North, South, East, West, etc.) or relative (forward, left, right).

**Transport Instructions:**

Generated at connection transitions:
```json
{
  "type": "Transport",
  "connection_id": "507f1f77bcf86cd799439013",
  "destination_area": "507f1f77bcf86cd799439014",
  "transport_type": "Elevator"
}
```

The mobile UI interprets this as "Take elevator to Floor 2" (extracting floor from destination area metadata).

**Instruction Merging:**

Consecutive walk instructions in the same direction merge:
```
Walk 5m north + Walk 3m north → Walk 8m north
```

This reduces instruction count and improves user experience.

## Stage 7: Mobile Instruction Parsing

The mobile receives JSON instructions and converts them into UI-renderable format:

```typescript
interface RenderedInstruction {
  text: string;           // "Walk 15 meters north"
  icon: string;           // Icon name for UI
  distance?: number;      // For progress tracking
  type: 'walk' | 'transport' | 'arrive';
}
```

**Localization:**

Instruction text supports i18n (internationalization):
```typescript
const instructionText = {
  en: "Take elevator to Floor 2",
  zh: "乘坐电梯到2楼",
  ja: "エレベーターで2階へ"
}[userLanguage];
```

**Icon Selection:**

Each instruction type maps to an icon:
- Walk: Arrow icon
- Elevator: Elevator icon
- Stairs: Stairs icon
- Escalator: Escalator icon
- Arrive: Checkmark icon

## Stage 8: Visual Route Rendering

The mobile renders the route on the map using Konva canvas overlays:

**Waypoint Transformation:**

Server instructions include waypoint coordinates in local coordinate systems. The mobile transforms these to screen coordinates:

```typescript
function localToScreen(x: number, y: number, area: Area): Point {
  const mapBounds = getMapBounds(area);
  const screenX = (x - mapBounds.minX) / (mapBounds.maxX - mapBounds.minX) * screenWidth;
  const screenY = (y - mapBounds.minY) / (mapBounds.maxY - mapBounds.minY) * screenHeight;
  return { x: screenX, y: screenY };
}
```

**Polyline Rendering:**

The route is drawn as a styled polyline:
```typescript
const routeLine = new Konva.Line({
  points: waypoints.flatMap(p => [p.x, p.y]),
  stroke: '#0066FF',
  strokeWidth: 4,
  lineCap: 'round',
  lineJoin: 'round',
});
```

**Directional Arrows:**

Every 10 meters along the route, an arrow is rendered pointing in the direction of travel. This provides visual flow indication.

**Current Position Marker:**

A pulsing circle marks the user's current position, updated every 5 seconds via the localization pipeline.

## Stage 9: Real-Time Navigation

As the user moves, the mobile tracks progress:

**Progress Calculation:**

For each walk instruction, the distance from current position to instruction's endpoint is compared to the instruction's total distance:
```typescript
const progress = 1 - (remainingDistance / totalDistance);
```

When progress exceeds 90%, the UI highlights the next instruction.

**Off-Route Detection:**

If the user's position deviates >5 meters from the route polyline, the system triggers re-routing:
1. Calculate new route from current position to original destination
2. Animate transition from old route to new route
3. Update instructions

This handles cases where users take wrong turns or deliberately deviate from suggested routes.

**Floor Transition Handling:**

When the user approaches a connection (within 3 meters), the UI prompts:
"Approaching elevator. Prepare to go to Floor 2."

After passing through (detected via area change in localization), the UI updates the displayed floor plan.

## Offline Mode

If the server is unreachable, the mobile attempts local pathfinding using cached data:

**Limitations:**

Local pathfinding is limited to same-floor navigation (no connection graph traversal). The mobile can find paths within a single area using cached polygon geometry, but cannot route across floors.

**Fallback Strategy:**

1. Check if source and destination are in the same area
2. If yes, perform local polygon-based pathfinding
3. If no, display error: "Multi-floor navigation requires network connectivity"

This degraded mode maintains basic functionality during network outages while clearly communicating limitations.

## Performance Optimization

**Server-Side:**

- Bump allocation reduces pathfinding latency by ~30%
- Spawn blocking offloads CPU-intensive pathfinding from async runtime
- Connection pooling prevents MongoDB connection overhead

**Mobile-Side:**

- Instruction rendering happens on canvas, not DOM (faster updates)
- Route polyline uses single Konva shape (not per-segment shapes)
- Position updates throttled to 10 FPS (reduces GPU usage)

**Network:**

- Route requests return full instruction list in single response (no pagination)
- Instructions include all necessary metadata (no follow-up requests)
- Gzip compression reduces payload size by ~70%

## Future Enhancements

- **Dynamic routing**: Incorporate real-time crowd density (avoid congested areas)
- **Multi-modal routing**: Combine indoor and outdoor navigation seamlessly
- **Landmark-based instructions**: "Turn right at Starbucks" instead of "Turn right in 15 meters"
- **Voice guidance**: Audio instructions for hands-free navigation
- **AR visualization**: Overlay arrows in camera view using ARKit/ARCore

## Related Documentation

- [Server Pathfinding Implementation](/components/server#pathfinding-system)
- [Localization Pipeline](/pipelines/localization)
- [Mobile Route Rendering](/components/mobile#navigation-and-routing)
