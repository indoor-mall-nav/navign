# Server

The Navign server represents the central nervous system of the indoor navigation platform, orchestrating data flow between mobile clients, beacons, and administrative systems. Built on Rust's Axum framework with MongoDB for persistence, it handles the computational heavy lifting that mobile devices cannot efficiently perform: multi-floor pathfinding across complex building geometries, cryptographic operations for access control, and real-time coordination of autonomous robot fleets.

**Language:** Rust
**Framework:** Axum 0.8.6
**Runtime:** Tokio 1.47.1 (async)
**Database:** MongoDB 3.3.0
**Port:** 3000 (configurable via `SERVER_BIND_ADDR`)

## Architectural Philosophy

The server's design reflects three core principles that emerged from the practical constraints of indoor navigation systems:

**Computation Centralization:**
Mobile devices have limited battery and processing power. Complex pathfinding operations—particularly Dijkstra's algorithm across hundreds of interconnected areas—drain batteries rapidly. By centralizing these computations on the server, mobile clients can request routes via simple HTTP calls and receive pre-computed navigation instructions. This architectural decision trades network latency (typically 100-300ms) for battery preservation and consistent performance across diverse mobile hardware.

**Stateless Request Handling:**
The server maintains no session state for navigation requests. Each pathfinding query includes complete context: source location, destination, entity ID, and accessibility constraints. This statelessness enables horizontal scaling—multiple server instances behind a load balancer can handle requests without coordination. The trade-off is slightly larger request payloads, but the benefits of simplified deployment and elastic scaling outweigh this cost.

**Database as Source of Truth:**
All persistent state resides in MongoDB. The server's in-memory state is purely transient: request-scoped allocations for pathfinding computations, cached cryptographic keys loaded at startup, and rate limiting counters that periodically flush to prevent memory growth. This design simplifies disaster recovery—a server crash loses no data beyond in-flight requests, which clients automatically retry.

## Application State and Lifecycle

The server's runtime state is minimal, captured in a single structure:

```rust
pub struct AppState {
    db: Database,           // MongoDB connection pool
    private_key: SigningKey // P-256 ECDSA key for signatures
}
```

This state is shared across all request handlers via Axum's dependency injection. The database connection pool maintains 10-100 concurrent connections (configurable), with automatic connection recycling to handle MongoDB failover scenarios.

**Server Private Key:**

The P-256 ECDSA private key serves multiple purposes in the cryptographic protocol. At startup, the server either loads an existing key from disk or generates a new one. The corresponding public key is available via the `/cert` endpoint, allowing beacons and mobile clients to verify server signatures without pre-shared secrets.

This key enables future enhancements like encrypted API responses or signed firmware updates. Currently, it primarily signs TOTP tokens for access control, though the full potential remains untapped. The design anticipates a security model where the server acts as a certificate authority for the deployment, with beacons and clients trusting server-signed credentials.

## Middleware Architecture

Axum's middleware tower processes every request through several layers before reaching business logic handlers:

**CORS (Cross-Origin Resource Sharing):**

The current configuration permits requests from any origin—a deliberate choice for development flexibility. Production deployments should restrict this to known client origins (web admin dashboards, mobile app webviews if applicable). The permissive configuration exists because indoor navigation clients are typically mobile apps (not web browsers), where CORS doesn't apply. However, if a web-based admin panel exists, CORS becomes critical.

The middleware responds to preflight OPTIONS requests automatically, allowing browsers to verify permissions before sending actual requests. Without this, modern browsers block cross-origin API calls, breaking web-based clients.

**Rate Limiting:**

The server employs tower-governor for IP-based rate limiting, defaulting to 100 requests/second with burst allowance up to 200. This protects against accidental DOS from misconfigured clients and deliberate abuse. The rate limiter uses a smart IP extractor that handles proxy headers (X-Forwarded-For, X-Real-IP), correctly identifying clients behind load balancers or reverse proxies.

Rate limiting operates per-IP, not per-user. A multi-tenant building with shared WiFi might have hundreds of users behind a single public IP, all sharing the rate limit. This is acceptable for navigation queries (low frequency per user) but could become problematic for high-throughput use cases. Future enhancements might implement per-JWT-token limits for authenticated endpoints.

The background cleanup thread runs every 60 seconds, removing expired rate limit entries to prevent memory growth. Without this, the limiter's hash map would accumulate entries for every IP that ever made a request, eventually consuming gigabytes of RAM in high-traffic deployments.

## Authentication System

The server supports two authentication strategies: password-based and OAuth2, both issuing JWT tokens for subsequent requests.

