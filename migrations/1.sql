CREATE TABLE places (
  id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  lat REAL NOT NULL,
  lon REAL NOT NULL,
  created_at TEXT NOT NULL DEFAULT ( strftime('%Y-%m-%dT%H:%M:%SZ') ),
  updated_at TEXT NOT NULL DEFAULT ( strftime('%Y-%m-%dT%H:%M:%SZ') )
);