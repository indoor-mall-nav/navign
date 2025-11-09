# Orchestrator-Central Server Communication Protocol

## Overview

This document specifies the event-driven communication protocol between local mall orchestrators and the central Navign server. The protocol is designed for:

- **Firewall-friendly:** Orchestrator initiates all connections
- **Event-driven:** Connect only when needed, no persistent connections
- **Resilient:** Handles network interruptions and GitHub accessibility issues
- **Secure:** Mutual authentication and encrypted channels
- **Scalable:** Supports thousands of mall orchestrators

## Architecture

```
┌──────────────┐
│   Central    │
│   Server     │◄─── HTTPS/SSE ───┐
│  (Cloud)     │                  │
└──────────────┘                  │
       │                          │
       │ REST API                 │
       │                          │
┌──────▼───────┐          ┌───────┴──────┐
│   Mobile     │          │ Orchestrator │
│     App      │          │  (Mall A)    │
└──────────────┘          └───────┬──────┘
                                  │
                          ┌───────┼───────┐
                          │       │       │
                      ┌───▼──┐ ┌──▼──┐ ┌─▼────┐
                      │Beacon│ │Tower│ │Robot │
                      └──────┘ └─────┘ └──────┘
```

### Communication Paths

1. **Mobile ↔ Central Server:** Direct HTTPS (existing)
2. **Orchestrator ↔ Central Server:** Event-driven HTTPS + SSE (new)
3. **Robot ↔ Orchestrator:** gRPC streaming (existing)
4. **Beacon ↔ Orchestrator:** HTTPS REST (new)
5. **Mobile ↔ Orchestrator:** **BLOCKED** (must go through central)

## Protocol Components

### 1. Orchestrator Registration

**Endpoint:** `POST /api/orchestrators/register`

**Request:**
```json
{
  "entity_id": "mall-a-uuid",
  "orchestrator_id": "orch-001",
  "version": "0.1.0",
  "capabilities": [
    "task_assignment",
    "firmware_distribution",
    "beacon_management"
  ],
  "public_key": "-----BEGIN PUBLIC KEY-----\n...",
  "heartbeat_interval": 60,
  "local_address": "https://orchestrator.mall-a.local:50051"
}
```

**Response:**
```json
{
  "token": "jwt-token-for-orchestrator",
  "expires_at": 1735862400,
  "assigned_id": "orch-mall-a-001",
  "sync_endpoints": {
    "events": "https://server.navign.com/api/orchestrators/events",
    "data_sync": "https://server.navign.com/api/orchestrators/sync",
    "firmware": "https://server.navign.com/api/firmware"
  },
  "initial_sync_required": true
}
```

**Authentication:** ECDSA P-256 signature in `X-Signature` header
**Rate Limit:** 1 request per 24 hours per orchestrator

---

### 2. Event Subscription (Server-Sent Events)

**Endpoint:** `GET /api/orchestrators/events`

**Headers:**
```
Authorization: Bearer {orchestrator_token}
Accept: text/event-stream
X-Entity-ID: mall-a-uuid
X-Orchestrator-ID: orch-mall-a-001
```

**Event Types:**

#### 2.1 Data Update Event
```
event: data_update
id: evt-123456
data: {
  "type": "entity_update",
  "entity_id": "mall-a-uuid",
  "updated_at": "2025-01-08T10:30:00Z",
  "changes": {
    "areas": ["area-1", "area-2"],
    "beacons": ["beacon-3"],
    "merchants": [],
    "connections": ["conn-5"]
  },
  "checksum": "sha256:abc123..."
}
```

**Orchestrator Action:**
1. Fetch changed data via `/api/orchestrators/sync/delta`
2. Update local cache
3. Notify affected beacons/robots
4. Send acknowledgment

#### 2.2 Firmware Update Event
```
event: firmware_update
id: evt-123457
data: {
  "type": "beacon_firmware",
  "target": "esp32c3",
  "version": "0.2.0",
  "release_date": "2025-01-08T00:00:00Z",
  "firmware_id": "fw-beacon-v020",
  "size_bytes": 524288,
  "checksum": "sha256:def456...",
  "download_url": "https://server.navign.com/api/firmware/fw-beacon-v020/download",
  "fallback_url": "https://cdn.navign.com/firmware/beacon-v020.bin",
  "priority": "normal",
  "rollout_strategy": "canary"
}
```

