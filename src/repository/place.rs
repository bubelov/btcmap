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
        let mut stmt = self.conn.prepare("SELECT id, name, lat, lon, address, amenity, phone, website, opening_hours, source, created_at, updated_at FROM places")?;

        let rows = stmt.query_map(
            [],
            |row| Ok(Place {
                id: row.get(0)?,
                name: row.get(1)?,
                lat: row.get(2)?,
                lon: row.get(3)?,
                address: row.get(4)?,
                amenity: row.get(5)?,
                phone: row.get(6)?,
                website: row.get(7)?,
                opening_hours: row.get(8)?,
                source: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            }),
        )?;

        let mut places: Vec<Place> = Vec::new();

        for place in rows {
            places.push(place?);
        }

        Ok(places)
    }
}