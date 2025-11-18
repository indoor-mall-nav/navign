-- Migration v3: Align with PostgreSQL schema using INTEGER IDs and WKB spatial data
-- SQLite with SpatiaLite extension for spatial functions

-- Enable SpatiaLite extension (must be loaded at runtime)
-- PRAGMA case_sensitive_like = ON;

-- Entities table (UUID stored as TEXT)
CREATE TABLE IF NOT EXISTS entities (
    id TEXT PRIMARY KEY,  -- UUID as text
    type TEXT NOT NULL,  -- EntityType as string
    name TEXT NOT NULL,
    description TEXT,
    point_min_wkb BLOB NOT NULL,  -- WKB POINT (longitude_min, latitude_min)
    point_max_wkb BLOB NOT NULL,  -- WKB POINT (longitude_max, latitude_max)
    altitude_min REAL,
    altitude_max REAL,
    nation TEXT,
    region TEXT,
    city TEXT,
    tags TEXT NOT NULL,  -- JSON array
    created_at INTEGER NOT NULL,  -- Unix timestamp in milliseconds
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name);
CREATE INDEX IF NOT EXISTS idx_entities_location ON entities(nation, region, city);

-- Areas table (INTEGER id, aligned with PostgreSQL SERIAL)
CREATE TABLE IF NOT EXISTS areas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    floor_type TEXT,  -- FloorType: "Level", "Floor", or "Basement"
    floor_name INTEGER,
    beacon_code TEXT NOT NULL,
    polygon_wkb BLOB NOT NULL,  -- WKB POLYGON
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_areas_entity ON areas(entity_id);
CREATE INDEX IF NOT EXISTS idx_areas_floor ON areas(floor_type, floor_name);
CREATE INDEX IF NOT EXISTS idx_areas_beacon_code ON areas(beacon_code);
CREATE INDEX IF NOT EXISTS idx_areas_name ON areas(name);

-- Beacons table (INTEGER id)
CREATE TABLE IF NOT EXISTS beacons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    area_id INTEGER NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    merchant_id INTEGER REFERENCES merchants(id) ON DELETE SET NULL,
    connection_id INTEGER REFERENCES connections(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    description TEXT,
    type TEXT NOT NULL,  -- BeaconType as string
    location_wkb BLOB NOT NULL,  -- WKB POINT
    device TEXT NOT NULL,  -- BeaconDevice as string
    mac TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_beacons_entity ON beacons(entity_id);
CREATE INDEX IF NOT EXISTS idx_beacons_area ON beacons(area_id);
CREATE INDEX IF NOT EXISTS idx_beacons_merchant ON beacons(merchant_id);
CREATE INDEX IF NOT EXISTS idx_beacons_connection ON beacons(connection_id);
CREATE INDEX IF NOT EXISTS idx_beacons_mac ON beacons(mac);

-- Merchants table (INTEGER id)
CREATE TABLE IF NOT EXISTS merchants (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    area_id INTEGER NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    chain TEXT,
    beacon_code TEXT NOT NULL,
    type TEXT NOT NULL,  -- JSON serialized MerchantType
    color TEXT,
    tags TEXT NOT NULL,  -- JSON array
    location_wkb BLOB NOT NULL,  -- WKB POINT (centroid/entrance)
    style TEXT NOT NULL,  -- MerchantStyle as string
    polygon_wkb BLOB NOT NULL,  -- WKB POLYGON (merchant boundary)
    available_period TEXT,  -- JSON array [[start, end], ...]
    opening_hours TEXT,  -- JSON array of arrays (7 days)
    email TEXT,
    phone TEXT,
    website TEXT,
    social_media TEXT,  -- JSON array
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_merchants_entity ON merchants(entity_id);
CREATE INDEX IF NOT EXISTS idx_merchants_area ON merchants(area_id);
CREATE INDEX IF NOT EXISTS idx_merchants_name ON merchants(name);
CREATE INDEX IF NOT EXISTS idx_merchants_beacon_code ON merchants(beacon_code);

-- Connections table (INTEGER id)
CREATE TABLE IF NOT EXISTS connections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    type TEXT NOT NULL,  -- ConnectionType as string
    connected_areas TEXT NOT NULL,  -- JSON array: [[area_id, x, y, enabled], ...]
    available_period TEXT NOT NULL,  -- JSON array: [[start, end], ...]
    tags TEXT NOT NULL,  -- JSON array
    gnd_wkb BLOB,  -- WKB POINT (ground coordinates, optional)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(entity_id, name)
);

CREATE INDEX IF NOT EXISTS idx_connections_entity ON connections(entity_id);
CREATE INDEX IF NOT EXISTS idx_connections_name ON connections(name);
CREATE INDEX IF NOT EXISTS idx_connections_type ON connections(type);

-- Route cache (for offline pathfinding)
CREATE TABLE IF NOT EXISTS route_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id TEXT NOT NULL,
    from_area INTEGER NOT NULL,
    from_x REAL NOT NULL,
    from_y REAL NOT NULL,
    to_area INTEGER NOT NULL,
    to_x REAL NOT NULL,
    to_y REAL NOT NULL,
    limits TEXT NOT NULL,  -- JSON serialized ConnectivityLimits
    instructions TEXT NOT NULL,  -- JSON serialized route instructions
    computed_at INTEGER NOT NULL,
    UNIQUE(entity_id, from_area, from_x, from_y, to_area, to_x, to_y, limits)
);

CREATE INDEX IF NOT EXISTS idx_route_cache_entity ON route_cache(entity_id);
CREATE INDEX IF NOT EXISTS idx_route_cache_computed_at ON route_cache(computed_at);

-- User preferences
CREATE TABLE IF NOT EXISTS preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Sync metadata
CREATE TABLE IF NOT EXISTS sync_metadata (
    entity_id TEXT PRIMARY KEY,
    last_sync INTEGER NOT NULL,
    full_sync_required INTEGER NOT NULL DEFAULT 0  -- SQLite doesn't have BOOLEAN, use INTEGER
);