**Orchestrator Action:**
1. Download firmware to local cache
2. Verify checksum
3. Announce to beacons via mDNS/BLE broadcast
4. Serve firmware to beacons at `/firmwares/{id}/download`
5. Report update status to central server

#### 2.3 Task Creation Event
```
event: task_create
id: evt-123458
data: {
  "task_id": "task-delivery-001",
  "type": "delivery",
  "entity_id": "mall-a-uuid",
  "priority": "high",
  "source": {
    "area_id": "area-merchant-5",
    "coordinates": {"x": 100.5, "y": 50.2, "z": 0, "floor": "L1"}
  },
  "destination": {
    "area_id": "area-entrance-3",
    "coordinates": {"x": 200.5, "y": 150.2, "z": 0, "floor": "L2"}
  },
  "payload": {
    "order_id": "order-789",
    "customer_phone": "+1234567890"
  },
  "created_at": "2025-01-08T10:35:00Z"
}
```

**Orchestrator Action:**
1. Add task to local queue
2. Run robot selection algorithm
3. Assign to robot via Tower/gRPC
4. Report assignment to central server

#### 2.4 Access Log Request
```
event: access_log_request
id: evt-123459
data: {
  "beacon_id": "beacon-gate-5",
  "since": "2025-01-08T00:00:00Z",
  "request_id": "req-001"
}
```

**Orchestrator Action:**
1. Query local access logs
2. Upload to central server via `/api/orchestrators/logs/upload`

#### 2.5 Configuration Update
```
event: config_update
id: evt-123460
data: {
  "type": "robot_selection_params",
  "config": {
    "min_battery_threshold": 25,
    "max_distance_meters": 500,
    "priority_weights": {
      "distance": 0.6,
      "battery": 0.3,
      "queue_length": 0.1
    }
  },
  "version": 2
}
```

**Orchestrator Action:**
1. Validate configuration
2. Apply to local orchestrator
3. Send acknowledgment

#### 2.6 Connection Keep-Alive
```
event: ping
id: evt-123461
data: {"timestamp": "2025-01-08T10:40:00Z"}
```

**Orchestrator Action:**
1. Respond with pong (no action needed, SSE handles this)

---

### 3. Data Synchronization

#### 3.1 Initial Full Sync

**Endpoint:** `GET /api/orchestrators/sync/full`

**Query Parameters:**
```
?entity_id=mall-a-uuid
&include=areas,beacons,merchants,connections
&format=compact
```

**Response:**
```json
{
  "entity_id": "mall-a-uuid",
  "sync_id": "sync-full-001",
  "timestamp": "2025-01-08T10:30:00Z",
  "checksum": "sha256:abc123...",
  "data": {
    "areas": [...],
    "beacons": [...],
    "merchants": [...],
    "connections": [...]
  },
  "metadata": {
    "total_areas": 150,
    "total_beacons": 300,
    "total_merchants": 80,
    "total_connections": 50
  }
}
```

**When to use:**
- Initial orchestrator startup
- After prolonged disconnection (> 24 hours)
- After checksum mismatch
- Manual resync requested

#### 3.2 Delta Sync

**Endpoint:** `GET /api/orchestrators/sync/delta`

**Query Parameters:**
```
?entity_id=mall-a-uuid
&since=2025-01-08T10:00:00Z
&cursor=cursor-abc123
```

**Response:**
```json
{
  "entity_id": "mall-a-uuid",
  "sync_id": "sync-delta-002",
  "timestamp": "2025-01-08T10:30:00Z",
  "base_checksum": "sha256:abc123...",
  "new_checksum": "sha256:def456...",
  "changes": [
    {
      "type": "area",
      "operation": "update",
      "id": "area-1",
      "data": {...},
      "updated_at": "2025-01-08T10:25:00Z"
    },
    {
      "type": "beacon",
      "operation": "create",
      "id": "beacon-new",
      "data": {...},
      "updated_at": "2025-01-08T10:28:00Z"
    },
    {
      "type": "merchant",
      "operation": "delete",
      "id": "merchant-old",
      "updated_at": "2025-01-08T10:20:00Z"
    }
  ],
  "has_more": false,
  "next_cursor": null
}
```

**Operations:**
- `create`: New entity added
- `update`: Existing entity modified
- `delete`: Entity removed

#### 3.3 Checksum Verification

**Endpoint:** `GET /api/orchestrators/sync/checksum`

