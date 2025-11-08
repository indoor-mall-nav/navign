-- Comprehensive SQLite schema for offline navigation
-- Version 2: Aligned with latest shared schemas using WKT for coordinates

-- Entities (buildings, malls, etc.)
CREATE TABLE IF NOT EXISTS entities (
    id VARCHAR(24) PRIMARY KEY,
    type TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    longitude_min REAL NOT NULL,
    longitude_max REAL NOT NULL,
    latitude_min REAL NOT NULL,
    latitude_max REAL NOT NULL,
    altitude_min REAL,
    altitude_max REAL,
    nation TEXT,
    region TEXT,
    city TEXT,
    tags TEXT NOT NULL,  -- JSON array
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Areas (polygonal zones within entities)
CREATE TABLE IF NOT EXISTS areas (
    id VARCHAR(24) PRIMARY KEY,
    entity VARCHAR(24) NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    beacon_code TEXT NOT NULL,
    floor_type TEXT,  -- "level", "floor", or "basement"
    floor_name INTEGER,
    polygon TEXT NOT NULL,  -- WKT POLYGON string
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_areas_entity ON areas(entity);
CREATE INDEX IF NOT EXISTS idx_areas_floor ON areas(floor_type, floor_name);

-- Connections (gates, elevators, escalators, stairs between areas)
CREATE TABLE IF NOT EXISTS connections (
    id VARCHAR(24) PRIMARY KEY,
    entity VARCHAR(24) NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    type TEXT NOT NULL,  -- "gate", "elevator", "escalator", "stairs", "rail", "shuttle"
    connected_areas TEXT NOT NULL,  -- JSON array: [{"area": "id", "x": 0.0, "y": 0.0, "enabled": true}, ...]
    available_period TEXT NOT NULL,  -- JSON array: [[start, end], ...]
    tags TEXT NOT NULL,  -- JSON array
    gnd TEXT,  -- WKT POINT string for ground coordinates (if connects to outside)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_connections_entity ON connections(entity);
CREATE INDEX IF NOT EXISTS idx_connections_type ON connections(type);

-- Beacons (BLE devices for positioning and access control)
CREATE TABLE IF NOT EXISTS beacons (
    id VARCHAR(24) PRIMARY KEY,
    entity VARCHAR(24) NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    area VARCHAR(24) NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    merchant VARCHAR(24),  -- Can be NULL
    connection VARCHAR(24),  -- Can be NULL
    name TEXT NOT NULL,
    description TEXT,
    type TEXT NOT NULL,  -- "navigation", "marketing", "tracking", "environmental", "security", "other"
    location TEXT NOT NULL,  -- WKT POINT string
    device TEXT NOT NULL,  -- "esp32", "esp32c3", "esp32c5", "esp32c6", "esp32s3"
    mac TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_beacons_entity ON beacons(entity);
CREATE INDEX IF NOT EXISTS idx_beacons_area ON beacons(area);
CREATE INDEX IF NOT EXISTS idx_beacons_mac ON beacons(mac);
CREATE INDEX IF NOT EXISTS idx_beacons_merchant ON beacons(merchant);
CREATE INDEX IF NOT EXISTS idx_beacons_connection ON beacons(connection);

-- Merchants (stores, restaurants, facilities)
CREATE TABLE IF NOT EXISTS merchants (
    id VARCHAR(24) PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    chain TEXT,
    entity VARCHAR(24) NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    beacon_code TEXT NOT NULL,
    area VARCHAR(24) NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    type TEXT NOT NULL,  -- JSON serialized MerchantType
    color TEXT,
    tags TEXT NOT NULL,  -- JSON array
    location TEXT NOT NULL,  -- WKT POINT string
    style TEXT NOT NULL,  -- "store", "kiosk", "pop-up", "food-truck", "room"
    polygon TEXT,  -- WKT POLYGON string (optional)
    available_period TEXT,  -- JSON array (optional)
    email TEXT,
    phone TEXT,
    website TEXT,
    social_media TEXT,  -- JSON array (optional)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_merchants_entity ON merchants(entity);
CREATE INDEX IF NOT EXISTS idx_merchants_area ON merchants(area);
CREATE INDEX IF NOT EXISTS idx_merchants_name ON merchants(name);

-- Offline route cache (optional - for caching computed routes)
CREATE TABLE IF NOT EXISTS route_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity VARCHAR(24) NOT NULL,
    from_area VARCHAR(24) NOT NULL,
    from_x REAL NOT NULL,
    from_y REAL NOT NULL,
    to_area VARCHAR(24) NOT NULL,
    to_x REAL NOT NULL,
    to_y REAL NOT NULL,
    limits TEXT NOT NULL,  -- JSON serialized ConnectivityLimits
    instructions TEXT NOT NULL,  -- JSON serialized route instructions
    computed_at INTEGER NOT NULL,
    UNIQUE(entity, from_area, from_x, from_y, to_area, to_x, to_y, limits)
);

CREATE INDEX IF NOT EXISTS idx_route_cache_entity ON route_cache(entity);
CREATE INDEX IF NOT EXISTS idx_route_cache_computed_at ON route_cache(computed_at);

-- User preferences
CREATE TABLE IF NOT EXISTS preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Sync metadata
CREATE TABLE IF NOT EXISTS sync_metadata (
    entity VARCHAR(24) PRIMARY KEY,
    last_sync INTEGER NOT NULL,
    full_sync_required BOOLEAN NOT NULL DEFAULT 0
);
