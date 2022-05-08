CREATE TABLE places (
    id INTEGER PRIMARY KEY,
    lat REAL NOT NULL,
    lon REAL NOT NULL,
    tags TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT ( strftime('%Y-%m-%dT%H:%M:%SZ') ),
    updated_at TEXT NOT NULL DEFAULT ( strftime('%Y-%m-%dT%H:%M:%SZ') ),
    deleted_at TEXT NOT NULL DEFAULT ''
);

CREATE TRIGGER places_updated_at UPDATE OF lat, lon, tags, deleted_at ON places
BEGIN
    UPDATE places SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ') WHERE id = old.id;
END;