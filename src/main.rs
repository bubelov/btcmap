extern crate core;

mod db;
mod osm;
mod place_repository;
mod place;

use std::env;

use actix_web::{App, HttpServer};
use actix_web::web::Json;
use rusqlite::Connection;
use crate::place::Place;
use crate::place_repository::PlaceRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            HttpServer::new(|| {
                App::new()
                    .service(get_places)
                    .service(get_place)
            }).bind(("127.0.0.1", 8000))?.run().await
        }
        _ => {
            match args.get(1).unwrap().as_str() {
                "db" => db::cli_main(&args[2..]).await.unwrap_or_else(|e| {
                    panic!("{e}");
                }),
                "osm" => osm::cli_main(&args[2..]).await.unwrap_or_else(|e| {
                    panic!("{e}");
                }),
                _ => {
                    panic!("Unknown action");
                }
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
async fn get_places(args: actix_web::web::Query<GetPlacesArgs>) -> Json<Vec<Place>> {
    let conn = Connection::open("btcmap.db").unwrap();
    let repo = PlaceRepository::new(conn);

    let places: Vec<Place> = match &args.created_or_updated_since {
        Some(created_or_updated_since) => {
            repo.select_all().unwrap().iter().filter(|it| &it.updated_at >= created_or_updated_since).cloned().collect()
        }
        None => {
            repo.select_all().unwrap()
        }
    };

    Json(places)
}

#[actix_web::get("/places/{id}")]
async fn get_place(path: actix_web::web::Path<i64>) -> Json<Option<Place>> {
    let id = path.into_inner();
    let conn = Connection::open("btcmap.db").unwrap();
    let repo = PlaceRepository::new(conn);
    Json(repo.select_by_id(id).unwrap())
}