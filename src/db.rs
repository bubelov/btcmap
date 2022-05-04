use std::fs::remove_file;
use std::path::Path;
use anyhow::{Result, Context};
use rusqlite::Connection;

pub async fn cli_main(args: &[String]) -> Result<()> {
    match args.first() {
        None => println!("No options passed"),
        Some(first_arg) => {
            match first_arg.as_str() {
                "migrate" => migrate().context("Unable to migrate database")?,
                "drop" => drop().context("Unable to drop database")?,
                _ => println!("Unknown command {first_arg}"),
            }
        }
    }

    Ok(())
}

fn migrate() -> Result<()> {
    let conn: Connection = Connection::open("btcmap.db")?;

    let schema_ver: i16 = conn.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
        row.get(0)
    })?;

    if schema_ver == 0 {
        conn.execute_batch(include_str!("../migrations/1.sql"))?;
        conn.execute_batch(&format!("PRAGMA user_version={}", 1))?;
    }

    println!("Database schema is up to date");
    Ok(())
}

fn drop() -> Result<()> {
    let db_url: &Path = Path::new("btcmap.db");
    println!("Removing btcmap.db");

    if !db_url.exists() {
        panic!("Database does not exist");
    } else {
        remove_file(db_url)?;
        println!("Database has been dropped");
    }

    Ok(())
}