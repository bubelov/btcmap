use rusqlite::Connection;
use crate::model::Place;

pub struct PlaceRepository {
    conn: Connection,
}

impl PlaceRepository {
    pub fn new(conn: Connection) -> PlaceRepository {
        PlaceRepository { conn }
    }

    pub fn select_all(&self) -> anyhow::Result<Vec<Place>> {
        let mut stmt = self.conn.prepare("SELECT id, source, lat, lon FROM places")?;

        let rows = stmt.query_map(
            [],
            |row| Ok(Place { id: row.get(0)?, source: row.get(1)?, lat: row.get(2)?, lon: row.get(3)? }),
        )?;

        let mut places: Vec<Place> = Vec::new();

        for place in rows {
            places.push(place?);
        }

        Ok(places)
    }
}