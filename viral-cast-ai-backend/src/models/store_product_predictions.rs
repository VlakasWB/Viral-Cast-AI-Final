use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoreProductPrediction {
    pub uuid: Uuid,
    pub store_uuid: Uuid,
    pub product_uuid: Uuid,
    pub region_code: Option<String>,
    pub demand_label: String,
    pub demand_probability: Option<f32>,
    pub recommended_stock_qty: Decimal,
    pub weather_summary: Option<String>,
    pub weather_temp_min_c: Option<f32>,
    pub weather_temp_max_c: Option<f32>,
    pub weather_precip_mm: Option<f32>,
    pub weather_humidity: Option<f32>,
    pub llm_reasoning: Option<String>,
    pub llm_model: Option<String>,
    pub llm_prompt: Option<serde_json::Value>,
    pub llm_response: Option<serde_json::Value>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoreProductPredictionWithProduct {
    pub uuid: Uuid,
    pub store_uuid: Uuid,
    pub product_uuid: Uuid,
    pub region_code: Option<String>,
    pub demand_label: String,
    pub demand_probability: Option<f32>,
    pub recommended_stock_qty: Decimal,
    pub weather_summary: Option<String>,
    pub weather_temp_min_c: Option<f32>,
    pub weather_temp_max_c: Option<f32>,
    pub weather_precip_mm: Option<f32>,
    pub weather_humidity: Option<f32>,
    pub llm_reasoning: Option<String>,
    pub llm_model: Option<String>,
    pub llm_prompt: Option<serde_json::Value>,
    pub llm_response: Option<serde_json::Value>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub product_price: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSnapshot {
    pub uuid: Uuid,
    pub name: String,
    pub sku: Option<String>,
    pub price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSlotSnapshot {
    pub valid_ms: i64,
    pub time_index: Option<String>,
    pub temperature_c: Option<f32>,
    pub humidity_pct: Option<f32>,
    pub precipitation_mm: Option<f32>,
    pub wind_speed_kmh: Option<f32>,
    pub weather_code: Option<i16>,
    pub weather_desc_id: Option<String>,
    pub weather_desc_en: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherDailyAggregate {
    pub summary: String,
    pub temp_min_c: Option<f32>,
    pub temp_max_c: Option<f32>,
    pub humidity_avg: Option<f32>,
    pub precipitation_total_mm: Option<f32>,
    pub slots: Vec<WeatherSlotSnapshot>,
}