**Query Parameters:**
```
?entity_id=mall-a-uuid
```

**Response:**
```json
{
  "entity_id": "mall-a-uuid",
  "timestamp": "2025-01-08T10:30:00Z",
  "checksums": {
    "areas": "sha256:aaa111...",
    "beacons": "sha256:bbb222...",
    "merchants": "sha256:ccc333...",
    "connections": "sha256:ddd444...",
    "global": "sha256:eee555..."
  }
}
```

**Orchestrator Action:**
1. Calculate local checksums
2. Compare with server checksums
3. Trigger delta sync if mismatch
4. Trigger full sync if multiple mismatches

---

### 4. Firmware Distribution

#### 4.1 Firmware Metadata Query

**Endpoint:** `GET /api/firmware/latest`

**Query Parameters:**
```
?target=esp32c3
&device_type=beacon
&channel=stable
```

**Response:**
```json
{
  "firmware_id": "fw-beacon-v020",
  "version": "0.2.0",
  "target": "esp32c3",
  "device_type": "beacon",
  "channel": "stable",
  "release_date": "2025-01-08T00:00:00Z",
  "size_bytes": 524288,
  "checksum": "sha256:def456...",
  "download_url": "https://server.navign.com/api/firmware/fw-beacon-v020/download",
  "fallback_url": "https://cdn.navign.com/firmware/beacon-v020.bin",
  "release_notes": "https://github.com/navign/navign/releases/tag/v0.2.0",
  "changelog": [
    "Fixed nonce expiration bug",
    "Added OTA rollback support",
    "Improved BLE range"
  ],
  "compatible_hardware": ["esp32c3-rev3", "esp32c3-rev4"],
  "min_previous_version": "0.1.5"
}
```

#### 4.2 Firmware Download

**Endpoint:** `GET /api/firmware/{firmware_id}/download`

**Headers:**
```
Authorization: Bearer {orchestrator_token}
X-Entity-ID: mall-a-uuid
Range: bytes=0-1048575
```

**Response:**
- Binary firmware file
- Supports HTTP range requests for resumable downloads
- Content-Type: `application/octet-stream`
- Content-Disposition: `attachment; filename="beacon-v020.bin"`

**Orchestrator Action:**
1. Download firmware to local cache directory
2. Verify checksum
3. Store metadata (version, target, checksum)
4. Make available at local endpoint: `http://orchestrator.local/firmwares/{id}/download`

#### 4.3 Firmware Upload (Cache Miss)

**Endpoint:** `POST /api/orchestrators/firmware/cache`

**Request (Multipart Form):**
```
firmware_id: fw-beacon-v020
target: esp32c3
file: (binary firmware)
```

**Response:**
```json
{
  "cached": true,
  "firmware_id": "fw-beacon-v020",
  "expires_at": "2025-02-08T10:30:00Z"
}
```

**When to use:**
- Orchestrator received firmware from alternative source (USB, local build)
- GitHub not accessible, used fallback CDN
- Manual firmware upload

#### 4.4 Firmware Update Status Report

**Endpoint:** `POST /api/orchestrators/firmware/status`

**Request:**
```json
{
  "entity_id": "mall-a-uuid",
  "firmware_id": "fw-beacon-v020",
  "rollout_id": "rollout-001",
  "beacon_id": "beacon-gate-5",
  "status": "success",
  "previous_version": "0.1.8",
  "new_version": "0.2.0",
  "updated_at": "2025-01-08T11:00:00Z",
  "duration_seconds": 45,
  "error": null
}
```

**Status Values:**
- `downloading`: Beacon downloading firmware
- `verifying`: Beacon verifying checksum
- `installing`: Beacon writing to OTA partition
- `success`: Update completed, beacon rebooted
- `failed`: Update failed (see error field)
- `rolled_back`: Beacon rolled back to previous version

**Error Types:**
- `download_failed`: Network error during download
- `checksum_mismatch`: Downloaded file corrupted
- `insufficient_space`: Flash partition too small
- `incompatible_hardware`: Firmware not compatible
- `boot_failed`: New firmware failed to boot (auto rollback)

---

### 5. Task Management

#### 5.1 Task Assignment Report

**Endpoint:** `POST /api/orchestrators/tasks/assign`

