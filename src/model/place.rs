use serde::Serialize;

#[derive(Serialize)]
pub struct Place {
    pub id: String,
    pub source: String,
    pub lat: f64,
    pub lon: f64,
}