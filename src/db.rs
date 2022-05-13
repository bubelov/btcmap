use std::fs::remove_file;
use rusqlite::{Connection, Row};
use serde_json::Value;
use crate::Place;

pub fn cli_main(
    args: &[String],
    db_conn: Connection,
) {
    match args.first() {
        Some(first_arg) => {
            match first_arg.as_str() {
                "create" => create(db_conn),
                "update" => update(db_conn),
                "delete" => delete(db_conn),
                _ => { panic!("Unknown action {first_arg}"); }
            }
        }
        None => { panic!("No osm actions passed"); }
    }
}

fn create(db_conn: Connection) {
    update(db_conn)
}

fn update(db_conn: Connection) {
    let schema_ver: i16 = db_conn.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
        row.get(0)
    }).unwrap();

    if schema_ver == 0 {
        println!("Creating database schema");
        db_conn.execute_batch(include_str!("../migrations/1.sql")).unwrap();
        db_conn.execute_batch(&format!("PRAGMA user_version={}", 1)).unwrap();
    } else {
        println!("Found database schema version {}", schema_ver);
    }

    println!("Database schema is up to date");
}

fn delete(db_conn: Connection) {
    if !db_conn.path().unwrap().exists() {
        panic!("Database does not exist");
    } else {
        println!("Found database at {}", db_conn.path().unwrap().to_str().unwrap());
        remove_file(db_conn.path().unwrap()).unwrap();
        println!("Database was deleted");
    }
}

pub fn place_mapper() -> fn(&Row) -> rusqlite::Result<Place> {
    |row: &Row| -> rusqlite::Result<Place> {
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
    }
}