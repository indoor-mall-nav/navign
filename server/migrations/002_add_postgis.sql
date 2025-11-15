-- Add PostGIS extension and convert x/y coordinates to GEOMETRY(POINT)

-- Enable PostGIS extension
CREATE EXTENSION IF NOT EXISTS postgis;

-- Update areas table to use PostGIS POINT for centroid
ALTER TABLE areas DROP COLUMN IF EXISTS centroid_x;
ALTER TABLE areas DROP COLUMN IF EXISTS centroid_y;
ALTER TABLE areas ADD COLUMN centroid GEOMETRY(POINT, 4326);

CREATE INDEX idx_areas_centroid ON areas USING GIST(centroid);

-- Update merchants table to use PostGIS POINT for location
ALTER TABLE merchants DROP COLUMN IF EXISTS x;
ALTER TABLE merchants DROP COLUMN IF EXISTS y;
ALTER TABLE merchants ADD COLUMN location GEOMETRY(POINT, 4326) NOT NULL;

CREATE INDEX idx_merchants_location ON merchants USING GIST(location);

-- Update beacons table to use PostGIS POINT for location
ALTER TABLE beacons DROP COLUMN IF EXISTS x;
ALTER TABLE beacons DROP COLUMN IF EXISTS y;
ALTER TABLE beacons ADD COLUMN location GEOMETRY(POINT, 4326) NOT NULL;

CREATE INDEX idx_beacons_location ON beacons USING GIST(location);

-- Helper function to create PostGIS POINTs from x, y coordinates
-- Usage: ST_SetSRID(ST_MakePoint(x, y), 4326)
COMMENT ON EXTENSION postgis IS 'PostGIS geometry and geography spatial types and functions';
