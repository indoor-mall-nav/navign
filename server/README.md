# Navign

A high-performance Rust-based backend server for intelligent indoor navigation and wayfinding systems. Navign provides advanced pathfinding algorithms, beacon-based access control, and comprehensive management APIs for complex indoor spaces such as shopping malls, transportation hubs, schools, and hospitals.

## 🌟 Features

### 🗺️ Advanced Indoor Navigation
- **Intelligent Pathfinding**: Dijkstra-based algorithm with optimized routing between areas using bump allocation for ultra-fast memory management
- **Multi-floor Navigation**: Support for elevators, escalators, and stairs with customizable restrictions
- **Area Connectivity Graph**: Dynamic graph generation for complex indoor layouts with support for multiple connection types
- **Agent Instance Pattern**: Smart handling of areas with single-entry access points
- **Real-time Route Instructions**: Step-by-step navigation with coordinate-based guidance
- **Point-to-Point Navigation**: Support for both merchant-to-merchant and coordinate-to-coordinate routing
- **Connectivity Limits**: Configurable routing constraints (elevator, escalator, stairs availability)

### 🔐 Beacon-Based Authentication & Access Control
- **TOTP (Time-based One-Time Password)**: Secure time-synchronized authentication with HMAC-SHA1
- **BLE Integration**: Bluetooth Low Energy beacon support for proximity-based access
- **Challenge-Response Protocol**: Secure handshake mechanism for door unlocking
- **Multi-method Auth**: Support for RFID, NFC, Biometric, TOTP, and Password authentication
- **Unlock Instance Management**: Track and manage unlock attempts with user and device tracking
- **ESP32 Support**: Compatible with ESP32, ESP32-C3, ESP32-C5, ESP32-C6, and ESP32-S3 devices

### 🏢 Entity Management
- **Multiple Entity Types**: Support for Malls, Transportation hubs, Schools, and Hospitals
- **Hierarchical Structure**: Entities → Areas → Merchants/Services with full relationship tracking
- **Geospatial Support**: Longitude, latitude, and altitude range management
- **Flexible Tagging**: Categorization and search via tags
- **Floor Management**: Support for both European (Level) and US (Floor) naming conventions, plus basements
- **Polygon-based Areas**: Define complex area shapes with coordinate polygons

