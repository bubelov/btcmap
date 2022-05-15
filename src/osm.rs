use std::collections::HashSet;
use std::fs::{File, Metadata};
use std::io::Write;
use std::path::Path;
use reqwest::Response;
use rusqlite::{Connection, params, Statement, Transaction};
use serde_json::{Map, Value};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use crate::{db::place_mapper, Place};

pub async fn cli_main(
    args: &[String],
    db_conn: Connection,
) {
    match args.first() {
        Some(first_arg) => {
            match first_arg.as_str() {
                "sync" => { sync(db_conn).await; }
                _ => { panic!("Unknown action {first_arg}"); }
            }
        }
        None => { panic!("No osm actions passed"); }
    }
}

async fn sync(mut db_conn: Connection) {
    let client = reqwest::Client::new();

    let cached_response = Path::new("/tmp/cached-osm-response.json");

    if cached_response.exists() {
        println!("Cached response exists");
        let metadata: Metadata = cached_response.metadata().unwrap();
        let created: OffsetDateTime = metadata.modified().unwrap().into();
        println!("Cached response was last modified at {}", created.format(&Rfc3339).unwrap());
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
            .await
            .unwrap();

        let mut file = File::create("/tmp/cached-osm-response.json").unwrap();
        let response_body = response.bytes().await.unwrap();
        file.write_all(&response_body).unwrap();
    }

    let cached_response: File = File::open("/tmp/cached-osm-response.json").unwrap();

    let elements: Value = serde_json::from_reader(cached_response).unwrap();
    let elements: &Vec<Value> = elements["elements"].as_array().unwrap();
    println!("Got {} elements", elements.len());

    let nodes: Vec<&Value> = elements
        .iter()
        .filter(|it| it["type"].as_str().unwrap() == "node")
        .collect();

    let ways: Vec<&Value> = elements
        .iter()
        .filter(|it| it["type"].as_str().unwrap() == "way")
        .collect();

    let relations: Vec<&Value> = elements
        .iter()
        .filter(|it| it["type"].as_str().unwrap() == "relation")
        .collect();

    println!(
        "Of them:\nNodes {}\nWays {}\nRelations {}",
        nodes.len(),
        ways.len(),
        relations.len(),
    );

    let element_ids: HashSet<i64> = elements.iter().map(|it| it["id"].as_i64().unwrap()).collect();

    let tx: Transaction = db_conn.transaction().unwrap();
    let mut cached_places_stmt: Statement = tx.prepare("SELECT id, lat, lon, tags, created_at, updated_at, deleted_at FROM places").unwrap();
    let cached_places: Vec<Place> = cached_places_stmt.query_map([], place_mapper()).unwrap().map(|row| row.unwrap()).collect();
    drop(cached_places_stmt);

    for cached_place in &cached_places {
        if !element_ids.contains(&cached_place.id) && cached_place.deleted_at.is_empty() {
            println!("Place with id {} was deleted from OSM", cached_place.id);
            let query = "UPDATE places SET deleted_at = strftime('%Y-%m-%dT%H:%M:%SZ') WHERE id = ?";
            println!("Executing query:{}", query);
            tx.execute(query, params![cached_place.id]).unwrap();
        }
    }

    for element in elements {
        let id = element["id"].as_i64().unwrap();

        let (lat, lon) = match element["type"].as_str().unwrap() {
            "node" => (element["lat"].as_f64().unwrap(), element["lon"].as_f64().unwrap()),
            "way" | "relation" => (
                element["center"].as_object().unwrap()["lat"].as_f64().unwrap(),
                element["center"].as_object().unwrap()["lon"].as_f64().unwrap(),
            ),
            _ => panic!("Unknown element type"),
        };

        let empty_map: Map<String, Value> = Map::new();
        let tags: &Map<String, Value> = element["tags"].as_object().unwrap_or(&empty_map);

        let cached_place = cached_places.iter().find(|it| it.id == id);

        match cached_place {
            Some(cached_place) => {
                let cached_tags: String = serde_json::to_string(&cached_place.tags).unwrap();
                let fresh_tags = serde_json::to_string(tags).unwrap();

                if cached_tags != fresh_tags {
                    println!("Change detected");
                    println!("Cached tags:\n{}", cached_tags);
                    println!("Fresh tags:\n{}", fresh_tags);

                    tx.execute(
                        "UPDATE places SET tags = ? WHERE id = ?",
                        params![fresh_tags, id],
                    ).unwrap();
                }
            }
            None => {
                println!("Place does not exist, inserting");
                println!("id: {}, lat: {}, lon {}", id, lat, lon);
                println!("Tags:\n{}", serde_json::to_string(tags).unwrap());

                tx.execute(
                    "INSERT INTO places (id, lat, lon, tags) VALUES (?, ?, ?, ?)",
                    params![id, lat, lon, serde_json::to_string(tags).unwrap()],
                ).unwrap();
            }
        }
    }

    tx.commit().unwrap();
    println!("Finished sync");
}