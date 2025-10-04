# Indoor Mall Navigation Server

A high-performance Rust-based backend server for indoor navigation and wayfinding systems. This server provides intelligent pathfinding, beacon-based authentication, and comprehensive management APIs for indoor spaces such as shopping malls, transportation hubs, schools, and hospitals.

## 🌟 Features

### 🗺️ Advanced Indoor Navigation
- **Intelligent Pathfinding**: Dijkstra-based algorithm with optimized routing between areas
- **Multi-floor Navigation**: Support for elevators, escalators, and stairs with customizable restrictions
- **Area Connectivity Graph**: Dynamic graph generation for complex indoor layouts
- **Agent Instance Pattern**: Smart handling of areas with single-entry access points
- **Real-time Route Instructions**: Step-by-step navigation with coordinate-based guidance

### 🔐 Beacon-Based Authentication
- **TOTP (Time-based One-Time Password)**: Secure time-synchronized authentication
- **BLE Integration**: Bluetooth Low Energy beacon support for proximity-based access
- **Challenge-Response Protocol**: Secure handshake mechanism for door unlocking
- **Multi-method Auth**: Support for RFID, NFC, Biometric, TOTP, and Password authentication

### 🏢 Entity Management
- **Multiple Entity Types**: Support for Malls, Transportation hubs, Schools, and Hospitals
- **Hierarchical Structure**: Entities → Areas → Merchants/Services
- **Geospatial Support**: Longitude, latitude, and altitude range management
- **Flexible Tagging**: Categorization and search via tags

### 🔌 RESTful API
- **Full CRUD Operations**: Complete management for all resources
- **OAuth2 Integration**: GitHub, Google, and WeChat authentication
- **JWT Token-based Auth**: Secure session management
- **CORS Enabled**: Cross-origin resource sharing for web clients

### ⚡ High Performance
- **Bump Allocation**: Ultra-fast memory allocation using `bumpalo` for pathfinding operations
- **Async/Await**: Built on Tokio runtime for concurrent request handling
- **MongoDB Integration**: Efficient document storage and querying
- **TypeScript Schema Export**: Automatic type generation for frontend integration

## 🏗️ Architecture

### Core Components

```
server/
├── src/
│   ├── kernel/           # Core business logic
│   │   ├── route/        # Pathfinding and navigation
│   │   │   ├── types/    # Data structures (Area, Connection, Entity, Merchant)
│   │   │   ├── utils/    # Algorithms (connectivity, displacement, blocks)
│   │   │   └── instructions.rs
│   │   ├── auth/         # Authentication modules (GitHub, Google, WeChat, Password)
│   │   ├── beacon.rs     # Beacon management
│   │   ├── totp.rs       # TOTP implementation
│   │   └── unlocker.rs   # Door unlock logic
│   ├── schema/           # Database schemas
│   │   ├── entity.rs     # Entity definitions
│   │   ├── area.rs       # Area and Floor types
│   │   ├── connection.rs # Connection types
│   │   ├── merchant.rs   # Merchant/Service definitions
│   │   ├── beacon.rs     # Beacon schema
│   │   └── user.rs       # User management
│   ├── database.rs       # MongoDB connection
│   └── main.rs           # Server entry point
└── ts-schema/            # TypeScript type definitions
```

### Data Model

- **Entity**: Top-level container (e.g., a shopping mall)
- **Area**: Physical spaces within an entity (e.g., floors, zones)
- **Connection**: Links between areas (stairs, elevators, gates, etc.)
- **Merchant**: Stores or services within areas
- **Beacon**: Physical BLE devices for authentication and location

## 🚀 Getting Started

### Prerequisites

- Rust 2024 Edition (1.75+)
- MongoDB 3.2+
- Environment variables configuration

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd indoor-mall/server
```

2. Create a `.env` file:
```env
MONGODB_URI=mongodb://localhost:27017
MONGODB_HOST=localhost:27017
MONGODB_DB_NAME=indoor-mall-nav
```

3. Build and run:
```bash
cargo build --release
cargo run
```

The server will start on `http://0.0.0.0:3000`

## 📡 API Endpoints

### Health & Info
- `GET /` - Root endpoint
- `GET /health` - Health check with database ping
- `GET /cert` - Public key certificate

### Entities
- `GET /api/entities` - Search entities
- `GET /api/entities/{id}` - Get entity details
- `POST /api/entities` - Create entity
- `PUT /api/entities` - Update entity
- `DELETE /api/entities/{id}` - Delete entity

