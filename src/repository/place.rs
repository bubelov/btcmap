use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;
use anyhow::{Error, Result};
use crate::model::Place;

pub struct PlaceRepository {
    conn: Connection,
}

impl PlaceRepository {
    pub fn new(conn: Connection) -> PlaceRepository {
        PlaceRepository { conn }
    }

    pub fn select_all(&self) -> Result<Vec<Place>> {
        let mut stmt = self.conn.prepare("SELECT id, lat, lon, tags, created_at, updated_at, deleted_at FROM places ORDER BY id DESC")?;

        let rows = stmt.query_map(
            [],
            |row| {
                let tags: String = row.get(3)?;
                let tags: Value = serde_json::from_str(&tags).unwrap_or_default();

                Ok(Place {
                    id: row.get(0)?,
                    lat: row.get(1)?,
                    lon: row.get(2)?,
                    tags,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    deleted_at: row.get(6)?,
                })
            },
        )?;

        let mut places: Vec<Place> = Vec::new();

        for place in rows {
            places.push(place?);
        }

        Ok(places)
    }

    pub fn select_by_id(&self, id: i64) -> Result<Option<Place>> {
        self.conn.query_row(
            "SELECT id, lat, lon, tags, created_at, updated_at FROM places WHERE id = ?",
            params![id],
            |row| {
                let tags: String = row.get(3)?;
                let tags: Value = serde_json::from_str(&tags).unwrap_or_default();

                Ok(Place {
                    id: row.get(0)?,
                    lat: row.get(1)?,
                    lon: row.get(2)?,
                    tags,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    deleted_at: row.get(6)?,
                })
            },
        )
            .optional()
            .map_err(|e| Error::new(e))
    }
}