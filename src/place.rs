use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone)]
pub struct Place {
    pub id: i64,
    pub lat: f64,
    pub lon: f64,
    pub tags: Value,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: String,
}