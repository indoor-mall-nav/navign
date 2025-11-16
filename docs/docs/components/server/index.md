# Server Component

The Navign server is a centralized backend built with Rust and Axum, providing REST APIs for navigation, access control, and entity management.

## Overview

**Location:** `server/`

**Technologies:**
- **Framework:** Axum 0.8.6 (async web framework)
- **Runtime:** Tokio 1.47.1 (async runtime)
- **Database:** MongoDB 3.3.0 (primary), PostgreSQL via SQLx 0.8.6 (optional)
- **Cryptography:** p256 0.13.2 (ECDSA), bcrypt 0.17.1
- **Authentication:** jsonwebtoken 10.0.0, oauth2 5.0.0

**Port:** 3000 (default)

## Key Features

### 1. RESTful API
- Complete CRUD for entities, areas, beacons, merchants, connections
- User authentication and authorization
- Pathfinding and navigation instructions
- Access control instance management

### 2. Multi-Database Support
- MongoDB (primary) - Flexible schema, fast prototyping
- PostgreSQL (optional) - ACID compliance, relational queries
- Dual-database mode for gradual migration

### 3. Authentication
- OAuth2 (GitHub, Google, WeChat)
- Password-based authentication with bcrypt
- JWT token generation (24h expiration)

### 4. Pathfinding
- Dijkstra's algorithm with bump allocation
- Multi-floor routing via connections
- Support for elevators, stairs, escalators
- Turn-by-turn navigation instructions

### 5. Security
- P-256 ECDSA cryptography
- TOTP generation for access control
- Rate limiting (planned)
- CORS configuration

## Architecture

```
┌─────────────────────────────────────┐
│         REST API (Axum)            │
│     Port 3000 (HTTP/HTTPS)         │
└────────────┬────────────────────────┘
             │
       ┌─────┴─────────┐
       ▼               ▼
  ┌──────────┐   ┌──────────┐
  │ MongoDB  │   │PostgreSQL│
  │(Primary) │   │(Optional)│
  └──────────┘   └──────────┘
```

## API Endpoints

### Health & Info
```
GET  /                    # Health check
GET  /health              # Database ping
GET  /cert                # Server public key (PEM)
```

### Authentication
```
POST /api/auth/register   # User registration
POST /api/auth/login      # User login
GET  /api/auth/{provider}/authorize  # OAuth2 flow
```

### Entities
```
GET    /api/entities             # Search entities
POST   /api/entities             # Create entity
GET    /api/entities/{id}        # Get entity
PUT    /api/entities             # Update entity
DELETE /api/entities/{id}        # Delete entity
GET    /api/entities/{id}/route  # Pathfinding
```

### Areas
```
GET    /api/entities/{eid}/areas       # List areas
POST   /api/entities/{eid}/areas       # Create area
GET    /api/entities/{eid}/areas/{id}  # Get area
PUT    /api/entities/{eid}/areas       # Update area
DELETE /api/entities/{eid}/areas/{id}  # Delete area
```

### Beacons
```
GET    /api/entities/{eid}/beacons       # List beacons
POST   /api/entities/{eid}/beacons       # Create beacon
GET    /api/entities/{eid}/beacons/{id}  # Get beacon
PUT    /api/entities/{eid}/beacons       # Update beacon
DELETE /api/entities/{eid}/beacons/{id}  # Delete beacon
```

### Merchants
```
GET    /api/entities/{eid}/merchants       # List merchants
POST   /api/entities/{eid}/merchants       # Create merchant
GET    /api/entities/{eid}/merchants/{id}  # Get merchant
PUT    /api/entities/{eid}/merchants       # Update merchant
DELETE /api/entities/{eid}/merchants/{id}  # Delete merchant
```

### Connections
```
GET    /api/entities/{eid}/connections       # List connections
POST   /api/entities/{eid}/connections       # Create connection
GET    /api/entities/{eid}/connections/{id}  # Get connection
PUT    /api/entities/{eid}/connections       # Update connection
DELETE /api/entities/{eid}/connections/{id}  # Delete connection
```

### Access Control
```
POST /api/entities/{eid}/beacons/{id}/unlocker                  # Create unlock instance
PUT  /api/entities/{eid}/beacons/{id}/unlocker/{instance}/status   # Update status
PUT  /api/entities/{eid}/beacons/{id}/unlocker/{instance}/outcome  # Record result
```

## Database Schema