**Password Authentication:**

User registration hashes passwords with bcrypt at cost factor 12—a deliberate balance between security and performance. Each hash takes approximately 200-300ms to compute, preventing rainbow table attacks while remaining responsive enough for login flows. Lower cost factors (8-10) are vulnerable to modern GPUs; higher factors (14+) introduce noticeable latency that degrades user experience.

The cost factor should increase over time as hardware improves. What's secure today (cost 12) may become vulnerable in five years as GPU performance advances. A planned enhancement tracks password hash creation timestamps, automatically re-hashing with updated cost factors on user login.

**OAuth2 Integration:**

The server acts as an OAuth2 client, supporting GitHub, Google, and WeChat as identity providers. The authentication flow follows standard OAuth2:

1. Client redirects user to provider's authorization page
2. User grants permission
3. Provider redirects back to server with authorization code
4. Server exchanges code for access token (server-to-server, never exposed to client)
5. Server fetches user profile using access token
6. Server creates or updates user record in MongoDB
7. Server issues JWT token to client

This flow keeps provider credentials secure—only the server knows the OAuth2 client secret, preventing exposure in decompiled mobile apps.

**JWT Token Structure:**

Tokens encode minimal claims to keep size small:

```rust
pub struct TokenClaims {
    pub sub: String,      // User ID (MongoDB ObjectId)
    pub username: String, // Display name
    pub exp: i64,         // Expiration timestamp (24 hours from issue)
    pub iat: i64,         // Issued at timestamp
}
```

The server signs tokens with HS256 (HMAC-SHA256) using a secret key stored in environment variables. Token verification occurs on every authenticated endpoint, with expired tokens returning 401 Unauthorized.

