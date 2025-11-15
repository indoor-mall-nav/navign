# Admin Panel - CRUD for Indoor Navigation System

This document describes the admin panel functionality for managing beacons, areas, merchants, and connections in the Navign indoor navigation system.

## Overview

The mobile frontend now includes a full-featured admin panel that supports:

- **Dual Mode Operation**: Works both as a Tauri app and as a standalone web application
- **CRUD Operations**: Create, Read, Update, Delete for all major entities
- **RESTful API Integration**: Direct HTTP communication when deployed as web app
- **Tauri Commands**: Uses native Tauri commands when running in desktop/mobile mode

## Features

### 1. Beacons Management
- Create/edit/delete BLE beacons
- Assign beacons to areas
- Configure beacon types (navigation, marketing)
- Set device types (ESP32, ESP32-C3, ESP32-S3, ESP32-C6)
- Define beacon locations

**Route**: `/admin/beacons`

### 2. Areas Management
- Create/edit/delete areas (zones)
- Define polygon boundaries for areas
- Set floor information (floor/level/basement)
- Assign beacon codes for identification

**Route**: `/admin/areas`

### 3. Merchants Management
- Create/edit/delete merchants (stores, restaurants, etc.)
- Set merchant types and categories
- Add tags for search and filtering
- Configure opening hours
- Add contact information (website, phone, email)
- Define merchant polygons and locations

**Route**: `/admin/merchants`

### 4. Connections Management
- Create/edit/delete connections between areas
- Support multiple connection types:
  - Gates (🚪)
  - Elevators (🛗)
  - Escalators (↗️)
  - Stairs (🪜)
  - Rails (🚇)
  - Shuttles (🚐)
- Define connected areas with coordinates
- Set availability periods

**Route**: `/admin/connections`

## Deployment Modes

### Mode 1: Tauri Desktop/Mobile App

When running as a Tauri application, the admin panel uses native Tauri commands to communicate with the backend.

**Setup:**
```bash
cd mobile
pnpm install
pnpm run tauri dev
```

**Features:**
- Native desktop/mobile experience
- Direct Rust backend communication
- No CORS issues
- Secure credential storage via Stronghold

### Mode 2: Standalone Web Application

The admin panel can be deployed as a standalone web application for browser-based administration.

**Setup:**

1. Create `.env` file:
```bash
cp .env.example .env
# Edit .env and set both VITE_API_BASE_URL and VITE_ORCHESTRATOR_URL
# CRUD operations go through the orchestrator, not the server
```

2. Build for production:
```bash
cd mobile
pnpm install
pnpm run build
```

3. Deploy the `dist/` directory to your web server.

**Features:**
- Browser-based access
- No installation required
- Cross-platform (any device with a browser)
- Direct HTTP API communication

### Environment Configuration

Create a `.env` file in the `mobile/` directory:

```env
# Server API (for navigation, etc.)
VITE_API_BASE_URL=http://localhost:3000

# Orchestrator API (for admin CRUD operations)
VITE_ORCHESTRATOR_URL=http://localhost:8081
```

For production:
```env
VITE_API_BASE_URL=https://api.yourserver.com
VITE_ORCHESTRATOR_URL=https://orchestrator.yourserver.com
```

**Note**: Environment variables are only used in web mode. Tauri mode ignores them and uses native commands.

### Architecture: Why Orchestrator for CRUD?

The admin panel routes all CRUD operations (Create, Read, Update, Delete) through the **orchestrator** rather than directly to the server. This is by design:

- **Server** (`localhost:3000`): Handles end-user operations like navigation, routing, and search
- **Orchestrator** (`localhost:8081`): The only service authorized to modify the central database for beacons, areas, merchants, and connections
- **Security**: This ensures that only authorized admin operations can modify core infrastructure data

When you use the admin panel in web mode, it makes HTTP requests to the orchestrator's REST API endpoints.

## API Abstraction Layer

The admin panel includes a smart API abstraction layer (`src/lib/api/client.ts`) that automatically detects the runtime environment:

```typescript
// Detects if running in Tauri or web mode
const isTauriMode = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}
```

### API Functions

All CRUD operations are available through the client API:

**Beacons:**
- `listBeacons(entityId, token)`
- `getBeacon(entityId, beaconId, token)`
- `createBeacon(entityId, beacon, token)`
- `updateBeacon(entityId, beacon, token)`
- `deleteBeacon(entityId, beaconId, token)`

**Areas:**
- `listAreas(entityId, token)`
- `getArea(entityId, areaId, token)`
- `createArea(entityId, area, token)`
- `updateArea(entityId, area, token)`
- `deleteArea(entityId, areaId, token)`

**Merchants:**
- `listMerchants(entityId, token)`
- `getMerchant(entityId, merchantId, token)`
- `createMerchant(entityId, merchant, token)`
- `updateMerchant(entityId, merchant, token)`
- `deleteMerchant(entityId, merchantId, token)`

**Connections:**
- `listConnections(entityId, token)`
- `getConnection(entityId, connectionId, token)`
- `createConnection(entityId, connection, token)`
- `updateConnection(entityId, connection, token)`
- `deleteConnection(entityId, connectionId, token)`

## API Endpoints (Web Mode - Orchestrator)

When running as a standalone web app, the admin panel makes HTTP requests to these **orchestrator** endpoints (default: `http://localhost:8081`):

**Important**: All CRUD operations are routed through the orchestrator, NOT the server. The orchestrator is the only service authorized to modify the central database.