**Request:**
```json
{
  "task_id": "task-delivery-001",
  "entity_id": "mall-a-uuid",
  "robot_id": "robot-007",
  "assigned_at": "2025-01-08T10:36:00Z",
  "estimated_duration_seconds": 180,
  "path": {
    "distance_meters": 150.5,
    "floors": ["L1", "L2"],
    "waypoints": 12
  }
}
```

**Response:**
```json
{
  "acknowledged": true,
  "task_id": "task-delivery-001",
  "tracking_url": "https://server.navign.com/tasks/task-delivery-001/track"
}
```

#### 5.2 Task Status Update

**Endpoint:** `POST /api/orchestrators/tasks/status`

**Request:**
```json
{
  "task_id": "task-delivery-001",
  "entity_id": "mall-a-uuid",
  "robot_id": "robot-007",
  "status": "in_progress",
  "progress_percent": 45,
  "current_location": {
    "x": 125.5,
    "y": 80.2,
    "z": 0,
    "floor": "L1"
  },
  "timestamp": "2025-01-08T10:38:00Z",
  "estimated_completion": "2025-01-08T10:39:00Z"
}
```

**Status Values:**
- `queued`: Waiting for robot assignment
- `assigned`: Robot assigned, not started
- `in_progress`: Robot executing task
- `completed`: Task finished successfully
- `failed`: Task failed (see error field)
- `cancelled`: Task cancelled by user/system

#### 5.3 Task Completion Report

**Endpoint:** `POST /api/orchestrators/tasks/complete`

**Request:**
```json
{
  "task_id": "task-delivery-001",
  "entity_id": "mall-a-uuid",
  "robot_id": "robot-007",
  "status": "completed",
  "completed_at": "2025-01-08T10:39:15Z",
  "duration_seconds": 195,
  "actual_path": {
    "distance_meters": 152.3,
    "deviation_meters": 5.2
  },
  "delivery_proof": {
    "photo_url": "https://orchestrator.local/media/proof-001.jpg",
    "signature": "base64-encoded-signature"
  }
}
```

---

### 6. Beacon Management

#### 6.1 Beacon Registration (via Orchestrator)

**Endpoint:** `POST /api/orchestrators/beacons/register`

**Request:**
```json
{
  "entity_id": "mall-a-uuid",
  "device_id": "aabbccddeeff00112233",
  "device_type": "turnstile",
  "capabilities": ["unlock_gate", "environmental_data"],
  "public_key": "-----BEGIN PUBLIC KEY-----\n...",
  "firmware_version": "0.1.8",
  "hardware_revision": "esp32c3-rev3",
  "location": {
    "area_id": "area-gate-1",
    "coordinates": {"x": 50.0, "y": 50.0, "z": 1.5, "floor": "L1"}
  },
  "registered_at": "2025-01-08T10:00:00Z"
}
```

**Response:**
```json
{
  "beacon_id": "beacon-gate-5",
  "entity_id": "mall-a-uuid",
  "approved": true,
  "sync_interval_seconds": 3600,
  "firmware_update_available": true,
  "latest_firmware": {
    "firmware_id": "fw-beacon-v020",
    "version": "0.2.0",
    "download_url": "https://orchestrator.local/firmwares/fw-beacon-v020/download"
  }
}
```

#### 6.2 Beacon Heartbeat

**Endpoint:** `POST /api/orchestrators/beacons/heartbeat`

**Request:**
```json
{
  "beacon_id": "beacon-gate-5",
  "entity_id": "mall-a-uuid",
  "status": "online",
  "uptime_seconds": 86400,
  "battery_percent": null,
  "firmware_version": "0.2.0",
  "metrics": {
    "unlock_attempts_24h": 120,
    "successful_unlocks_24h": 118,
    "failed_unlocks_24h": 2,
    "temperature_celsius": 28.5,
    "humidity_percent": 65.0,
    "free_heap_bytes": 102400
  },
  "timestamp": "2025-01-08T11:00:00Z"
}
```

**Response:**
```json
{
  "acknowledged": true,
  "next_heartbeat_seconds": 3600,
  "commands": [
    {
      "type": "firmware_update",
      "firmware_id": "fw-beacon-v021",
      "priority": "low"
    }
  ]
}
```

#### 6.3 Access Log Upload

**Endpoint:** `POST /api/orchestrators/beacons/access-logs`

