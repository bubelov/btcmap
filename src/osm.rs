use std::fs::File;
use std::io::Write;
use std::path::Path;
use anyhow::{Context, Result};
use reqwest::Response;
use rusqlite::{Connection, params};
use serde_json::{Map, Value};
use uuid::Uuid;

pub async fn cli_main(args: &[String]) -> Result<()> {
    match args.first() {
        None => println!("No options passed"),
        Some(first_arg) => {
            match first_arg.as_str() {
                "sync" => sync().await.context("Failed to sync with OSM")?,
                _ => println!("Unknown command {first_arg}"),
            }
        }
    }

    Ok(())
}

async fn sync() -> Result<()> {
    let client = reqwest::Client::new();

    let cached_response = Path::new("/tmp/cached-osm-response.json");

    if cached_response.exists() {
        println!("Cached response exists")
    } else {
        println!("Response cache is empty");
        println!("Querying OSM API, it could take a while...");

        let response: Response = client.post("https://overpass-api.de/api/interpreter")
            .body(r#"
                [out:json][timeout:300];
                (
                  node["payment:bitcoin"="yes"];
                  way["payment:bitcoin"="yes"];
                  relation["payment:bitcoin"="yes"];
                );
                out center;
            "#)
            .send()
            .await?;

        let mut file = File::create("/tmp/cached-osm-response.json")?;
        let response_body = response.bytes().await?;
        file.write_all(&response_body)?;
    }

    let cached_response: File = File::open("/tmp/cached-osm-response.json")?;
    let json: Value = serde_json::from_reader(cached_response)?;

    let elements: &Vec<Value> = json["elements"].as_array().unwrap();
    println!("Got {} elements", elements.len());

    let mut conn = Connection::open("btcmap.db")?;
    let tx = conn.transaction()?;

    for place in elements {
        let id = place["id"].as_i64().unwrap();
        let source: String = format!("osm,id={}", id);

        let osm_type = place["type"].as_str().unwrap();

        let lat: f64 = match osm_type {
            "node" => place["lat"].as_f64().unwrap(),
            _ => place["center"].as_object().unwrap()["lat"].as_f64().unwrap(),
        };

        let lon: f64 = match osm_type {
            "node" => place["lon"].as_f64().unwrap(),
            _ => place["center"].as_object().unwrap()["lon"].as_f64().unwrap(),
        };

        let empty_map: Map<String, Value> = Map::new();
        let tags: &Map<String, Value> = place["tags"].as_object().unwrap_or(&empty_map);
        let empty_str: Value = Value::String(String::new());
        let name: &str = tags.get("name").unwrap_or(&empty_str).as_str().unwrap();

        let exists: bool = tx.query_row("SELECT count(*) FROM places WHERE source = ?", [source.clone()], |row| {
            row.get(0)
        })?;

        if exists {
            println!("Place exists");
        } else {
            println!("Place does not exist, inserting");

            tx.execute(
                "INSERT INTO places (id, source, lat, lon, name) VALUES (?, ?, ?, ?, ?)",
                params![Uuid::new_v4().to_string(), source.clone(), lat, lon, name],
            )?;
        }
    }

    tx.commit()?;

    println!("Finished sync");
    Ok(())
}