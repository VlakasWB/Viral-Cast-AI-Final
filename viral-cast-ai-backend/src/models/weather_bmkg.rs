use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeatherPrediction {
    pub datetime: String,
    pub utc_datetime: String,
    pub local_datetime: String,
    pub t: f64,       // Air temperature in °C
    pub hu: f64,      // Air humidity in %
    pub weather: i32, // Weather code
    #[serde(default)]
    pub weather_desc: Option<String>, // Weather condition in Indonesian
    #[serde(default)]
    pub weather_desc_en: Option<String>, // Weather condition in English
    pub ws: f64,      // Wind speed in km/h
    #[serde(default)]
    pub wd: Option<String>, // Wind direction from
    #[serde(default)]
    pub wd_to: Option<String>, // Wind direction to
    pub wd_deg: f64,  // Wind direction in degrees
    pub tcc: f64,     // Cloud cover in %
    pub vs: f64,      // Visibility in meters
    #[serde(default)]
    pub vs_text: Option<String>, // Visibility distance text
    pub tp: f64,      // Rainfall
    #[serde(default)]
    pub time_index: Option<String>, // Time index
    #[serde(default)]
    pub analysis_date: Option<String>, // Forecast data production time in UTC
    #[serde(default)]
    pub image: Option<String>, // Weather icon URL
}

// BMKG API response structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BMKGApiResponse {
    pub lokasi: LocationInfo,
    pub data: Vec<BMKGLocationData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BMKGLocationData {
    pub lokasi: LocationInfo,
    pub cuaca: Vec<Vec<WeatherPrediction>>,
}

// Flattened structure used for client responses
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BMKGWeatherResponse {
    pub data: Vec<WeatherPrediction>,
    pub lokasi: LocationInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationInfo {
    pub adm1: String,      // Province code
    pub adm2: String,      // Regency code
    pub adm3: String,      // District code
    pub adm4: String,      // Village code
    pub provinsi: String,  // Province name
    pub kotkab: String,    // Regency/City name
    pub kecamatan: String, // District name
    pub desa: String,      // Village name
    pub lon: f64,
    pub lat: f64,
    pub timezone: String,
    #[serde(default)]
    pub r#type: Option<String>,
}

// Region models
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Province {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Regency {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
    pub province_uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct District {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
    pub regency_uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Village {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
    pub district_uuid: Uuid,
}

// View models with hierarchical information
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct ProvinceView {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct RegencyView {
    pub code: String,
    pub name: String,
    pub province_code: String,
    pub province_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct DistrictView {
    pub code: String,
    pub name: String,
    pub regency_code: String,
    pub regency_name: String,
    pub province_code: String,
    pub province_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct VillageView {
    pub code: String,
    pub name: String,
    pub district_code: String,
    pub district_name: String,
    pub regency_code: String,
    pub regency_name: String,
    pub province_code: String,
    pub province_name: String,
}
// Keep the old RegionMaster for backward compatibility during transition
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct RegionMaster {
    pub id: i32,
    pub kode_wilayah: String,     // Administrative area code (adm4)
    pub provinsi: String,         // Province (adm1)
    pub kabupaten_kota: String,   // Regency/City (adm2)
    pub kecamatan: String,        // District (adm3)
    pub kelurahan_desa: String,   // Village (adm4)
    pub kotkab: String,           // Regency/City code
    pub latitude: Option<f64>,    // Changed to Option<f64> for compatibility
    pub longitude: Option<f64>,   // Changed to Option<f64> for compatibility
    pub timezone: Option<String>, // Changed to Option<String> for compatibility
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
