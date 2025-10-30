use crate::models::weather_bmkg::{LocationInfo, RegionMaster, WeatherPrediction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherPredictionRequest {
    pub region_code: String, // Administrative area code (adm4)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherPredictionResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<WeatherData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherData {
    pub region_code: String,
    pub lokasi: LocationInfo,
    pub prakiraan_cuaca: Vec<WeatherPrediction>,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegionListRequest {
    pub provinsi: Option<String>,
    pub kabupaten_kota: Option<String>,
    pub kecamatan: Option<String>,
    pub search: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegionListResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<Vec<RegionMaster>>,
    pub pagination: Option<PaginationInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub current_page: i32,
    pub total_pages: i32,
    pub total_items: i64,
    pub items_per_page: i32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RegionDetailResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<RegionMaster>,
}

// CreateRegionRequest removed: there is no POST API to create regions.
