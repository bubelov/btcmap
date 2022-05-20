use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use directories::ProjectDirs;
use reqwest::Response;
use serde_json::Value;
use crate::get_project_dirs;

pub async fn sync() {
    let project_dirs: ProjectDirs = get_project_dirs();

    if !project_dirs.cache_dir().exists() {
        create_dir_all(project_dirs.cache_dir()).unwrap()
    }

    let data_file_path: PathBuf = project_dirs.cache_dir().join("data.json");
    let response: Response = reqwest::Client::new()
        .get("https://raw.githubusercontent.com/bubelov/btcmap-data/main/data.json")
        .send()
        .await
        .unwrap();

    let mut data_file: File = File::create(&data_file_path).unwrap();
    let response_body = response.bytes().await.unwrap();
    data_file.write_all(&response_body).unwrap();

    let data_file: File = File::open(&data_file_path).unwrap();
    let elements: Value = serde_json::from_reader(data_file).unwrap();
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