use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct GenerateStorePredictionParams {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct WeatherBriefDto {
    pub summary: String,
    pub temp_min_c: Option<f32>,
    pub temp_max_c: Option<f32>,
    pub humidity_avg: Option<f32>,
    pub precipitation_total_mm: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProductPredictionDto {
    pub product_uuid: Uuid,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub demand_label: String,
    pub demand_probability: Option<f32>,
    pub recommended_stock_qty: Decimal,
    pub llm_reasoning: Option<String>,
    pub forecast_error_margin_pct: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct StorePredictionResponseDto {
    pub store_uuid: Uuid,
    pub region_code: Option<String>,
    pub timezone: Option<String>,
    pub weather: Option<WeatherBriefDto>,
    pub products: Vec<ProductPredictionDto>,
    pub llm_model: Option<String>,
    pub llm_summary: Option<String>,
    pub generated_at_ms: i64,
}