### Beacons
- `GET /api/entities/{entityId}/beacons` - List all beacons
- `GET /api/entities/{entityId}/beacons/{beaconId}` - Get beacon details
- `POST /api/entities/{entityId}/beacons` - Create beacon
- `PUT /api/entities/{entityId}/beacons` - Update beacon
- `DELETE /api/entities/{entityId}/beacons/{beaconId}` - Delete beacon

### Areas
- `GET /api/entities/{entityId}/areas` - List all areas
- `GET /api/entities/{entityId}/areas/{areaId}` - Get area details
- `POST /api/entities/{entityId}/areas` - Create area
- `PUT /api/entities/{entityId}/areas` - Update area
- `DELETE /api/entities/{entityId}/areas/{areaId}` - Delete area

### Merchants
- `GET /api/entities/{entityId}/merchants` - List all merchants
- `GET /api/entities/{entityId}/merchants/{merchantId}` - Get merchant details
- `POST /api/entities/{entityId}/merchants` - Create merchant
- `PUT /api/entities/{entityId}/merchants` - Update merchant
- `DELETE /api/entities/{entityId}/merchants/{merchantId}` - Delete merchant

### Connections
- `GET /api/entities/{entityId}/connections` - List all connections
- `GET /api/entities/{entityId}/connections/{connectionId}` - Get connection details
- `POST /api/entities/{entityId}/connections` - Create connection
- `PUT /api/entities/{entityId}/connections` - Update connection
- `DELETE /api/entities/{entityId}/connections/{connectionId}` - Delete connection

## Authentication

All API requests (in web mode) require authentication via JWT token:

```
Authorization: Bearer <token>
```

The token is obtained through login and stored in the session state.

## Usage Examples

### Creating a Beacon

1. Navigate to `/admin/beacons`
2. Click "Create Beacon"
3. Fill in the form:
   - Name (e.g., "BEACON-0001-0001")
   - Type (navigation/marketing)
   - Device (ESP32-C3, etc.)
   - Select area
   - Set location coordinates
4. Click "Create"

### Creating an Area

1. Navigate to `/admin/areas`
2. Click "Create Area"
3. Fill in the form:
   - Name
   - Beacon code (unique identifier)
   - Floor information (optional)
   - Polygon coordinates as JSON: `[[x1, y1], [x2, y2], [x3, y3], ...]`
4. Click "Create"

### Creating a Connection

1. Navigate to `/admin/connections`
2. Click "Create Connection"
3. Fill in the form:
   - Name
   - Type (elevator, stairs, etc.)
   - Connected areas as JSON: `[["area_id_1", x1, y1], ["area_id_2", x2, y2]]`
   - Available period (optional): `[[36000000, 72000000]]` (milliseconds from midnight)
4. Click "Create"

## File Structure

```
mobile/src/
├── lib/api/
│   ├── client.ts              # API abstraction layer (Tauri/HTTP dual mode)
│   └── tauri.ts               # Original Tauri API functions
├── views/admin/
│   ├── AdminDashboard.vue     # Main admin dashboard
│   ├── BeaconsView.vue        # Beacons list view
│   ├── BeaconFormView.vue     # Beacon create/edit form
│   ├── AreasView.vue          # Areas list view
│   ├── AreaFormView.vue       # Area create/edit form
│   ├── MerchantsView.vue      # Merchants list view
│   ├── MerchantFormView.vue   # Merchant create/edit form
│   ├── ConnectionsView.vue    # Connections list view
│   └── ConnectionFormView.vue # Connection create/edit form
└── router/index.ts            # Router configuration with admin routes
```

## Security Considerations

### CORS Configuration

When running as a standalone web app, ensure your server has proper CORS configuration:

```rust
let cors = CorsLayer::new()
    .allow_origin("https://admin.yourdomain.com".parse::<HeaderValue>().unwrap())
    .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(vec![AUTHORIZATION, CONTENT_TYPE]);
```

### Authentication

- Always use HTTPS in production
- Store JWT tokens securely
- Implement token refresh mechanism
- Set appropriate token expiration times
- Use HttpOnly cookies when possible

### Input Validation

All forms include basic client-side validation, but **always validate on the server side** as well:
- Polygon coordinates must be valid arrays
- Required fields must be present
- ObjectIds must be valid MongoDB ObjectIds
- Numeric values must be within acceptable ranges

## Troubleshooting

### Issue: API calls return CORS errors

**Solution**: Check your server's CORS configuration and ensure the frontend origin is allowed.

### Issue: "Not implemented in Tauri mode" errors

**Solution**: Some CRUD operations may not have Tauri commands implemented yet. Either:
1. Add the missing Tauri commands in `src-tauri/src/lib.rs`
2. Deploy as a standalone web app instead

### Issue: Polygon parsing errors

**Solution**: Ensure polygon JSON is valid:
- Use double quotes for strings in JSON
- Include at least 3 coordinate pairs for areas
- Format: `[[x1, y1], [x2, y2], [x3, y3]]`

### Issue: Token expired errors

**Solution**: Implement token refresh or re-authenticate when tokens expire.

## Future Enhancements

- [ ] Add image upload for merchants
- [ ] Visual polygon editor (click to draw on map)
- [ ] Batch operations (import/export CSV)
- [ ] Real-time validation against server
- [ ] Undo/redo functionality
- [ ] Audit log for all CRUD operations
- [ ] Role-based access control
- [ ] Multi-language support for admin interface

## Contributing

When adding new CRUD functionality:

1. Add API functions to `src/lib/api/client.ts`
2. Create list view component in `src/views/admin/`
3. Create form view component in `src/views/admin/`
4. Add routes to `src/router/index.ts`
5. Update this documentation

## License

MIT License - See main project LICENSE file for details.
