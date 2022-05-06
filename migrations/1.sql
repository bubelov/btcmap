CREATE TABLE places (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  lat REAL NOT NULL,
  lon REAL NOT NULL,
  address TEXT NOT NULL,
  amenity TEXT NOT NULL,
  phone TEXT NOT NULL,
  website TEXT NOT NULL,
  opening_hours TEXT NOT NULL,
  source TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT ( strftime('%Y-%m-%dT%H:%M:%SZ') ),
  updated_at TEXT NOT NULL DEFAULT ( strftime('%Y-%m-%dT%H:%M:%SZ') )
);