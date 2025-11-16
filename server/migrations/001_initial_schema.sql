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
    nation VARCHAR(100),
    region VARCHAR(100),
    city VARCHAR(100),
    address TEXT,
    longitude_min DOUBLE PRECISION NOT NULL,
    longitude_max DOUBLE PRECISION NOT NULL,
    latitude_min DOUBLE PRECISION NOT NULL,
    latitude_max DOUBLE PRECISION NOT NULL,
    floors JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_entities_name ON entities(name);
CREATE INDEX idx_entities_location ON entities(nation, region, city);
CREATE INDEX idx_entities_longitude ON entities(longitude_min, longitude_max);
CREATE INDEX idx_entities_latitude ON entities(latitude_min, latitude_max);

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
    floor VARCHAR(50) NOT NULL,
    beacon_code VARCHAR(100) NOT NULL,
    polygon JSONB NOT NULL,
    centroid GEOMETRY(POINT, 4326), -- PostGIS POINT with WGS84 SRID
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_areas_entity ON areas(entity_id);
CREATE INDEX idx_areas_floor ON areas(floor);
CREATE INDEX idx_areas_beacon_code ON areas(beacon_code);
CREATE INDEX idx_areas_name ON areas(name);
CREATE INDEX idx_areas_centroid ON areas USING GIST(centroid); -- Spatial index

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
    device_id VARCHAR(48) NOT NULL UNIQUE,
    floor VARCHAR(50) NOT NULL,
    location GEOMETRY(POINT, 4326) NOT NULL, -- PostGIS POINT with WGS84 SRID
    public_key TEXT,
    capabilities JSONB DEFAULT '[]'::jsonb,
    unlock_method VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_beacons_entity ON beacons(entity_id);
CREATE INDEX idx_beacons_area ON beacons(area_id);
CREATE INDEX idx_beacons_merchant ON beacons(merchant_id);
CREATE INDEX idx_beacons_device_id ON beacons(device_id);
CREATE INDEX idx_beacons_floor ON beacons(floor);
CREATE INDEX idx_beacons_location ON beacons USING GIST(location); -- Spatial index

-- Merchants table (INTEGER) with PostGIS
CREATE TABLE merchants (
    id SERIAL PRIMARY KEY,
    entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    area_id INTEGER NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    chain VARCHAR(255),
    type VARCHAR(50) NOT NULL,
    logo TEXT,
    images JSONB DEFAULT '[]'::jsonb,
    social_media JSONB DEFAULT '[]'::jsonb,
    floor VARCHAR(50) NOT NULL,
    location GEOMETRY(POINT, 4326) NOT NULL, -- PostGIS POINT for centroid/entrance
    polygon GEOMETRY(POLYGON, 4326) NOT NULL, -- PostGIS POLYGON for merchant boundary
    merchant_style VARCHAR(50),
    food_type VARCHAR(50),
    food_cuisine VARCHAR(50),
    chinese_food_cuisine VARCHAR(50),
    facility_type VARCHAR(50),
    rating DOUBLE PRECISION,
    reviews INTEGER DEFAULT 0,
    opening_hours JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_merchants_entity ON merchants(entity_id);
CREATE INDEX idx_merchants_area ON merchants(area_id);
CREATE INDEX idx_merchants_name ON merchants(name);
CREATE INDEX idx_merchants_type ON merchants(type);
CREATE INDEX idx_merchants_floor ON merchants(floor);
CREATE INDEX idx_merchants_location ON merchants USING GIST(location); -- Spatial index for centroid
CREATE INDEX idx_merchants_polygon ON merchants USING GIST(polygon); -- Spatial index for boundary

-- Connections table (INTEGER)
CREATE TABLE connections (
    id SERIAL PRIMARY KEY,
    entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    type VARCHAR(50) NOT NULL,
    connected_areas JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT unique_connection_name_per_entity UNIQUE(entity_id, name)
);

CREATE INDEX idx_connections_entity ON connections(entity_id);
CREATE INDEX idx_connections_name ON connections(name);
CREATE INDEX idx_connections_type ON connections(type);

-- Add foreign key for beacons.connection_id (deferred to avoid circular dependency)
ALTER TABLE beacons ADD CONSTRAINT fk_beacons_connection
    FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE SET NULL;

CREATE INDEX idx_beacons_connection ON beacons(connection_id);

-- Beacon secrets table (INTEGER, linked to beacons)
CREATE TABLE beacon_secrets (
    id SERIAL PRIMARY KEY,
    beacon_id INTEGER NOT NULL REFERENCES beacons(id) ON DELETE CASCADE,
    private_key BYTEA NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
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

CREATE TRIGGER update_firmwares_updated_at BEFORE UPDATE ON firmwares
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Helper functions for creating PostGIS POINTs from x, y coordinates
-- Usage: ST_SetSRID(ST_MakePoint(x, y), 4326)
COMMENT ON EXTENSION postgis IS 'PostGIS geometry and geography spatial types and functions';
