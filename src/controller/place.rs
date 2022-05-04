use crate::model::Place;
use rocket::get;
use rocket::serde::json::Json;
use rusqlite::Connection;

#[get("/places")]
pub async fn index() -> Json<Vec<Place>> {
    let conn = Connection::open("btcmap.db").unwrap();
    let repo = crate::repository::PlaceRepository::new(conn);
    Json(repo.select_all().unwrap())
}