The 24-hour expiration balances security (compromised tokens have limited lifetime) with user experience (users aren't constantly re-authenticating). Mobile apps could implement refresh tokens to extend sessions without user interaction, though this isn't currently implemented.

## Data Models and Persistence

MongoDB serves as the primary data store, with collections representing core domain entities:

**Entities (Buildings):**

Entities represent physical structures—shopping malls, airports, hospitals, office complexes. Each entity has geographic bounds defined as a WKT (Well-Known Text) polygon, enabling spatial queries. The polygon typically represents the building footprint, allowing the system to determine if a GPS coordinate falls within the building's outdoor perimeter.

Entities contain floor lists, which are simply string identifiers ("1F", "2F", "B1" for basement 1, etc.). There's no enforced schema for floor naming—different regions use different conventions. Some buildings use British floor numbering (ground floor, first floor), others American (first floor, second floor). The system treats floor identifiers as opaque strings, relying on human operators to maintain consistency within a single entity.

**Areas (Rooms and Zones):**

Areas represent navigable spaces within entities. Each area has a polygon boundary in local coordinates (not geographic coordinates). The coordinate system's origin is arbitrary—typically the building's southwest corner at ground level. This local coordinate system simplifies indoor calculations, avoiding the complexity of map projections.

Areas are the fundamental unit for pathfinding. Connections link areas, creating a graph structure. The polygon geometry enables obstacle avoidance within areas—the pathfinding algorithm ensures routes stay within navigable space rather than passing through walls.

Each area associates with a single floor, establishing the vertical layering of the building model. Multi-floor areas (like atriums) are modeled as separate areas per floor with explicit connections.

**Beacons (Positioning Infrastructure):**

Beacons have both physical properties (MAC address, location coordinates) and logical properties (device type, capabilities). The system doesn't enforce a 1:1 mapping between physical beacons and database records—a single physical device could have multiple database entries if it serves multiple purposes (unlikely but permitted by the schema).

Device types classify beacons functionally:

- **Merchant**: Marks a store or service location
- **Pathway**: Navigation waypoint in hallways or open spaces
- **Connection**: Marks connection entry/exit points
- **Turnstile**: Access control checkpoint

These types influence how the mobile app presents beacons to users. Merchant beacons show business information; pathway beacons are invisible UI elements for positioning; turnstile beacons offer unlock actions.

**Connections (Vertical and Horizontal Transitions):**

Connections model how users move between areas. They're more complex than simple edges in a graph—each connection specifies entry/exit coordinates for every connected area. This enables precise route instructions: "Walk to elevator at (50, 30), take elevator to Floor 2, exit at (52, 31)."

Connection types (Elevator, Stairs, Escalator) enable accessibility routing. A wheelchair user excludes stairs and escalators from their connectivity graph, with the pathfinding algorithm finding alternative routes using only elevators.

Horizontal connections (between separated areas on the same floor, like distinct buildings connected by skybridges) use the same model. The connection type might be "Hallway" or "Bridge," though the current implementation doesn't formally distinguish horizontal from vertical.

## Pathfinding Engine

The pathfinding system represents the server's most computationally intensive operation, processing complex graph traversals across multi-floor building models.

**Two-Level Pathfinding:**

Navigation queries decompose into two levels:

1. **Graph-level**: Find sequence of areas from source to destination
2. **Geometric-level**: Find path within each area, avoiding obstacles

This hierarchical approach dramatically reduces computational complexity. Instead of searching a graph with thousands of nodes (if every polygon vertex were a node), the system searches a graph with dozens to hundreds of nodes (areas), then separately solves smaller geometric problems.

**Dijkstra's Algorithm:**

Area-to-area pathfinding uses Dijkstra's algorithm, chosen for its optimality guarantees and predictable performance. The algorithm finds the shortest path from source to destination, considering connection types based on user constraints.

The implementation uses a binary heap priority queue, yielding O((V + E) log V) time complexity where V is area count and E is connection count. For a typical mall with 100 areas and 150 connections, this completes in under 5 milliseconds on modest server hardware.

**Bump Allocation:**

Graph construction allocates numerous temporary data structures: node lists, edge lists, distance tables, predecessor maps. Traditional heap allocation would call malloc/free thousands of times per pathfinding request, with allocation overhead exceeding actual computation time.

Bump allocation solves this by allocating from a contiguous memory region (arena). Allocations are pointer bumps—incrementing an offset counter. Deallocation is free (no-op). When the request completes, the entire arena deallocates in one operation.

This reduces pathfinding latency by approximately 30% compared to standard allocation, a meaningful improvement for user-perceived responsiveness.

**Polygon-Based Obstacle Avoidance:**

Within-area pathfinding must respect polygon boundaries. The current implementation converts polygons to a grid-based representation, then applies A\* search. This approach is fast but produces suboptimal (longer) paths due to grid discretization.

The planned enhancement uses visibility graphs: nodes are polygon vertices, edges connect mutually visible vertices. Shortest path through this graph yields an optimal route that hugs polygon boundaries. The trade-off is higher preprocessing cost (O(V²) visibility checks for V vertices) versus faster queries.

**Instruction Generation:**

The pathfinding result is a sequence of waypoints, which the server converts into instructions:

- **Walk**: Movement within an area
- **Transport**: Use connection to change areas (elevator/stairs/escalator)
- **EnterArea**: Cross into a new area (informational)
- **ExitArea**: Leave current area (informational)

These abstract instructions enable the mobile client to provide contextual guidance. A "Transport" instruction with type "Elevator" shows elevator-specific UI: wait indicator, floor selection prompt, arrival notification.

## Access Control System

The server coordinates access control through a three-party protocol: mobile client, beacon, server. The server's role is authorization verification and audit logging.

**Unlock Instance Creation:**

When a mobile user taps "Unlock" on a door, the app requests an unlock instance from the server:

```
POST /api/entities/{eid}/beacons/{bid}/unlocker
Authorization: Bearer {jwt}
```

The server validates the JWT, verifies the user has permission for this beacon (checking user groups, roles, time-based restrictions), then generates a TOTP code bound to this specific unlock attempt.

**TOTP Generation:**

TOTP (Time-based One-Time Password) provides time-limited authorization. The server generates a 6-digit code using HMAC-SHA1 with a secret key shared with the beacon (during beacon provisioning). The TOTP changes every 30 seconds, so the mobile must complete the unlock protocol within this window.

This time-binding prevents indefinite reuse of unlock credentials. Even if an attacker captures an unlock proof, it expires quickly. Combined with the beacon's nonce-based replay prevention, this creates a robust defense against common attacks.

**Audit Logging:**

After the beacon grants or denies access, the mobile reports the outcome to the server. The server logs every unlock attempt—successful or failed—with timestamp, user ID, beacon ID, and outcome.

This audit trail serves multiple purposes:

- **Security monitoring**: Detect brute force attempts or unusual access patterns
- **Compliance**: Regulatory requirements often mandate access logging
- **Debugging**: Investigate user reports of unlock failures
- **Analytics**: Understand building traffic patterns

The logs are immutable append-only records. Deleting or modifying audit logs is prohibited, ensuring forensic integrity.

## Firmware Distribution

The server stores firmware binaries for OTA updates to ESP32-C3 beacons, though the distribution mechanism operates through the Orchestrator component.

**Upload and Validation:**

Administrators upload firmware binaries with metadata (device type, version, description). The server validates the binary format—checking for ESP32 magic bytes and valid segment headers—before accepting the upload. This prevents distribution of corrupted or incompatible firmware that could brick devices.

Each firmware is SHA-256 hashed, with the checksum stored alongside the binary. During OTA updates, beacons verify checksums before flashing, ensuring integrity despite potential network corruption.

**Version Management:**

The system maintains multiple concurrent firmware versions: stable releases for production deployment, beta versions for testing, and previous stable versions for rollback. Beacons query for the latest firmware matching their device type, with the server returning version numbers, download URLs, and checksums.

Staged rollouts are managed through deployment flags—marking specific firmware versions for gradual deployment (10% of devices, then 50%, then 100%). This limits blast radius if a firmware update has unforeseen issues.

## Performance Characteristics

**Pathfinding Latency:**

Typical pathfinding requests complete in 5-15ms:

- Graph construction: 2-5ms
- Dijkstra's algorithm: 2-5ms
- Instruction generation: 1-3ms
- Serialization: 1-2ms

These are CPU-bound operations that benefit from server-class processors. Mobile devices would take 50-100ms for equivalent computations, plus the battery drain from sustained CPU usage.

**Database Query Performance:**

MongoDB queries with proper indexing return in 1-5ms for typical datasets (thousands of entities, tens of thousands of areas). Without indexes, complex queries (like finding all beacons in an area) can take 100-500ms, degrading user experience.

Critical indexes:

- `entities.name` (text search)
- `areas.entity + areas.floor` (compound, for floor-specific queries)
- `beacons.entity + beacons.area` (compound, for area beacon listing)
- `beacons.device_id` (unique, for beacon identification)

**Rate Limiter Memory:**

The rate limiter consumes approximately 100 bytes per tracked IP address. At 100,000 unique IPs per day, this is 10MB—negligible on modern servers. The periodic cleanup prevents unbounded growth, maintaining steady-state memory usage even under sustained traffic.

## Deployment Considerations

**Environment Variables:**

The server requires several environment variables:

- `DATABASE_URL`: MongoDB connection string
- `DATABASE_NAME`: Database name
- `SERVER_BIND_ADDR`: Listen address (default: 0.0.0.0:3000)
- `JWT_SECRET`: Secret key for JWT signing
- `RUST_LOG`: Logging level (info, debug, trace)

**Database Connectivity:**

The MongoDB connection string supports replica sets for high availability:

```
mongodb://host1:27017,host2:27017,host3:27017/?replicaSet=rs0
```

The driver automatically handles primary node elections during failovers, with brief connection interruptions (typically <1 second).

**Horizontal Scaling:**

The stateless design enables horizontal scaling behind a load balancer. Multiple server instances share the same MongoDB database, with no inter-server coordination required. Request affinity (sticky sessions) is unnecessary—any server can handle any request.

The main scaling bottleneck is MongoDB, not the server application. Read-heavy workloads benefit from MongoDB read replicas, distributing query load across multiple database nodes.

## Security Best Practices

**Production Deployment Hardening:**

Development defaults prioritize convenience over security. Production deployments should:

1. **Restrict CORS origins** to known client domains
2. **Enable HTTPS** via reverse proxy (Nginx, Caddy)
3. **Use secret management** (HashiCorp Vault, AWS Secrets Manager) instead of environment variables for sensitive credentials
4. **Enable MongoDB authentication** and encryption at rest
5. **Implement request signing** for mobile API calls to prevent API abuse
6. **Add observability** (Prometheus metrics, OpenTelemetry tracing) for operational visibility

**Rate Limiting Tuning:**

The default 100 requests/second suits typical deployments with hundreds of users. Large deployments (thousands of concurrent users) should increase limits based on observed traffic patterns. Monitor rate limit rejection rates—if >1% of requests are rejected, limits are too aggressive.

**JWT Token Management:**

The current 24-hour token expiration is a compromise. Shorter expiration (1 hour) improves security but forces frequent re-authentication. Implementing refresh tokens allows long sessions while maintaining security—access tokens expire quickly (15 minutes), refresh tokens last longer (7 days) but only work for token renewal, not API access.

## Related Documentation

- [Pathfinding Algorithm Details](/pipelines/navigation)
- [Access Control Protocol](/pipelines/unlock)
- [Database Schema Reference](/components/server/schema)
- [API Authentication Guide](/components/server/authentication)
