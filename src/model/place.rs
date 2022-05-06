use serde::Serialize;

#[derive(Serialize)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub address: String,
    pub amenity: String,
    pub phone: String,
    pub website: String,
    pub opening_hours: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}