**Request:**
```json
{
  "beacon_id": "beacon-gate-5",
  "entity_id": "mall-a-uuid",
  "logs": [
    {
      "timestamp": "2025-01-08T10:30:00Z",
      "user_id": "user-123",
      "device_id": "mobile-abc",
      "result": "success",
      "method": "ecdsa_proof",
      "duration_ms": 150
    },
    {
      "timestamp": "2025-01-08T10:35:00Z",
      "user_id": "user-456",
      "device_id": "mobile-def",
      "result": "failed",
      "method": "ecdsa_proof",
      "error": "signature_invalid",
      "duration_ms": 50
    }
  ],
  "batch_id": "batch-001",
  "total_logs": 2
}
```

**Response:**
```json
{
  "acknowledged": true,
  "batch_id": "batch-001",
  "stored": 2,
  "duplicates": 0
}
```

---

### 7. Connection Management

#### 7.1 SSE Reconnection Strategy

**Initial Connection:**
1. Orchestrator connects to `/api/orchestrators/events`
2. Server sends `event: connected` with connection ID
3. Orchestrator stores connection ID and last event ID

**On Disconnection:**
1. Wait 5 seconds
2. Reconnect with `Last-Event-ID` header
3. Server replays missed events from last ID
4. Exponential backoff: 5s → 10s → 20s → 40s → 60s (max)

**Timeout Handling:**
- Server sends `event: ping` every 30 seconds
- Orchestrator must reconnect if no event received in 90 seconds

#### 7.2 Heartbeat Mechanism

**Endpoint:** `POST /api/orchestrators/heartbeat`

**Request:**
```json
{
  "orchestrator_id": "orch-mall-a-001",
  "entity_id": "mall-a-uuid",
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 86400,
  "connected_robots": 12,
  "active_tasks": 3,
  "pending_tasks": 1,
  "metrics": {
    "cpu_percent": 25.5,
    "memory_used_mb": 512,
    "disk_used_gb": 2.5,
    "cache_size_mb": 100
  },
  "timestamp": "2025-01-08T11:00:00Z"
}
```

**Response:**
```json
{
  "acknowledged": true,
  "server_time": "2025-01-08T11:00:01Z",
  "next_heartbeat_seconds": 60,
  "commands": []
}
```

**Heartbeat Interval:** 60 seconds (configurable)
**Timeout:** Orchestrator marked offline after 5 missed heartbeats (5 minutes)

#### 7.3 Connection Recovery

**Scenario: Orchestrator offline > 24 hours**

1. Orchestrator reconnects
2. Server detects stale state
3. Server sends `event: resync_required`
4. Orchestrator performs full sync
5. Orchestrator validates local cache
6. Orchestrator resumes normal operation

**Scenario: Network partition**

1. Orchestrator continues local operations
2. Robots still function (pathfinding, task execution)
3. Beacons still function (access control)
4. Orchestrator queues events for upload
5. On reconnection, orchestrator uploads queued events in batches

---

### 8. Security

#### 8.1 Authentication

**Orchestrator Authentication:**
- JWT token issued during registration
- Token rotation every 7 days
- ECDSA P-256 signature for registration

**Beacon Authentication (to Orchestrator):**
- Device ID + ECDSA signature
- Nonce-based challenge-response
- Rate limiting: 10 requests per minute per beacon

**Robot Authentication (to Orchestrator):**
- Handled by existing gRPC/Tower mechanism
- JWT token or mTLS (to be specified)

#### 8.2 Authorization

**Role-Based Access Control:**

| Role | Permissions |
|------|-------------|
| Orchestrator | Read entity data, write logs, download firmware, report task status |
| Beacon | Register, heartbeat, upload logs, download firmware |
| Robot | Receive tasks, report status, query pathfinding |
| Mobile | Create tasks, query entity data, unlock beacons |

**Entity Isolation:**
- Orchestrators can only access data for their assigned entity
- Cross-entity access blocked at API level

#### 8.3 Data Encryption

**In Transit:**
- TLS 1.3 for all HTTPS connections
- mTLS for gRPC (optional, recommended)

**At Rest:**
- Orchestrator local cache: Encrypted with AES-256-GCM
- Beacon firmware cache: Signed with ECDSA
- Access logs: Encrypted before upload