### Navigation
- `GET /api/entities/{id}/route?from={merchant_id}&to={merchant_id}&disallow={restrictions}` - Find route
  - Restrictions: `e` (elevator), `s` (stairs), `c` (escalator)

### Areas
- `GET /api/entities/{eid}/areas` - List areas
- `GET /api/entities/{eid}/areas/{id}` - Get area
- `POST /api/entities/{eid}/areas` - Create area
- `PUT /api/entities/{eid}/areas` - Update area
- `DELETE /api/entities/{eid}/areas/{id}` - Delete area

### Merchants
- `GET /api/entities/{eid}/merchants` - List merchants
- `GET /api/entities/{eid}/merchants/{id}` - Get merchant
- `POST /api/entities/{eid}/merchants` - Create merchant
- `PUT /api/entities/{eid}/merchants` - Update merchant
- `DELETE /api/entities/{eid}/merchants/{id}` - Delete merchant

### Connections
- `GET /api/entities/{eid}/connections` - List connections
- `GET /api/entities/{eid}/connections/{id}` - Get connection
- `POST /api/entities/{eid}/connections` - Create connection
- `PUT /api/entities/{eid}/connections` - Update connection
- `DELETE /api/entities/{eid}/connections/{id}` - Delete connection

### Beacons
- `GET /api/entities/{eid}/beacons` - List beacons
- `GET /api/entities/{eid}/beacons/{id}` - Get beacon
- `POST /api/entities/{eid}/beacons` - Create beacon
- `PUT /api/entities/{eid}/beacons` - Update beacon
- `DELETE /api/entities/{eid}/beacons/{id}` - Delete beacon
- `POST /api/entities/{eid}/beacons/unlocker` - Initiate unlock challenge

## 🧭 Navigation Algorithm

The server implements a sophisticated multi-stage pathfinding algorithm:

1. **Quick Path Detection**: Checks for direct connections or contiguous areas
2. **Agent Instance Resolution**: Handles special case areas with single-entry points
3. **Dijkstra's Algorithm**: Falls back to full graph search with Manhattan distance heuristic
4. **Instruction Generation**: Converts path into coordinate-based movement instructions

### Connectivity Limits

Control which connection types are allowed during pathfinding:

```rust
pub struct ConnectivityLimits {
    elevator: bool,   // Allow elevators
    stairs: bool,     // Allow stairs
    escalator: bool,  // Allow escalators
}
```

## 🔒 Security Features

- **ECDSA P-256**: Cryptographic signing for unlock challenges
- **TOTP with HMAC-SHA1**: Time-based authentication codes
- **BCrypt Password Hashing**: Secure user password storage
- **JWT Tokens**: Stateless authentication
- **Nonce-based Challenges**: Prevent replay attacks

## 🛠️ Technology Stack

- **Framework**: Axum 0.8 (high-performance async web framework)
- **Runtime**: Tokio (async runtime)
- **Database**: MongoDB 3.2
- **Cryptography**: P256 (ECDSA), RSA, HMAC-SHA1, SHA2
- **Serialization**: Serde, BSON
- **Memory Management**: Bumpalo (arena allocation)
- **Authentication**: OAuth2, JWT, BCrypt

## 📊 Performance Optimizations

- **Bump Allocation**: Zero-cost memory management for pathfinding operations
- **Graph Caching**: Efficient connectivity graph generation
- **Async I/O**: Non-blocking database operations
- **Connection Pooling**: MongoDB connection pool with configurable limits
- **Binary Heap**: Optimized priority queue for Dijkstra's algorithm

## 🧪 Testing

Run the comprehensive test suite:

```bash
cargo test
```

Tests include:
- Contiguous area detection
- Agent instance resolution
- Dijkstra pathfinding
- Multi-floor navigation scenarios

## 📝 License

MIT License - Copyright (c) 2025 Ethan Wu

See [LICENSE](LICENSE) for full details.

## 🤝 Contributing

This is a sophisticated indoor navigation system. Contributions are welcome! Please ensure all tests pass before submitting pull requests.

## 📧 Contact

For questions or issues, please open an issue on the repository.

---

**Note**: This server is designed for indoor navigation systems and requires careful configuration of entities, areas, and connections to function properly. Refer to the TypeScript schema files in `ts-schema/` for detailed type definitions.
