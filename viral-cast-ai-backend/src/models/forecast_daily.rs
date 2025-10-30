use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ForecastMethod {
    SMA,
    WMA,
    EMA,
    LinearRegression,
    MultivariateRegression,
}

impl ForecastMethod {
    pub fn to_string(&self) -> String {
        match self {
            ForecastMethod::SMA => "SMA".to_string(),
            ForecastMethod::WMA => "WMA".to_string(),
            ForecastMethod::EMA => "EMA".to_string(),
            ForecastMethod::LinearRegression => "LINEAR_REGRESSION".to_string(),
            ForecastMethod::MultivariateRegression => "MULTIVARIATE_REGRESSION".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "SMA" => Some(ForecastMethod::SMA),
            "WMA" => Some(ForecastMethod::WMA),
            "EMA" => Some(ForecastMethod::EMA),
            "LINEAR_REGRESSION" => Some(ForecastMethod::LinearRegression),
            "MULTIVARIATE_REGRESSION" => Some(ForecastMethod::MultivariateRegression),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ForecastDaily {
    pub uuid: Uuid,
    pub product_uuid: Uuid,
    pub date_ts: i64,
    pub method: String,
    pub window_size: Option<i32>,
    pub params: Option<serde_json::Value>,
    pub forecast_qty: rust_decimal::Decimal,
    pub conf_low: Option<rust_decimal::Decimal>,
    pub conf_high: Option<rust_decimal::Decimal>,
    pub mae: Option<rust_decimal::Decimal>,
    pub mape: Option<rust_decimal::Decimal>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ForecastDailyWithProduct {
    pub uuid: Uuid,
    pub product_uuid: Uuid,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub date_ts: i64,
    pub method: String,
    pub window_size: Option<i32>,
    pub params: Option<serde_json::Value>,
    pub forecast_qty: rust_decimal::Decimal,
    pub conf_low: Option<rust_decimal::Decimal>,
    pub conf_high: Option<rust_decimal::Decimal>,
    pub mae: Option<rust_decimal::Decimal>,
    pub mape: Option<rust_decimal::Decimal>,
    pub accuracy_score: Option<f64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForecastDailySummary {
    pub uuid: Uuid,
    pub product_uuid: Uuid,
    pub product_name: Option<String>,
    pub date_ts: i64,
    pub date_str: String,
    pub method: String,
    pub forecast_qty: rust_decimal::Decimal,
    pub conf_low: Option<rust_decimal::Decimal>,
    pub conf_high: Option<rust_decimal::Decimal>,
    pub accuracy_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForecastAccuracy {
    pub method: String,
    pub avg_mae: Option<rust_decimal::Decimal>,
    pub avg_mape: Option<rust_decimal::Decimal>,
    pub accuracy_score: f64,
    pub forecast_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductForecastStats {
    pub product_uuid: Uuid,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub total_forecasts: i64,
    pub methods_used: Vec<String>,
    pub best_method: Option<String>,
    pub best_accuracy: Option<f64>,
    pub avg_forecast_qty: rust_decimal::Decimal,
    pub forecast_range_days: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForecastComparison {
    pub date_ts: i64,
    pub date_str: String,
    pub product_uuid: Uuid,
    pub product_name: Option<String>,
    pub actual_qty: Option<rust_decimal::Decimal>,
    pub forecasts: Vec<ForecastMethodResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForecastMethodResult {
    pub method: String,
    pub forecast_qty: rust_decimal::Decimal,
    pub conf_low: Option<rust_decimal::Decimal>,
    pub conf_high: Option<rust_decimal::Decimal>,
    pub accuracy: Option<f64>,
    pub error: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForecastTrend {
    pub product_uuid: Uuid,
    pub product_name: Option<String>,
    pub method: String,
    pub trend_direction: String, // "increasing", "decreasing", "stable"
    pub trend_strength: f64,     // 0.0 to 1.0
    pub forecast_points: Vec<ForecastPoint>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForecastPoint {
    pub date_ts: i64,
    pub date_str: String,
    pub forecast_qty: rust_decimal::Decimal,
    pub conf_low: Option<rust_decimal::Decimal>,
    pub conf_high: Option<rust_decimal::Decimal>,
}
