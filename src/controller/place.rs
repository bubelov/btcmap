use crate::model::Place;
use rocket::get;
use rocket::serde::json::Json;
use rusqlite::Connection;

#[get("/places")]
pub async fn get_places() -> Json<Vec<Place>> {
    let conn = Connection::open("btcmap.db").unwrap();
    let repo = crate::repository::PlaceRepository::new(conn);
    Json(repo.select_all().unwrap())
}

#[get("/places/<id>")]
pub async fn get_place(id: i64) -> Json<Option<Place>> {
    let conn = Connection::open("btcmap.db").unwrap();
    let repo = crate::repository::PlaceRepository::new(conn);
    Json(repo.select_by_id(id).unwrap())
}