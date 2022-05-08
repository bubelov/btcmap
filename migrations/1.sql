CREATE TABLE places (
    id TEXT PRIMARY KEY,
    lat REAL NOT NULL,
    lon REAL NOT NULL,
    tags TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT ( strftime('%Y-%m-%dT%H:%M:%SZ') ),
    updated_at TEXT NOT NULL DEFAULT ( strftime('%Y-%m-%dT%H:%M:%SZ') )
);

CREATE TRIGGER places_updated_at UPDATE OF tags ON places
BEGIN
    UPDATE places SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ') WHERE id = old.id;
END;