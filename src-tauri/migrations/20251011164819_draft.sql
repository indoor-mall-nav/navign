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
    id
        TEXT
        PRIMARY
            KEY,
    name
        TEXT
        NOT
            NULL,
    entry
        TEXT
        NOT
            NULL
);