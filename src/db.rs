use std::fs::remove_file;
use std::path::Path;
use anyhow::{Result, Context};

pub async fn cli_main(args: &[String]) -> Result<()> {
    match args.first() {
        None => println!("No options passed"),
        Some(first_arg) => {
            match first_arg.as_str() {
                "drop" => drop().context("Unable to drop database")?,
                _ => println!("Unknown command {first_arg}"),
            }
        }
    }

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