#### 8.4 Rate Limiting

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/api/orchestrators/register` | 1 | 24 hours |
| `/api/orchestrators/heartbeat` | 1 | 60 seconds |
| `/api/orchestrators/sync/delta` | 10 | 1 minute |
| `/api/orchestrators/firmware/status` | 100 | 1 minute |
| `/api/orchestrators/tasks/*` | 100 | 1 minute |
| `/api/orchestrators/beacons/*` | 1000 | 1 minute |

---

### 9. Error Handling

#### 9.1 Error Response Format

```json
{
  "error": {
    "code": "SYNC_CHECKSUM_MISMATCH",
    "message": "Local checksum does not match server checksum",
    "details": {
      "local_checksum": "sha256:abc123...",
      "server_checksum": "sha256:def456...",
      "suggested_action": "perform_full_sync"
    },
    "timestamp": "2025-01-08T11:00:00Z",
    "request_id": "req-001"
  }
}
```

#### 9.2 Error Codes

| Code | Description | Suggested Action |
|------|-------------|------------------|
| `AUTH_TOKEN_EXPIRED` | JWT token expired | Re-register orchestrator |
| `AUTH_INVALID_SIGNATURE` | ECDSA signature invalid | Check private key |
| `SYNC_CHECKSUM_MISMATCH` | Data integrity error | Perform full sync |
| `FIRMWARE_NOT_FOUND` | Requested firmware not available | Check firmware ID |
| `ENTITY_NOT_FOUND` | Entity does not exist | Verify entity ID |
| `RATE_LIMIT_EXCEEDED` | Too many requests | Wait and retry |
| `BEACON_NOT_REGISTERED` | Beacon unknown to system | Register beacon first |
| `TASK_ALREADY_ASSIGNED` | Task assigned to another robot | Query task status |
| `NETWORK_TIMEOUT` | Request timed out | Retry with backoff |

#### 9.3 Retry Strategy

**Idempotent Operations (GET, PUT):**
- Immediate retry
- Exponential backoff: 1s → 2s → 4s → 8s → 16s
- Max retries: 5

**Non-Idempotent Operations (POST):**
- Include idempotency key: `X-Idempotency-Key: {uuid}`
- Server deduplicates within 24-hour window
- Retry with same idempotency key

---

### 10. Monitoring & Observability

#### 10.1 Metrics

**Orchestrator Metrics:**
- Connection uptime percentage
- Event processing latency (p50, p95, p99)
- Sync operation duration
- Firmware cache hit rate
- Task assignment success rate

**Central Server Metrics:**
- Active orchestrator connections
- Event fanout latency
- Data sync throughput (bytes/sec)
- Firmware download bandwidth
- Task completion rate

#### 10.2 Logging

**Log Format (JSON):**
```json
{
  "timestamp": "2025-01-08T11:00:00Z",
  "level": "INFO",
  "component": "orchestrator",
  "entity_id": "mall-a-uuid",
  "orchestrator_id": "orch-mall-a-001",
  "event": "data_sync_completed",
  "duration_ms": 1500,
  "records_synced": 15,
  "trace_id": "trace-abc123"
}
```

**Log Levels:**
- `DEBUG`: Detailed operation logs
- `INFO`: Normal operation events
- `WARN`: Recoverable errors, degraded performance
- `ERROR`: Operation failures, requires attention
- `FATAL`: Critical failures, orchestrator shutdown

#### 10.3 Health Checks

**Orchestrator Health Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "healthy",
  "checks": {
    "central_server_connection": "healthy",
    "local_cache": "healthy",
    "grpc_server": "healthy",
    "tower_connection": "healthy",
    "disk_space": "healthy"
  },
  "metrics": {
    "connected_robots": 12,
    "active_tasks": 3,
    "cache_size_mb": 100,
    "uptime_seconds": 86400
  },
  "timestamp": "2025-01-08T11:00:00Z"
}
```

**Status Values:**
- `healthy`: All systems operational
- `degraded`: Some non-critical systems down
- `unhealthy`: Critical systems down

---

### 11. Deployment Considerations

#### 11.1 Network Requirements

**Central Server:**
- Public IP address
- TLS certificate (Let's Encrypt or commercial CA)
- Ports: 443 (HTTPS)

**Orchestrator:**
- Private IP (mall local network)
- No inbound connections required (firewall-friendly)
- Ports: 50051 (gRPC), 8080 (Tower), 443 (HTTPS for beacons)

**Firewall Rules:**
- Orchestrator outbound HTTPS to central server (443)
- Orchestrator inbound gRPC from Tower (50051)
- Orchestrator inbound HTTPS from beacons (443)
- Tower inbound WebSocket from robots (8080)

#### 11.2 Scaling

**Horizontal Scaling (Central Server):**
- Load balancer for multiple server instances
- Redis for SSE connection state
- MongoDB replica set for data persistence
- CDN for firmware distribution

**Vertical Scaling (Orchestrator):**
- 2 CPU cores, 4 GB RAM recommended
- 20 GB disk for firmware cache
- SSD recommended for local database

**Capacity Planning:**
- 1 orchestrator per mall entity
- Up to 500 beacons per orchestrator
- Up to 50 robots per orchestrator
- Up to 1000 concurrent tasks per orchestrator

#### 11.3 Disaster Recovery

**Central Server Backup:**
- Daily MongoDB backups
- Firmware repository mirrored to S3/GCS
- Configuration stored in version control

**Orchestrator Backup:**
- Local cache backed up weekly
- Configuration stored in persistent volume
- Automatic failover to backup orchestrator (future)

**Recovery Time Objective (RTO):**
- Central server: < 1 hour
- Orchestrator: < 15 minutes

**Recovery Point Objective (RPO):**
- Central server: < 1 hour (database replication)
- Orchestrator: < 24 hours (daily backup)

---

## Implementation Roadmap

### Phase 1: Core Protocol (Week 1-2)
- [ ] Implement orchestrator registration
- [ ] Implement SSE event subscription
- [ ] Implement heartbeat mechanism
- [ ] Implement basic data sync (full + delta)

### Phase 2: Data Synchronization (Week 3-4)
- [ ] Implement checksum verification
- [ ] Implement conflict resolution
- [ ] Implement batch sync for large entities
- [ ] Add monitoring and metrics

### Phase 3: Firmware Distribution (Week 5-6)
- [ ] Implement firmware metadata API
- [ ] Implement firmware download caching
- [ ] Implement beacon firmware update flow
- [ ] Add rollback support

### Phase 4: Task Management (Week 7-8)
- [ ] Implement task assignment reporting
- [ ] Implement task status updates
- [ ] Implement task completion flow
- [ ] Add task analytics

### Phase 5: Beacon Integration (Week 9-10)
- [ ] Implement beacon registration via orchestrator
- [ ] Implement beacon heartbeat
- [ ] Implement access log upload
- [ ] Add beacon fleet management UI

### Phase 6: Production Hardening (Week 11-12)
- [ ] Add comprehensive error handling
- [ ] Implement rate limiting
- [ ] Add security audit logging
- [ ] Load testing and optimization
- [ ] Documentation and runbooks

---

## Appendix

### A. Protocol Buffers (Alternative to REST)

For high-throughput scenarios, consider using Protocol Buffers + gRPC instead of REST:

**Example: Data Sync Service**
```protobuf
service OrchestratorSync {
  rpc Register(RegisterRequest) returns (RegisterResponse);
  rpc StreamEvents(EventsRequest) returns (stream Event);
  rpc SyncData(SyncRequest) returns (SyncResponse);
  rpc ReportHeartbeat(HeartbeatRequest) returns (HeartbeatResponse);
}

message Event {
  string id = 1;
  string type = 2;
  google.protobuf.Timestamp timestamp = 3;
  bytes payload = 4;
}
```

### B. WebSocket Alternative

For malls with restrictive firewalls that block SSE:

**Endpoint:** `wss://server.navign.com/api/orchestrators/ws`

**Message Format:**
```json
{
  "type": "event",
  "event": "data_update",
  "id": "evt-123456",
  "data": {...}
}
```

### C. Offline Operation

When orchestrator cannot reach central server:

1. **Read Operations:** Serve from local cache
2. **Write Operations:** Queue in local database
3. **Task Assignment:** Continue with local algorithm
4. **Firmware Updates:** Serve from cache
5. **Access Control:** Beacons function independently

**Queue Flush on Reconnection:**
- Upload queued events in batches (100 per request)
- Resume at last successful batch on failure

### D. Migration Guide

**Migrating from GitHub-based firmware to central server:**

1. Upload all firmware binaries to central server
2. Configure orchestrators with new firmware endpoints
3. Update beacon OTA code to use orchestrator endpoints
4. Deprecate direct GitHub access after 30 days

---

**Document Version:** 1.0.0
**Last Updated:** 2025-01-08
**Status:** Draft for Review