### Collections (MongoDB)
- `entities` - Buildings (malls, hospitals, airports)
- `areas` - Polygonal zones within entities
- `beacons` - BLE devices for positioning/access
- `merchants` - Stores, restaurants, facilities
- `connections` - Inter-area links (elevators, stairs)
- `users` - User accounts and authentication
- `beacon_secrets` - Private keys for beacons

### Tables (PostgreSQL)
Same schema, with additional relational integrity constraints and foreign keys.

## Documentation

### PostgreSQL Migration
**[PostgreSQL Migration Guide](./postgres-migration.md)** - Migrating from MongoDB to PostgreSQL

Complete guide for the dual-database architecture and gradual migration strategy.

**Topics:**
- Repository pattern implementation
- Type-safe ID handling (UUID vs Integer)
- Schema migrations
- Dual-database API handlers

---

### PostgreSQL Migration Summary
**[Migration Summary](./postgres-migration-summary.md)** - Quick overview of migration status

Summary of the PostgreSQL migration layer implementation, including:
- Current status
- Implemented repositories
- 4-phase migration strategy
- ID type conventions

---

## Pathfinding Algorithm

**Location:** `server/src/kernel/route/implementations/`

The server uses Dijkstra's algorithm with bump allocation for efficient pathfinding:

```rust
use bumpalo::Bump;

let arena = Bump::new();
let graph = build_graph(&arena, entity);
let path = dijkstra(&arena, graph, start, end);
let instructions = generate_instructions(path);
```

**Features:**
- Multi-floor routing
- Connection type handling (elevator, stairs, escalator)
- Turn-by-turn instructions
- Obstacle avoidance
- Optimal path selection

**Instruction Types:**
- `ENTER_AREA` - Enter a new area/zone
- `USE_CONNECTION` - Use elevator/stairs/escalator
- `TURN_LEFT` / `TURN_RIGHT` - Navigation turns
- `DESTINATION_REACHED` - End of route

## Environment Variables

```bash
# MongoDB (required)
MONGODB_HOST=localhost:27017
MONGODB_DB_NAME=navign

# PostgreSQL (optional)
POSTGRES_URL=postgresql://user:password@localhost:5432/navign
POSTGRES_RUN_MIGRATIONS=true

# Logging
RUST_LOG=info

# OAuth2 (optional)
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...
GOOGLE_CLIENT_ID=...
GOOGLE_CLIENT_SECRET=...
```

## Development

### Setup

```bash
# Start MongoDB
docker run -d -p 27017:27017 mongo:8.0

# Optional: Start PostgreSQL
docker run -d -p 5432:5432 \
  -e POSTGRES_PASSWORD=password \
  postgres:16

cd server
cargo build
```

### Running

```bash
# Development mode
cargo run

# Production build
cargo build --release
cargo run --release

# With logging
RUST_LOG=debug cargo run
```

### Testing

```bash
# Run tests (requires MongoDB)
cargo test

# Run with specific test
cargo test test_pathfinding

# CI checks
just ci-server
```

## PostgreSQL Migration

### Running Migrations

```bash
# Run migrations manually
cd server
cargo run --bin migrate

# Auto-run on server start
POSTGRES_RUN_MIGRATIONS=true cargo run
```

### Migration Scripts

```bash
# Migrate specific collection
./server/scripts/migrate_entities.sh

# Migrate all data
./server/scripts/migrate_all.sh
```

See [PostgreSQL Migration Guide](./postgres-migration.md) for complete details.

## Security Considerations

1. **Cryptography:**
   - P-256 ECDSA for beacon signatures
   - bcrypt for password hashing (cost factor 12)
   - JWT tokens with 24h expiration

2. **Rate Limiting:**
   - API rate limiting (planned)
   - Unlock attempt rate limiting (beacon-side)

3. **CORS:**
   - Currently permissive (development)
   - Should restrict origins in production

4. **Input Validation:**
   - All user inputs validated
   - Database queries sanitized
   - Bounds checking for coordinates

## Performance

### Pathfinding Optimization
- Bump allocation for zero-cost routing
- Graph caching (planned)
- Pre-computed shortest paths (planned)

### Database Optimization
- Index all frequently queried fields
- Connection pooling
- Query optimization

### Async Runtime
- Tokio for async I/O
- Multi-threaded executor
- Non-blocking database queries

## See Also

- [Main Server Documentation](../server.md) - Complete server component guide
- [Mobile API Integration](../mobile.md) - Client-side API usage
- [Beacon Protocol](../beacon.md) - Access control implementation
- [Admin Orchestrator](../admin/) - Robot fleet management
