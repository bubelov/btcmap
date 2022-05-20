extern crate core;

mod db;
mod osm;
mod place;
mod sync;

use std::env;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use std::sync::Mutex;

use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::web::Json;
use directories::ProjectDirs;
use rusqlite::{Connection, OptionalExtension, Statement};
use serde_json::Value;
use crate::{place::Place, db::place_mapper};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if env::var("RUST_BACKTRACE").is_err() {
        env::set_var("RUST_BACKTRACE", "1");
    }

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let db_conn = Connection::open(get_db_file_path()).unwrap();

    match args.len() {
        1 => {
            let db_conn = web::Data::new(Mutex::new(db_conn));

            println!("Starting a server");
            HttpServer::new(move || {
                App::new()
                    .wrap(Logger::default())
                    .app_data(db_conn.clone())
                    .service(get_places)
                    .service(get_place)
                    .service(get_data)
            }).bind(("127.0.0.1", 8000))?.run().await
        }
        _ => {
            let db_conn = Connection::open(get_db_file_path()).unwrap();

            match args.get(1).unwrap().as_str() {
                "db" => { db::cli_main(&args[2..], db_conn); }
                "osm" => { osm::cli_main(&args[2..], db_conn).await; }
                "sync" => { sync::sync().await; }
                _ => { panic!("Unknown action"); }
            }

            Ok(())
        }
    }
}

#[derive(serde::Deserialize)]
struct GetPlacesArgs {
    created_or_updated_since: Option<String>,
}

#[actix_web::get("/places")]
async fn get_places(
    args: web::Query<GetPlacesArgs>,
    conn: web::Data<Mutex<Connection>>,
) -> Json<Vec<Place>> {
    let conn = conn.lock().unwrap();

    let places: Vec<Place> = match &args.created_or_updated_since {
        Some(created_or_updated_since) => {
            let query = "SELECT id, lat, lon, tags, created_at, updated_at, deleted_at FROM places WHERE updated_at > ? ORDER BY updated_at DESC";
            let mut stmt: Statement = conn.prepare(query).unwrap();
            stmt.query_map([created_or_updated_since], place_mapper()).unwrap().map(|row| row.unwrap()).collect()
        }
        None => {
            let query = "SELECT id, lat, lon, tags, created_at, updated_at, deleted_at FROM places ORDER BY updated_at DESC";
            let mut stmt: Statement = conn.prepare(query).unwrap();
            stmt.query_map([], place_mapper()).unwrap().map(|row| row.unwrap()).collect()
        }
    };

    Json(places)
}

#[actix_web::get("/data")]
async fn get_data() -> Json<Value> {
    let project_dirs: ProjectDirs = get_project_dirs();

    if !project_dirs.cache_dir().exists() {
        create_dir_all(project_dirs.cache_dir()).unwrap()
    }

    let last_response_path: PathBuf = project_dirs.cache_dir().join("data.json");
    let response: File = File::open(last_response_path).unwrap();
    let response: Value = serde_json::from_reader(response).unwrap();
    Json(response)
}

#[actix_web::get("/places/{id}")]
async fn get_place(
    path: web::Path<i64>,
    conn: web::Data<Mutex<Connection>>,
) -> Json<Option<Place>> {
    let id = path.into_inner();

    let query = "SELECT id, lat, lon, tags, created_at, updated_at, deleted_at FROM places WHERE id = ?";
    let place = conn.lock().unwrap().query_row(query, [id], place_mapper()).optional().unwrap();

    Json(place)
}

fn get_db_file_path() -> PathBuf {
    let project_dirs = get_project_dirs();

    if !project_dirs.data_dir().exists() {
        create_dir_all(project_dirs.data_dir()).unwrap()
    }

    project_dirs.data_dir().join("btcmap.db")
}

fn get_project_dirs() -> ProjectDirs {
    return ProjectDirs::from("org", "BTC Map", "BTC Map").unwrap();
}