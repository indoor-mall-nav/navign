# Network Client (Navign)

The network component provides HTTP communication with the Navign server for pathfinding and entity data.

## Overview

**Language:** Rust
**Location:** `robot/network/`
**Protocol:** HTTP/REST + Protocol Buffers

## Responsibilities

- Pathfinding requests to server
- Entity and area data fetching
- Response caching for offline operation
- Future: BLE operations for beacon interaction

## Zenoh Integration

### Published Topics

- `robot/network/pathfinding/response` - Navigation paths
- `robot/network/entity/data` - Entity/area information

### Subscribed Topics

- `robot/network/pathfinding/request` - Pathfinding queries
- `robot/network/entity/request` - Entity data requests

## API Endpoints

- `GET /api/entities/{id}/route` - Pathfinding
- `GET /api/entities/{id}` - Entity metadata
- `GET /api/entities/{eid}/areas` - Area polygons
- `GET /api/entities/{eid}/beacons` - Beacon locations

## Running

```bash
cd robot/network
SERVER_URL=http://localhost:3000 cargo run
```

## Environment Variables

- `SERVER_URL` - Default: `http://localhost:3000`
- `ENTITY_ID` - Robot's entity ID for navigation

## See Also

- [Scheduler](scheduler.md)
- [Server API](../../server.md)
- [Protocol Buffers](/robot/proto/network.proto)
