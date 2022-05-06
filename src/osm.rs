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
        let addr_housenumber: &str = tags.get("addr:housenumber").unwrap_or(&empty_str).as_str().unwrap();
        let addr_street: &str = tags.get("addr:street").unwrap_or(&empty_str).as_str().unwrap();
        let mut addr = String::new();
        if addr_housenumber.len() > 0 && addr_street.len() > 0 {
            addr.push_str(addr_housenumber);
            addr.push_str(" ");
            addr.push_str(addr_street);
        }
        let amenity: &str = tags.get("amenity").unwrap_or(&empty_str).as_str().unwrap();
        let phone: &str = tags.get("phone").unwrap_or(&empty_str).as_str().unwrap();
        let website: &str = tags.get("website").unwrap_or(&empty_str).as_str().unwrap();
        let opening_hours: &str = tags.get("opening_hours").unwrap_or(&empty_str).as_str().unwrap();

        let exists: bool = tx.query_row("SELECT count(*) FROM places WHERE source = ?", [source.clone()], |row| {
            row.get(0)
        })?;

        if exists {
            println!("Place exists");
        } else {
            println!("Place does not exist, inserting");

            tx.execute(
                "INSERT INTO places (id, name, lat, lon, address, amenity, phone, website, opening_hours, source) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![Uuid::new_v4().to_string(), name.clone(), lat, lon, addr.clone(), amenity.clone(), phone.clone(), website.clone(), opening_hours.clone(), source.clone()],
            )?;
        }
    }

    tx.commit()?;

    println!("Finished sync");
    Ok(())
}