### 🏪 Merchant & Service Management
- **Rich Merchant Types**: Food (with cuisine types), Electronics, Clothing, Supermarket, Health, Entertainment, Facilities, Rooms
- **Chain Store Support**: Track merchants that are part of chain store series
- **Branding**: Color codes for UI representation (e.g., Starbucks green, McDonald's yellow)
- **Operating Hours**: Configurable available periods with time-based access control
- **Contact Information**: Email, phone, website, and social media integration
- **Merchant Styles**: Different visual representations (Counter, Booth, Room, etc.)

### 🔌 RESTful API
- **Full CRUD Operations**: Complete management for entities, areas, connections, merchants, and beacons
- **OAuth2 Integration**: GitHub, Google, and WeChat authentication
- **JWT Token-based Auth**: Secure session management with custom claims
- **CORS Enabled**: Cross-origin resource sharing for web clients
- **Health Check Endpoint**: Database connectivity monitoring
- **Public Key Certification**: P-256 ECDSA public key endpoint for verification

### ⚡ High Performance
- **Bump Allocation**: Ultra-fast memory allocation using `bumpalo` for pathfinding operations
- **Async/Await**: Built on Tokio runtime for concurrent request handling
- **MongoDB Integration**: Efficient document storage and querying with connection pooling
  (*We plan to migrate to PostgreSQL in future releases.*)
- **Optimized Routing**: Efficient graph algorithms with minimal memory overhead

## 🏗️ Architecture

### Core Components

```
server/
├── src/
│   ├── main.rs                    # Application entry point, route definitions
│   ├── database.rs                # MongoDB connection management
│   ├── certification.rs           # Public key certification
│   ├── shared.rs                  # Shared utilities and types
│   │
│   ├── kernel/                    # Core business logic
│   │   ├── mod.rs
│   │   ├── cryptography.rs        # Cryptographic utilities
│   │   ├── totp.rs                # TOTP generation and validation
│   │   ├── user.rs                # User management
│   │   │
│   │   ├── auth/                  # Authentication modules
│   │   │   ├── mod.rs
│   │   │   ├── token.rs           # JWT token handling
│   │   │   ├── github.rs          # GitHub OAuth
│   │   │   ├── google.rs          # Google OAuth
│   │   │   ├── wechat.rs          # WeChat OAuth
│   │   │   └── password.rs        # Password authentication
│   │   │
│   │   ├── route/                 # Pathfinding and navigation
│   │   │   ├── mod.rs             # Route finding entry point
│   │   │   ├── instructions.rs    # Navigation instruction types
│   │   │   │
│   │   │   ├── types/             # Data structures for routing
│   │   │   │   ├── mod.rs
│   │   │   │   ├── area.rs
│   │   │   │   ├── connection.rs
│   │   │   │   ├── entity.rs
│   │   │   │   └── merchant.rs
│   │   │   │
│   │   │   └── implementations/   # Routing algorithms
│   │   │       ├── mod.rs
│   │   │       ├── connectivity_graph.rs  # Graph construction
│   │   │       ├── navigate.rs            # Navigation logic
│   │   │       ├── displacement_route.rs  # Path displacement
│   │   │       ├── agent_instance.rs      # Agent pattern handling
│   │   │       ├── connect_with_instance.rs
│   │   │       ├── contiguous.rs          # Contiguous area handling
│   │   │       └── convert_entity_in.rs
│   │   │
│   │   └── unlocker/              # Beacon unlock management
│   │       ├── mod.rs             # Unlock API handlers
│   │       └── instance.rs        # Unlock instance tracking
│   │
│   └── schema/                    # Database models
│       ├── mod.rs
│       ├── entity.rs              # Entity schema
│       ├── area.rs                # Area schema with polygon support
│       ├── connection.rs          # Connection schema
│       ├── merchant.rs            # Merchant schema
│       ├── beacon.rs              # Beacon schema
│       ├── beacon_secrets.rs      # Beacon authentication secrets
│       ├── user.rs                # User schema
│       ├── user_public.rs         # Public user data
│       ├── authentication.rs      # Authentication records
│       ├── service.rs             # Service trait for CRUD
│       ├── metadata.rs            # Pagination and metadata
│       └── polygon/               # Polygon geometry utilities
│           ├── mod.rs
│           ├── node.rs
│           └── line.rs
│
├── data/                          # Sample/test data
│   ├── areas.json
│   ├── connections.json
│   └── merchants.json
│
├── ts-schema/                     # TypeScript type definitions
│   ├── index.d.ts
│   ├── entity.d.ts
│   ├── area.d.ts
│   ├── connection.d.ts
│   ├── merchant.d.ts
│   └── beacon.d.ts
│
├── Cargo.toml                     # Rust dependencies
└── README.md
```

## 🚀 Getting Started

### Prerequisites

- **Rust**: 1.70+ (2024 edition)
- **MongoDB**: 4.4+
- **Node.js**: 16+ (for TypeScript schema generation, optional)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/navign.git
   cd navign/server
   ```

2. **Set up environment variables**
   
   Create a `.env` file in the server directory:
   ```env
   MONGODB_HOST=localhost:27017
   MONGODB_DB_NAME=navign
   MONGODB_URI=mongodb://localhost:27017
   JWT_SIGN_KEY=your-secret-key-here
   
   # Optional OAuth credentials
   GITHUB_CLIENT_ID=your-github-client-id
   GITHUB_CLIENT_SECRET=your-github-client-secret
   GOOGLE_CLIENT_ID=your-google-client-id
   GOOGLE_CLIENT_SECRET=your-google-client-secret
   WECHAT_APP_ID=your-wechat-app-id
   WECHAT_APP_SECRET=your-wechat-app-secret
   ```

3. **Install dependencies and build**
   ```bash
   cargo build --release
   ```

4. **Run the server**
   ```bash
   cargo run --release
   ```

The server will start on `http://0.0.0.0:3000`

### Quick Test

```bash
# Health check
curl http://localhost:3000/health

# Get public key certificate
curl http://localhost:3000/cert
```

## 📡 API Reference

### Base URL
```
http://localhost:3000
```

### Core Endpoints

#### Health & Status
- `GET /` - Root endpoint
- `GET /health` - Health check (checks MongoDB connection)
- `GET /cert` - Get public key certificate (PEM format)

#### Entities
- `GET /api/entities` - Search entities (query params: nation, region, city, name, longitude, latitude)
- `GET /api/entities/{id}` - Get entity by ID
- `POST /api/entities` - Create new entity
- `PUT /api/entities` - Update entity
- `DELETE /api/entities/{id}` - Delete entity
- `GET /api/entities/{id}/route` - Find route within entity
- `GET /api/entities/{id}/route/point` - Find route by coordinates

#### Areas
- `GET /api/entities/{eid}/areas` - Get all areas in entity
- `GET /api/entities/{eid}/areas/{id}` - Get specific area
- `POST /api/entities/{eid}/areas` - Create area
- `PUT /api/entities/{eid}/areas` - Update area
- `DELETE /api/entities/{eid}/areas/{id}` - Delete area
- `GET /api/entities/{eid}/areas/{aid}/beacons` - Get beacons in area
- `GET /api/entities/{eid}/areas/{aid}/merchants` - Get merchants in area

#### Merchants
- `GET /api/entities/{eid}/merchants` - Get all merchants
- `GET /api/entities/{eid}/merchants/{id}` - Get specific merchant
- `POST /api/entities/{eid}/merchants` - Create merchant
- `PUT /api/entities/{eid}/merchants` - Update merchant
- `DELETE /api/entities/{eid}/merchants/{id}` - Delete merchant

#### Connections
- `GET /api/entities/{eid}/connections` - Get all connections
- `GET /api/entities/{eid}/connections/{id}` - Get specific connection
- `POST /api/entities/{eid}/connections` - Create connection
- `PUT /api/entities/{eid}/connections` - Update connection
- `DELETE /api/entities/{eid}/connections/{id}` - Delete connection

#### Beacons
- `GET /api/entities/{eid}/beacons` - Get all beacons
- `GET /api/entities/{eid}/beacons/{id}` - Get specific beacon
- `POST /api/entities/{eid}/beacons` - Create beacon
- `PUT /api/entities/{eid}/beacons` - Update beacon
- `DELETE /api/entities/{eid}/beacons/{id}` - Delete beacon

#### Beacon Unlock (Requires Authentication)
- `POST /api/entities/{eid}/beacons/{id}/unlocker` - Create unlock instance
- `PUT /api/entities/{eid}/beacons/{id}/unlocker/{instance}/status` - Update unlock status
- `PUT /api/entities/{eid}/beacons/{id}/unlocker/{instance}/outcome` - Record unlock result

### Navigation Request Example

```bash
# Route from merchant to merchant
curl "http://localhost:3000/api/entities/{entity_id}/route?departure={merchant_id}&arrival={merchant_id}&elevator=true&escalator=true&stairs=false"

# Route by coordinates (lon,lat,area_id)
curl "http://localhost:3000/api/entities/{entity_id}/route/point?departure=114.123,22.456,{area_id}&arrival=114.789,22.012,{area_id}"
```

**Response:**
```json
[
  {
    "move": [114.123, 22.456]
  },
  {
    "move": [114.125, 22.458]
  },
  {
    "transport": ["conn_id", "target_area_id", "elevator"]
  },
  {
    "move": [114.789, 22.012]
  }
]
```

## 🔒 Authentication

Navign supports multiple authentication methods:

### JWT Token Authentication
Include the JWT token in the Authorization header:
```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  http://localhost:3000/api/entities/{eid}/beacons/{id}/unlocker
```

### OAuth2 Providers (Not implemented yet)
- **GitHub**: OAuth2 integration for GitHub authentication
- **Google**: OAuth2 integration for Google authentication
- **WeChat**: OAuth2 integration for WeChat authentication

### Beacon Authentication Flow

1. **Beacon Registration**: Beacon sends device info to server
2. **Time Synchronization**: Server returns timestamp for clock adjustment
3. **TOTP Generation**: Beacon generates TOTP using shared secret
4. **Unlock Request**:
   - User connects to beacon via BLE
   - Beacon generates challenge (beacon ID + timestamp + nonce)
   - User forwards challenge to server via internet
   - Server validates permissions and generates TOTP
   - User forwards TOTP to beacon via BLE
   - Beacon verifies TOTP and unlocks

## 🧮 Data Models

### Entity Types
- **Mall**: Shopping centers
- **Transportation**: Airports, train stations, bus terminals
- **School**: Educational institutions
- **Hospital**: Medical facilities

### Connection Types
- **Gate**: Access-controlled passages
- **Escalator**: Moving staircases
- **Elevator**: Vertical transportation
- **Stairs**: Static staircases
- **Rail**: Dedicated rail systems (e.g., airport shuttles)
- **Shuttle**: Shuttle buses

### Merchant Types
- **Food**: Restaurants, cafes (with cuisine types)
- **Electronics**: Mobile, computer, accessories
- **Clothing**: Menswear, womenswear, childrenswear
- **Supermarket**: Grocery stores
- **Health**: Pharmacies, clinics
- **Entertainment**: Cinemas, arcades
- **Facility**: Restrooms, ATMs, information desks
- **Room**: Hotel rooms, offices, meeting rooms

### Beacon Types
- **Navigation**: Location-based services
- **Marketing**: Proximity marketing
- **Tracking**: Asset tracking
- **Environmental**: Environmental monitoring
- **Security**: Access control
- **Other**: Custom purposes

## 🛠️ Technology Stack

### Backend
- **Rust** - Systems programming language for safety and performance
- **Axum** - Modern web framework built on Tokio
- **Tokio** - Async runtime for concurrent operations
- **MongoDB** - NoSQL database for flexible document storage
- **BSON** - Binary JSON for MongoDB operations

### Cryptography & Security
- **jsonwebtoken** - JWT token handling
- **bcrypt** - Password hashing
- **p256** - ECDSA signature generation
- **rsa** - RSA cryptography
- **hmac** - HMAC message authentication
- **sha1**, **sha2** - Cryptographic hash functions
- **oauth2** - OAuth2 client implementation

### Performance Optimization
- **bumpalo** - Fast bump allocation for pathfinding
- **bumpalo-herd** - Thread-safe bump allocation

### Utilities
- **serde** - Serialization/deserialization
- **anyhow** - Error handling
- **chrono** - Date and time handling
- **uuid** - UUID generation
- **base64**, **hex** - Encoding utilities
- **wkt** - Well-Known Text geometry format
- **reqwest** - HTTP client for OAuth
- **tower-http** - HTTP middleware (CORS)

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with logging
cargo test -- --nocapture

# Run specific test
cargo test test_instruction_display
```

## 📊 Performance Considerations

- **Bump Allocation**: Pathfinding operations use bump allocation to reduce memory allocation overhead
- **Connection Pooling**: MongoDB connection pool (2-8 connections) for optimal resource usage
- **Async Operations**: All I/O operations are asynchronous for maximum throughput
- **Efficient Graph Algorithms**: Dijkstra's algorithm with optimized data structures

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright (c) 2025 Ethan Wu

## 🙏 Acknowledgments

- Built with Rust's amazing ecosystem
- Inspired by real-world indoor navigation challenges
- Designed for complex multi-floor indoor environments

## 📮 Contact

For questions and support, please open an issue on GitHub.

---

**Navign** - Navigate with confidence in complex indoor spaces.
