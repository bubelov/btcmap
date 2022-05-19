use std::fs::{create_dir_all, File, Metadata};
use std::io::Write;
use std::path::PathBuf;
use directories::ProjectDirs;
use reqwest::Response;
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use crate::get_project_dirs;

pub async fn sync() {
    let project_dirs: ProjectDirs = get_project_dirs();

    if !project_dirs.cache_dir().exists() {
        create_dir_all(project_dirs.cache_dir()).unwrap()
    }

    let last_response_path: PathBuf = project_dirs.cache_dir().join("last-osm-response-v2.json");

    if last_response_path.exists() {
        println!("Found last OSM response at {}", last_response_path.to_str().unwrap());
        let metadata: Metadata = last_response_path.metadata().unwrap();
        let modified: OffsetDateTime = metadata.modified().unwrap().into();
        println!(
            "Last OSM response file was last modified at {}",
            modified.format(&Rfc3339).unwrap(),
        );
    } else {
        println!("There are no previously cached responses");
        println!("Querying OSM API, it could take a while...");
        let response: Response = reqwest::Client::new().post("https://overpass-api.de/api/interpreter")
            .body(r#"
                [out:json][timeout:300];
                (
                    node["payment:bitcoin"="yes"];
                    way["payment:bitcoin"="yes"];
                    relation["payment:bitcoin"="yes"];
                );
                out center meta;
            "#)
            .send()
            .await
            .unwrap();

        let mut file: File = File::create(&last_response_path).unwrap();
        let response_body = response.bytes().await.unwrap();
        file.write_all(&response_body).unwrap();
    }

    let response: File = File::open(last_response_path).unwrap();
    let elements: Value = serde_json::from_reader(response).unwrap();
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

    println!("Finished sync");
}