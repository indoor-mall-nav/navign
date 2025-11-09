-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable PostGIS extension for geometry/geography types
CREATE EXTENSION IF NOT EXISTS postgis;

-- Entities table
CREATE TABLE entities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    type VARCHAR(50) NOT NULL CHECK (type IN ('mall', 'transportation', 'school', 'hospital')),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    longitude_min DOUBLE PRECISION NOT NULL,
    longitude_max DOUBLE PRECISION NOT NULL,
    latitude_min DOUBLE PRECISION NOT NULL,
    latitude_max DOUBLE PRECISION NOT NULL,
    altitude_min DOUBLE PRECISION,
    altitude_max DOUBLE PRECISION,
    nation VARCHAR(100),
    region VARCHAR(100),
    city VARCHAR(100),
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CONSTRAINT valid_longitude CHECK (longitude_min <= longitude_max),
    CONSTRAINT valid_latitude CHECK (latitude_min <= latitude_max),
    CONSTRAINT valid_altitude CHECK (altitude_min IS NULL OR altitude_max IS NULL OR altitude_min <= altitude_max)
);

CREATE INDEX idx_entities_location ON entities(longitude_min, longitude_max, latitude_min, latitude_max);
CREATE INDEX idx_entities_city ON entities(city) WHERE city IS NOT NULL;
CREATE INDEX idx_entities_type ON entities(type);

-- Areas table (incremental ID)
CREATE TABLE areas (
    id BIGSERIAL PRIMARY KEY,
    entity UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    beacon_code VARCHAR(50) NOT NULL,
    floor_type VARCHAR(20) CHECK (floor_type IN ('level', 'floor', 'basement')),
    floor_name INTEGER,
    polygon GEOMETRY(POLYGON, 4326) NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CONSTRAINT unique_area_name_per_entity UNIQUE(entity, name)
);

CREATE INDEX idx_areas_entity ON areas(entity);
CREATE INDEX idx_areas_beacon_code ON areas(beacon_code);
CREATE INDEX idx_areas_polygon ON areas USING GIST(polygon);

-- Beacons table (incremental ID)
CREATE TABLE beacons (
    id BIGSERIAL PRIMARY KEY,
    entity UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    area BIGINT NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    merchant UUID REFERENCES merchants(id) ON DELETE SET NULL,
    connection UUID REFERENCES connections(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    type VARCHAR(50) NOT NULL CHECK (type IN ('navigation', 'marketing', 'tracking', 'environmental', 'security', 'other')),
    location GEOMETRY(POINT, 4326) NOT NULL,
    device VARCHAR(20) NOT NULL CHECK (device IN ('esp32', 'esp32c3', 'esp32c5', 'esp32c6', 'esp32s3')),
    mac VARCHAR(17) NOT NULL UNIQUE,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX idx_beacons_entity ON beacons(entity);
CREATE INDEX idx_beacons_area ON beacons(area);
CREATE INDEX idx_beacons_mac ON beacons(mac);
CREATE INDEX idx_beacons_location ON beacons USING GIST(location);

-- Merchants table
CREATE TABLE merchants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    area BIGINT NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    chain VARCHAR(255),
    beacon_code VARCHAR(50) NOT NULL,
    type JSONB NOT NULL,
    color VARCHAR(7),
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    location GEOMETRY(POINT, 4326) NOT NULL,
    style VARCHAR(20) NOT NULL CHECK (style IN ('point', 'polygon')),
    polygon GEOMETRY(POLYGON, 4326),
    available_period JSONB,
    email VARCHAR(255),
    phone VARCHAR(50),
    website VARCHAR(500),
    social_media JSONB,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CONSTRAINT unique_merchant_name_per_entity UNIQUE(entity, name)
);

CREATE INDEX idx_merchants_entity ON merchants(entity);
CREATE INDEX idx_merchants_area ON merchants(area);
CREATE INDEX idx_merchants_location ON merchants USING GIST(location);
CREATE INDEX idx_merchants_beacon_code ON merchants(beacon_code);

-- Connections table
CREATE TABLE connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    type VARCHAR(20) NOT NULL CHECK (type IN ('gate', 'escalator', 'elevator', 'stairs', 'rail', 'shuttle')),
    connected_areas JSONB NOT NULL,
    available_period JSONB NOT NULL DEFAULT '[]'::jsonb,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    gnd GEOMETRY(POINT, 4326),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CONSTRAINT unique_connection_name_per_entity UNIQUE(entity, name)
);

CREATE INDEX idx_connections_entity ON connections(entity);
CREATE INDEX idx_connections_type ON connections(type);

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) UNIQUE,
    password_hash VARCHAR(255),
    github_id VARCHAR(100) UNIQUE,
    google_id VARCHAR(100) UNIQUE,
    wechat_id VARCHAR(100) UNIQUE,
    avatar_url TEXT,
    public_key BYTEA,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email) WHERE email IS NOT NULL;
CREATE INDEX idx_users_github_id ON users(github_id) WHERE github_id IS NOT NULL;
CREATE INDEX idx_users_google_id ON users(google_id) WHERE google_id IS NOT NULL;
CREATE INDEX idx_users_wechat_id ON users(wechat_id) WHERE wechat_id IS NOT NULL;

-- Beacon secrets table (for access control)
CREATE TABLE beacon_secrets (
    beacon_id BIGINT PRIMARY KEY REFERENCES beacons(id) ON DELETE CASCADE,
    private_key BYTEA NOT NULL,
    public_key BYTEA NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

-- Firmwares table
CREATE TABLE firmwares (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    version VARCHAR(50) NOT NULL,
    device VARCHAR(20) NOT NULL CHECK (device IN ('esp32', 'esp32c3', 'esp32c5', 'esp32c6', 'esp32s3')),
    description TEXT,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    is_stable BOOLEAN NOT NULL DEFAULT false,
    created_at BIGINT NOT NULL,
    CONSTRAINT unique_version_device UNIQUE(version, device)
);

CREATE INDEX idx_firmwares_device ON firmwares(device);
CREATE INDEX idx_firmwares_is_stable ON firmwares(is_stable);

-- Unlock instances table (for access control tracking)
CREATE TABLE unlock_instances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    beacon_id BIGINT NOT NULL REFERENCES beacons(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'success', 'failure', 'expired')),
    outcome VARCHAR(50),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX idx_unlock_instances_beacon ON unlock_instances(beacon_id);
CREATE INDEX idx_unlock_instances_user ON unlock_instances(user_id);
CREATE INDEX idx_unlock_instances_status ON unlock_instances(status);
