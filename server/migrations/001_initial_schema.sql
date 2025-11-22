-- Initial PostgreSQL schema migration with PostGIS
-- UUID for entities and users, INTEGER for other tables
-- PostGIS GEOMETRY(POINT) for all coordinates

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS postgis;

-- Entities table (UUID)
CREATE TABLE entities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    point_min GEOMETRY(POINT, 4326) NOT NULL,
    point_max GEOMETRY(POINT, 4326) NOT NULL,
    altitude_min DOUBLE PRECISION,
    altitude_max DOUBLE PRECISION,
    nation VARCHAR(100),
    region VARCHAR(100),
    city VARCHAR(100),
    tags TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_entities_name ON entities(name);
CREATE INDEX idx_entities_location ON entities(nation, region, city);
CREATE INDEX idx_entities_point_min ON entities USING GIST(point_min);
CREATE INDEX idx_entities_point_max ON entities USING GIST(point_max);
CREATE INDEX idx_entities_tags ON entities USING GIN(tags);

-- Users table (UUID)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    phone VARCHAR(50),
    google VARCHAR(255),
    wechat VARCHAR(255),
    hashed_password VARCHAR(255) NOT NULL,
    activated BOOLEAN NOT NULL DEFAULT FALSE,
    privileged BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);

-- Areas table (INTEGER) with PostGIS
CREATE TABLE areas (
    id SERIAL PRIMARY KEY,
    entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    floor_type VARCHAR(50),
    floor_name INTEGER,
    beacon_code VARCHAR(100) NOT NULL,
    polygon GEOMETRY(POLYGON, 4326) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_areas_entity ON areas(entity_id);
CREATE INDEX idx_areas_floor ON areas(floor_type, floor_name);
CREATE INDEX idx_areas_beacon_code ON areas(beacon_code);
CREATE INDEX idx_areas_name ON areas(name);
CREATE INDEX idx_areas_polygon ON areas USING GIST(polygon);

-- Beacons table (INTEGER) with PostGIS
CREATE TABLE beacons (
    id SERIAL PRIMARY KEY,
    entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    area_id INTEGER NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    merchant_id INTEGER,
    connection_id INTEGER,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    type VARCHAR(50) NOT NULL,
    location GEOMETRY(POINT, 4326) NOT NULL,
    device VARCHAR(50) NOT NULL,
    mac VARCHAR(17) NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_beacons_entity ON beacons(entity_id);
CREATE INDEX idx_beacons_area ON beacons(area_id);
CREATE INDEX idx_beacons_merchant ON beacons(merchant_id);
CREATE INDEX idx_beacons_connection ON beacons(connection_id);
CREATE INDEX idx_beacons_mac ON beacons(mac);
CREATE INDEX idx_beacons_location ON beacons USING GIST(location);

-- Merchants table (INTEGER) with PostGIS
CREATE TABLE merchants (
    id SERIAL PRIMARY KEY,
    entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    area_id INTEGER NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    chain VARCHAR(255),
    beacon_code VARCHAR(255) NOT NULL,
    type JSONB NOT NULL,
    color VARCHAR(50),
    tags JSONB DEFAULT '[]'::jsonb,
    location GEOMETRY(POINT, 4326) NOT NULL, -- PostGIS POINT for centroid/entrance
    style VARCHAR(50),
    polygon GEOMETRY(POLYGON, 4326) NOT NULL, -- PostGIS POLYGON for merchant boundary
    available_period JSONB DEFAULT '[]'::jsonb,
    opening_hours JSONB,
    email VARCHAR(255),
    phone VARCHAR(50),
    website TEXT,
    social_media JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_merchants_entity ON merchants(entity_id);
CREATE INDEX idx_merchants_area ON merchants(area_id);
CREATE INDEX idx_merchants_name ON merchants(name);
CREATE INDEX idx_merchants_beacon_code ON merchants(beacon_code);
CREATE INDEX idx_merchants_location ON merchants USING GIST(location); -- Spatial index for centroid
CREATE INDEX idx_merchants_polygon ON merchants USING GIST(polygon); -- Spatial index for boundary
CREATE INDEX idx_merchants_tags ON merchants USING GIN(tags);

-- Connections table (INTEGER)
CREATE TABLE connections (
    id SERIAL PRIMARY KEY,
    entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    type VARCHAR(50) NOT NULL,
    connected_areas JSONB NOT NULL DEFAULT '[]'::jsonb,
    available_period JSONB DEFAULT '[]'::jsonb,
    tags JSONB DEFAULT '[]'::jsonb,
    gnd GEOMETRY(POINT, 4326),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT unique_connection_name_per_entity UNIQUE(entity_id, name)
);

CREATE INDEX idx_connections_entity ON connections(entity_id);
CREATE INDEX idx_connections_name ON connections(name);
CREATE INDEX idx_connections_type ON connections(type);
CREATE INDEX idx_connections_tags ON connections USING GIN(tags);
CREATE INDEX idx_connections_gnd ON connections USING GIST(gnd);

-- Add foreign key for beacons.connection_id (deferred to avoid circular dependency)
ALTER TABLE beacons ADD CONSTRAINT fk_beacons_connection
    FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE SET NULL;

CREATE INDEX idx_beacons_connection ON beacons(connection_id);

-- Beacon secrets table (INTEGER, linked to beacons)
CREATE TABLE beacon_secrets (
    id SERIAL PRIMARY KEY,
    beacon_id INTEGER NOT NULL REFERENCES beacons(id) ON DELETE CASCADE,
    counter BIGINT NOT NULL DEFAULT 0,
    private_key BYTEA NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_epoch TIMESTAMP WITH TIME ZONE DEFAULT '1970-01-01 00:00:00+00'
);

CREATE UNIQUE INDEX idx_beacon_secrets_beacon ON beacon_secrets(beacon_id);

-- User public keys table (INTEGER, linked to users)
CREATE TABLE user_public_keys (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    public_key TEXT NOT NULL,
    device_id VARCHAR(255) NOT NULL,
    device_name VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT unique_user_device UNIQUE(user_id, device_id)
);

CREATE INDEX idx_user_public_keys_user ON user_public_keys(user_id);
CREATE INDEX idx_user_public_keys_device ON user_public_keys(device_id);

-- Unlock instances table (INTEGER)
CREATE TABLE unlock_instances (
    id SERIAL PRIMARY KEY,
    beacon_id INTEGER NOT NULL REFERENCES beacons(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id VARCHAR(255) NOT NULL,
    timestamp BIGINT NOT NULL,
    beacon_nonce TEXT NOT NULL,
    challenge_nonce TEXT NOT NULL,
    stage VARCHAR(50) NOT NULL,
    outcome TEXT NOT NULL DEFAULT '',
    type VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_unlock_instances_beacon ON unlock_instances(beacon_id);
CREATE INDEX idx_unlock_instances_user ON unlock_instances(user_id);
CREATE INDEX idx_unlock_instances_device ON unlock_instances(device_id);
CREATE INDEX idx_unlock_instances_stage ON unlock_instances(stage);
CREATE INDEX idx_unlock_instances_created_at ON unlock_instances(created_at);

-- Firmware table (INTEGER)
CREATE TABLE firmwares (
    id SERIAL PRIMARY KEY,
    version VARCHAR(50) NOT NULL UNIQUE,
    chip VARCHAR(50) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    release_notes TEXT,
    is_stable BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_firmwares_version ON firmwares(version);
CREATE INDEX idx_firmwares_chip ON firmwares(chip);

-- Updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply updated_at trigger to all tables
CREATE TRIGGER update_entities_updated_at BEFORE UPDATE ON entities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_areas_updated_at BEFORE UPDATE ON areas
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_merchants_updated_at BEFORE UPDATE ON merchants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_beacons_updated_at BEFORE UPDATE ON beacons
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_connections_updated_at BEFORE UPDATE ON connections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_beacon_secrets_updated_at BEFORE UPDATE ON beacon_secrets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_public_keys_updated_at BEFORE UPDATE ON user_public_keys
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_unlock_instances_updated_at BEFORE UPDATE ON unlock_instances
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_firmwares_updated_at BEFORE UPDATE ON firmwares
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Helper functions for creating PostGIS POINTs from x, y coordinates
-- Usage: ST_SetSRID(ST_MakePoint(x, y), 4326)
COMMENT ON EXTENSION postgis IS 'PostGIS geometry and geography spatial types and functions';


