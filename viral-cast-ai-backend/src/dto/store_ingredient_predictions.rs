use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct GenerateIngredientPredictionParams {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct IngredientWeatherBriefDto {
    pub summary: String,
    pub temp_min_c: Option<f32>,
    pub temp_max_c: Option<f32>,
    pub humidity_avg: Option<f32>,
    pub precipitation_total_mm: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct IngredientPredictionDto {
    pub ingredient_catalog_uuid: Uuid,
    pub ingredient_name: Option<String>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
    pub restock_label: String,
    pub restock_probability: Option<f32>,
    pub recommended_restock_qty: Decimal,
    pub current_stock_qty: Option<Decimal>,
    pub minimum_stock_qty: Option<Decimal>,
    pub llm_reasoning: Option<String>,
    pub forecast_error_margin_pct: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct StoreIngredientPredictionResponseDto {
    pub store_uuid: Uuid,
    pub region_code: Option<String>,
    pub timezone: Option<String>,
    pub weather: Option<IngredientWeatherBriefDto>,
    pub ingredients: Vec<IngredientPredictionDto>,
    pub llm_model: Option<String>,
    pub llm_summary: Option<String>,
    pub generated_at_ms: i64,
}
