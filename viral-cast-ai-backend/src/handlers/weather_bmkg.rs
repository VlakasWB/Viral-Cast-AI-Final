use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use reqwest;
use serde::Deserialize;

use crate::dto::weather_bmkg::{
    PaginationInfo, RegionDetailResponse, RegionListRequest, RegionListResponse, WeatherData,
    WeatherPredictionRequest, WeatherPredictionResponse,
};
use crate::models::weather_bmkg::{BMKGApiResponse, BMKGWeatherResponse, RegionMaster};
use crate::repository::weather_bmkg as weather_repository;
use crate::AppState;

const BMKG_API_BASE_URL: &str = "https://api.bmkg.go.id/publik/prakiraan-cuaca";

// Utility: pick prioritized region_code list
pub async fn get_prioritized_regions(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let rows = weather_repository::get_prioritized_regions(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows))
}

pub async fn get_weather_prediction(
    Query(params): Query<WeatherPredictionRequest>,
    State(state): State<AppState>,
) -> Result<Json<WeatherPredictionResponse>, StatusCode> {
    // Validate region_code format
    if !is_valid_region_code(&params.region_code) {
        return Ok(Json(WeatherPredictionResponse {
            success: false,
            message: "Invalid region_code format. Use: XX.XX.XX.XXXX".to_string(),
            data: None,
        }));
    }

    // DB-first: try to read latest forecasts from bmkg_forecast when enabled
    if state.bmkg_db_first {
        if let Ok(opt_row) =
            weather_repository::get_latest_raw_forecast_run_json(&state.db, &params.region_code)
                .await
        {
            if let Some(raw_json) = opt_row {
                // Convert stored raw_json (BMKGWeatherResponse) to WeatherData response
                let bmkg_wr: BMKGWeatherResponse =
                    serde_json::from_value(raw_json).unwrap_or(BMKGWeatherResponse {
                        data: vec![],
                        lokasi: crate::models::weather_bmkg::LocationInfo {
                            adm1: "".to_string(),
                            adm2: "".to_string(),
                            adm3: "".to_string(),
                            adm4: "".to_string(),
                            provinsi: "".to_string(),
                            kotkab: "".to_string(),
                            kecamatan: "".to_string(),
                            desa: "".to_string(),
                            lon: 0.0,
                            lat: 0.0,
                            timezone: "".to_string(),
                            r#type: None,
                        },
                    });
                let weather_data = WeatherData {
                    region_code: params.region_code.clone(),
                    lokasi: bmkg_wr.lokasi,
                    prakiraan_cuaca: bmkg_wr.data,
                    last_updated: Utc::now().to_rfc3339(),
                };
                return Ok(Json(WeatherPredictionResponse {
                    success: true,
                    message: "Weather forecast data served from database".to_string(),
                    data: Some(weather_data),
                }));
            }
        }
    }

    // Fallback to live fetch dari BMKG API
    match fetch_bmkg_weather_data(&params.region_code).await {
        Ok(weather_response) => {
            // Optional: simpan raw BMKG JSON ke disk
            if state.bmkg_save_json {
                let dir = std::path::Path::new("data/json");
                if let Err(e) = std::fs::create_dir_all(dir) {
                    eprintln!("Failed to create data/json: {}", e);
                }
                let fname = format!(
                    "bmkg_{}_{}.json",
                    params.region_code.replace('.', "-"),
                    Utc::now().format("%Y%m%d%H%M%S")
                );
                let path = dir.join(fname);
                if let Ok(json_str) = serde_json::to_string_pretty(&weather_response) {
                    if let Err(e) = std::fs::write(&path, json_str) {
                        eprintln!("Failed to write JSON dump {}: {}", path.display(), e);
                    }
                }
            }

            // Persist fetched response to database (forecast_run dan forecast)
            let analysis_iso_opt = weather_response
                .data
                .first()
                .and_then(|w| w.analysis_date.clone());
            let analysis_ms = if let Some(iso_str) = analysis_iso_opt {
                parse_iso_to_ms(&iso_str).unwrap_or(Utc::now().timestamp_millis())
            } else {
                Utc::now().timestamp_millis()
            };
            let raw_json = serde_json::to_value(&weather_response)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if let Err(e) = weather_repository::insert_forecast_run(
                &state.db,
                &params.region_code,
                analysis_ms as i64,
                raw_json,
            )
            .await
            {
                eprintln!("Failed to insert bmkg_forecast_run: {}", e);
            }

            for pred in weather_response.data.iter() {
                if let Err(e) = weather_repository::insert_forecast(
                    &state.db,
                    &params.region_code,
                    analysis_ms as i64,
                    pred,
                )
                .await
                {
                    eprintln!("Failed to insert bmkg_forecast row: {}", e);
                }
            }

            let weather_data = WeatherData {
                region_code: params.region_code.clone(),
                lokasi: weather_response.lokasi,
                prakiraan_cuaca: weather_response.data,
                last_updated: Utc::now().to_rfc3339(),
            };

            Ok(Json(WeatherPredictionResponse {
                success: true,
                message: "Weather forecast data retrieved successfully".to_string(),
                data: Some(weather_data),
            }))
        }
        Err(e) => {
            eprintln!("Error fetching weather data: {}", e);
            Ok(Json(WeatherPredictionResponse {
                success: false,
                message: format!("Failed to fetch weather forecast data: {}", e),
                data: None,
            }))
        }
    }
}

