use std::fs::{File, Metadata};
use std::io::Write;
use std::path::Path;
use anyhow::Result;
use reqwest::Response;
use rusqlite::{Connection, params, Transaction};
use serde_json::{Map, Value};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use crate::model::Place;
use crate::repository::PlaceRepository;

pub async fn cli_main(args: &[String]) -> Result<()> {
    match args.first() {
        None => println!("No options passed"),
        Some(first_arg) => {
            match first_arg.as_str() {
                "sync" => sync().await?,
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
        println!("Cached response exists");
        let metadata: Metadata = cached_response.metadata()?;
        let created: OffsetDateTime = metadata.modified()?.into();
        println!("Cached response was last modified at {}", created.format(&Rfc3339)?);
    } else {
        println!("Response cache is empty");
        println!("Querying OSM API, it could take a while...");

        let response: Response = client.post("https://overpass-api.de/api/interpreter")
            .body(r#"
                [out:json][timeout:300];
                (
                  node["payment:bitcoin"="yes"];
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

    let repo: PlaceRepository = PlaceRepository::new(Connection::open("btcmap.db")?);
    let ids: Vec<i64> = elements.iter().map(|it| it["id"].as_i64().unwrap()).collect();
    let cached_places: Vec<Place> = repo.select_all()?;

    for cached_place in &cached_places {
        if !ids.contains(&cached_place.id) {
            println!("Place with id {} was deleted from OSM", cached_place.id);
        }
    }

    let mut conn = Connection::open("btcmap.db")?;
    let tx: Transaction = conn.transaction()?;

    for place in elements {
        let id = place["id"].as_i64().unwrap();
        let lat: f64 = place["lat"].as_f64().unwrap();
        let lon: f64 = place["lon"].as_f64().unwrap();
        let empty_map: Map<String, Value> = Map::new();
        let tags: &Map<String, Value> = place["tags"].as_object().unwrap_or(&empty_map);

        let cached_place = cached_places.iter().find(|it| it.id == id);

        match cached_place {
            Some(cached_place) => {
                let cached_tags: String = serde_json::to_string(&cached_place.tags)?;
                let new_tags = serde_json::to_string(tags)?;

                if cached_tags != new_tags {
                    println!("Change detected");
                    println!("Cached tags:\n{}", cached_tags);
                    println!("New tags:\n{}", new_tags);

                    tx.execute(
                        "UPDATE places SET tags = ? WHERE id = ?",
                        params![new_tags, id],
                    )?;
                }
            }
            None => {
                println!("Place does not exist, inserting");
                println!("id: {}, lat: {}, lon {}", id, lat, lon);
                println!("Tags:\n{}", serde_json::to_string(tags)?);

                tx.execute(
                    "INSERT INTO places (id, lat, lon, tags) VALUES (?, ?, ?, ?)",
                    params![id, lat, lon, serde_json::to_string(tags)?],
                )?;
            }
        }
    }

    tx.commit()?;

    println!("Finished sync");
    Ok(())
}