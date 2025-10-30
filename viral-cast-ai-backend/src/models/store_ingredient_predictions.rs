use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoreIngredientPrediction {
    pub uuid: Uuid,
    pub store_uuid: Uuid,
    pub ingredient_catalog_uuid: Uuid,
    pub region_code: Option<String>,
    pub restock_label: String,
    pub restock_probability: Option<f32>,
    pub recommended_restock_qty: Decimal,
    pub current_stock_qty: Option<Decimal>,
    pub minimum_stock_qty: Option<Decimal>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
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
pub struct StoreIngredientPredictionWithIngredient {
    pub uuid: Uuid,
    pub store_uuid: Uuid,
    pub ingredient_catalog_uuid: Uuid,
    pub region_code: Option<String>,
    pub restock_label: String,
    pub restock_probability: Option<f32>,
    pub recommended_restock_qty: Decimal,
    pub current_stock_qty: Option<Decimal>,
    pub minimum_stock_qty: Option<Decimal>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
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
    pub ingredient_name: Option<String>,
}