pub async fn get_regions_list(
    Query(params): Query<RegionListRequest>,
    State(state): State<AppState>,
) -> Result<Json<RegionListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    match weather_repository::list_regions(&state.db, &params, limit, offset).await {
        Ok(region_masters) => {
            let total_count = weather_repository::count_regions(&state.db, &params)
                .await
                .unwrap_or(0);
            let total_pages = ((total_count as f64) / (limit as f64)).ceil() as i32;

            let pagination = PaginationInfo {
                current_page: page,
                total_pages,
                total_items: total_count,
                items_per_page: limit,
            };

            Ok(Json(RegionListResponse {
                success: true,
                message: "Region data retrieved successfully".to_string(),
                data: Some(region_masters),
                pagination: Some(pagination),
            }))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(Json(RegionListResponse {
                success: false,
                message: "Failed to retrieve region data".to_string(),
                data: None,
                pagination: None,
            }))
        }
    }
}

pub async fn get_region_by_id(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<RegionDetailResponse>, StatusCode> {
    match weather_repository::get_region_by_offset(&state.db, id - 1).await {
        Ok(Some(region_master)) => Ok(Json(RegionDetailResponse {
            success: true,
            message: "Region data retrieved successfully".to_string(),
            data: Some(region_master),
        })),
        Ok(None) => Ok(Json(RegionDetailResponse {
            success: false,
            message: "Region data not found".to_string(),
            data: None,
        })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(Json(RegionDetailResponse {
                success: false,
                message: "Failed to retrieve region data".to_string(),
                data: None,
            }))
        }
    }
}

// Create region API removed: all region data are managed via GET and seeding.

async fn fetch_bmkg_weather_data(
    region_code: &str,
) -> Result<BMKGWeatherResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let url = format!("{}?adm4={}", BMKG_API_BASE_URL, region_code);

    let response = client
        .get(&url)
        .header("User-Agent", "ViralCastAI/1.0")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("BMKG API returned status: {}", response.status()).into());
    }

    // Parse the actual BMKG API response structure
    let bmkg_api_response: BMKGApiResponse = response.json().await?;

    // Extract the first location data (should only be one for a specific adm4)
    if let Some(location_data) = bmkg_api_response.data.first() {
        // Flatten the nested weather data structure
        let mut all_predictions = Vec::new();

        for day_predictions in &location_data.cuaca {
            for prediction in day_predictions {
                all_predictions.push(prediction.clone());
            }
        }

        // Return our simplified structure
        Ok(BMKGWeatherResponse {
            data: all_predictions,
            lokasi: bmkg_api_response.lokasi,
        })
    } else {
        Err("No weather data found for the specified location".into())
    }
}

fn is_valid_region_code(kode: &str) -> bool {
    // Format: XX.XX.XX.XXXX (example: 31.71.03.1001)
    let parts: Vec<&str> = kode.split('.').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|part| part.chars().all(|c| c.is_ascii_digit()))
        && parts[0].len() == 2
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts[3].len() == 4
}

fn parse_iso_to_ms(iso: &str) -> Option<i64> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        return Some(dt.timestamp_millis());
    }
    chrono::DateTime::parse_from_str(iso, "%Y-%m-%d %H:%M:%S%z")
        .ok()
        .map(|dt| dt.timestamp_millis())
}

pub async fn post_weather_prediction(
    params: axum::extract::Query<WeatherPredictionRequest>,
    State(state): State<AppState>,
) -> Result<Json<WeatherPredictionResponse>, StatusCode> {
    if !is_valid_region_code(&params.region_code) {
        return Ok(Json(WeatherPredictionResponse {
            success: false,
            message: "Invalid region_code format. Use: XX.XX.XX.XXXX".to_string(),
            data: None,
        }));
    }

    match fetch_bmkg_weather_data(&params.region_code).await {
        Ok(weather_response) => {
            let analysis_iso_opt = weather_response
                .data
                .first()
                .and_then(|w| w.analysis_date.clone());
            let analysis_ms = if let Some(iso_str) = analysis_iso_opt {
                parse_iso_to_ms(&iso_str).unwrap_or(Utc::now().timestamp_millis())
            } else {
                Utc::now().timestamp_millis()
            };
            let raw_json = serde_json::to_value(&weather_response)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if let Err(e) = weather_repository::insert_forecast_run(
                &state.db,
                &params.region_code,
                analysis_ms as i64,
                raw_json,
            )
            .await
            {
                eprintln!("Failed to insert bmkg_forecast_run: {}", e);
            }

            for pred in weather_response.data.iter() {
                if let Err(e) = weather_repository::insert_forecast(
                    &state.db,
                    &params.region_code,
                    analysis_ms as i64,
                    pred,
                )
                .await
                {
                    eprintln!("Failed to insert bmkg_forecast row: {}", e);
                }
            }

            let weather_data = WeatherData {
                region_code: params.region_code.clone(),
                lokasi: weather_response.lokasi,
                prakiraan_cuaca: weather_response.data,
                last_updated: Utc::now().to_rfc3339(),
            };
            Ok(Json(WeatherPredictionResponse {
                success: true,
                message: "Weather forecast fetched and persisted".to_string(),
                data: Some(weather_data),
            }))
        }
        Err(e) => {
            eprintln!("Error fetching weather data: {}", e);
            Ok(Json(WeatherPredictionResponse {
                success: false,
                message: format!("Failed to fetch weather forecast data: {}", e),
                data: None,
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct UpdatePriorityRequest {
    region_code: String,
    priority: i32,
    active: Option<bool>,
}

pub async fn update_region_priority(
    State(state): State<AppState>,
    axum::extract::Json(req): axum::extract::Json<UpdatePriorityRequest>,
) -> Result<Json<bool>, StatusCode> {
    let now_ms: i64 = chrono::Utc::now().timestamp_millis();
    let res = weather_repository::upsert_area_priority(
        &state.db,
        &req.region_code,
        req.priority,
        req.active,
        now_ms,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(res > 0))
}
