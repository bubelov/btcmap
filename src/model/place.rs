use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct Place {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    pub tags: Value,
    pub created_at: String,
    pub updated_at: String,
}