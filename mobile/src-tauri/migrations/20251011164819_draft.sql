-- Add migration script here
CREATE TABLE IF NOT EXISTS active_areas
(
    id
        TEXT
        PRIMARY
            KEY,
    name
        TEXT
        NOT
            NULL,
    polygon
        TEXT
        NOT
            NULL,
    entity
        TEXT
        NOT
            NULL,
    updated_at
        INTEGER
        NOT
            NULL,
    stored_at
        INTEGER
        NOT
            NULL
);

CREATE TABLE IF NOT EXISTS beacons
(
    id
           TEXT
        PRIMARY
            KEY,
    mac
           TEXT
                NOT
                    NULL,
    location
           TEXT
                NOT
                    NULL,
    merchant
           TEXT
                NOT
                    NULL
        REFERENCES
            merchants
                (
                 id
                    ) ON DELETE CASCADE,
    area   TEXT NOT NULL REFERENCES active_areas
        (
         id
            )
        ON DELETE CASCADE,
    entity TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS merchants
(
    id VARCHAR(24) PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    chain TEXT,
    entity VARCHAR(24) NOT NULL,
    beacon_code TEXT NOT NULL,
    area VARCHAR(24) NOT NULL,
    type TEXT NOT NULL,
    color TEXT,
    tags TEXT NOT NULL,
    location TEXT NOT NULL,
    style TEXT NOT NULL,
    polygon TEXT,
    available_period TEXT,
    email TEXT,
    phone TEXT,
    website TEXT,
    social_media TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

INSERT INTO merchants (id, name, entity, beacon_code, area, type, tags, location, style, created_at, updated_at)
VALUES ('unknown', 'Unknown', 'unknown', 'unknown', 'unknown', '"other"', '[]', 'POINT(0 0)', 'store', 0, 0);