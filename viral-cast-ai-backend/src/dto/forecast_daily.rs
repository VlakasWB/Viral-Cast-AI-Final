use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::forecast_daily::{
    ForecastAccuracy, ForecastDaily, ForecastDailyWithProduct, ForecastMethodResult, ForecastPoint,
    ProductForecastStats,
};

// Request DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateForecastRequest {
    pub product_uuid: Uuid,
    #[validate(custom = "validate_date_range")]
    pub date_ts: NaiveDate,
    #[validate(custom = "validate_forecast_method")]
    pub method: String, // moving_average, exponential_smoothing, linear_regression, arima
    #[validate(range(min = 1, max = 365, message = "Window size must be between 1 and 365"))]
    pub window_size: i32,
    #[validate(custom = "validate_forecast_params")]
    pub params: Option<serde_json::Value>,
    #[validate(range(
        min = 0.0,
        max = 1000000.0,
        message = "Forecast quantity must be between 0 and 1,000,000"
    ))]
    pub forecast_qty: f64,
    #[validate(range(
        min = 0.0,
        max = 1000000.0,
        message = "Confidence low must be between 0 and 1,000,000"
    ))]
    pub conf_low: Option<f64>,
    #[validate(range(
        min = 0.0,
        max = 1000000.0,
        message = "Confidence high must be between 0 and 1,000,000"
    ))]
    pub conf_high: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0, message = "MAE must be between 0 and 100"))]
    pub mae: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0, message = "MAPE must be between 0 and 100"))]
    pub mape: Option<f64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateForecastRequest {
    #[validate(range(
        min = 0.0,
        max = 1000000.0,
        message = "Forecast quantity must be between 0 and 1,000,000"
    ))]
    pub forecast_qty: Option<f64>,
    #[validate(range(
        min = 0.0,
        max = 1000000.0,
        message = "Confidence low must be between 0 and 1,000,000"
    ))]
    pub conf_low: Option<f64>,
    #[validate(range(
        min = 0.0,
        max = 1000000.0,
        message = "Confidence high must be between 0 and 1,000,000"
    ))]
    pub conf_high: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0, message = "MAE must be between 0 and 100"))]
    pub mae: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0, message = "MAPE must be between 0 and 100"))]
    pub mape: Option<f64>,
    #[validate(custom = "validate_forecast_params")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForecastListRequest {
    pub product_uuid: Option<Uuid>,
    #[validate(custom = "validate_date_range")]
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    #[validate(custom = "validate_forecast_method")]
    pub method: Option<String>,
    #[validate(range(min = 1, max = 100, message = "Page size must be between 1 and 100"))]
    pub page_size: Option<i64>,
    #[validate(range(min = 1, message = "Page must be positive"))]
    pub page: Option<i64>,
    #[validate(custom = "validate_forecast_sort_by")]
    pub sort_by: Option<String>, // date_ts, forecast_qty, mae, mape
    #[validate(custom = "validate_sort_order")]
    pub sort_order: Option<String>, // asc, desc
}

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateForecastRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Product UUIDs list must contain between 1 and 50 items"
    ))]
    pub product_uuids: Vec<Uuid>,
    #[validate(custom = "validate_date_range")]
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[validate(length(min = 1, message = "Methods list cannot be empty"))]
    #[validate(custom = "validate_forecast_methods")]
    pub methods: Vec<String>, // moving_average, exponential_smoothing, linear_regression, arima
    #[validate(range(min = 7, max = 365, message = "Window size must be between 7 and 365"))]
    pub window_size: Option<i32>,
    #[validate(range(
        min = 0.5,
        max = 0.99,
        message = "Confidence level must be between 0.5 and 0.99"
    ))]
    pub confidence_level: Option<f64>, // 0.95 for 95% confidence interval
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForecastAccuracyRequest {
    pub product_uuid: Option<Uuid>,
    #[validate(custom = "validate_forecast_method")]
    pub method: Option<String>,
    #[validate(custom = "validate_date_range")]
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForecastComparisonRequest {
    pub product_uuid: Uuid,
    #[validate(custom = "validate_date_range")]
    pub date_ts: NaiveDate,
    #[validate(length(
        min = 1,
        max = 10,
        message = "Methods list must contain between 1 and 10 items"
    ))]
    #[validate(custom = "validate_forecast_methods")]
    pub methods: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForecastTrendRequest {
    pub product_uuid: Uuid,
    #[validate(custom = "validate_date_range")]
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[validate(custom = "validate_forecast_method")]
    pub method: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForecastBatchRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Product UUIDs list must contain between 1 and 100 items"
    ))]
    pub product_uuids: Vec<Uuid>,
    #[validate(custom = "validate_date_range")]
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[validate(custom = "validate_forecast_method")]
    pub method: String,
    #[validate(range(min = 7, max = 365, message = "Window size must be between 7 and 365"))]
    pub window_size: i32,
    #[validate(custom = "validate_forecast_params")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForecastValidationRequest {
    pub product_uuid: Uuid,
    #[validate(custom = "validate_date_range")]
    pub validation_start: NaiveDate,
    pub validation_end: NaiveDate,
    #[validate(custom = "validate_forecast_method")]
    pub method: String,
    #[validate(range(min = 7, max = 365, message = "Window size must be between 7 and 365"))]
    pub window_size: i32,
    #[validate(range(
        min = 0.5,
        max = 0.99,
        message = "Confidence level must be between 0.5 and 0.99"
    ))]
    pub confidence_level: Option<f64>,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct ForecastDailyResponse {
    pub uuid: Uuid,
    pub product_uuid: Uuid,
    pub date_ts: NaiveDate,
    pub method: String,
    pub window_size: i32,
    pub params: Option<serde_json::Value>,
    pub forecast_qty: f64,
    pub conf_low: Option<f64>,
    pub conf_high: Option<f64>,
    pub mae: Option<f64>,
    pub mape: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ForecastDaily> for ForecastDailyResponse {
    fn from(forecast: ForecastDaily) -> Self {
        Self {
            uuid: forecast.uuid,
            product_uuid: forecast.product_uuid,
            date_ts: DateTime::from_timestamp(forecast.date_ts, 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
                .date_naive(),
            method: forecast.method,
            window_size: forecast.window_size.unwrap_or(0),
            params: forecast.params,
            forecast_qty: forecast.forecast_qty.to_f64().unwrap_or(0.0),
            conf_low: forecast.conf_low.map(|d| d.to_f64().unwrap_or(0.0)),
            conf_high: forecast.conf_high.map(|d| d.to_f64().unwrap_or(0.0)),
            mae: forecast.mae.map(|d| d.to_f64().unwrap_or(0.0)),
            mape: forecast.mape.map(|d| d.to_f64().unwrap_or(0.0)),
            created_at: DateTime::from_timestamp(forecast.created_at.expect("REASON"), 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap()),
            updated_at: DateTime::from_timestamp(forecast.updated_at.expect("REASON"), 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ForecastDailyWithProductResponse {
    pub uuid: Uuid,
    pub product_uuid: Uuid,
    pub product_name: String,
    pub product_sku: Option<String>,
    pub date_ts: NaiveDate,
    pub method: String,
    pub window_size: i32,
    pub params: Option<serde_json::Value>,
    pub forecast_qty: f64,
    pub conf_low: Option<f64>,
    pub conf_high: Option<f64>,
    pub mae: Option<f64>,
    pub mape: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ForecastDailyWithProduct> for ForecastDailyWithProductResponse {
    fn from(forecast: ForecastDailyWithProduct) -> Self {
        Self {
            uuid: forecast.uuid,
            product_uuid: forecast.product_uuid,
            product_name: forecast.product_name.unwrap_or_default(),
            product_sku: forecast.product_sku,
            date_ts: DateTime::from_timestamp(forecast.date_ts, 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
                .date_naive(),
            method: forecast.method,
            window_size: forecast.window_size.unwrap_or(0),
            params: forecast.params,
            forecast_qty: forecast.forecast_qty.to_f64().unwrap_or(0.0),
            conf_low: forecast.conf_low.map(|d| d.to_f64().unwrap_or(0.0)),
            conf_high: forecast.conf_high.map(|d| d.to_f64().unwrap_or(0.0)),
            mae: forecast.mae.map(|d| d.to_f64().unwrap_or(0.0)),
            mape: forecast.mape.map(|d| d.to_f64().unwrap_or(0.0)),
            created_at: DateTime::from_timestamp(forecast.created_at.expect("REASON"), 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap()),
            updated_at: DateTime::from_timestamp(forecast.updated_at.expect("REASON"), 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ForecastListResponse {
    pub data: Vec<ForecastDailyWithProductResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize)]
pub struct ForecastAccuracyResponse {
    pub method: String,
    pub avg_mae: f64,
    pub avg_mape: f64,
    pub accuracy_score: f64,
    pub forecast_count: i64,
    pub best_performing_products: Vec<ProductAccuracyResponse>,
    pub worst_performing_products: Vec<ProductAccuracyResponse>,
}

#[derive(Debug, Serialize)]
pub struct ProductAccuracyResponse {
    pub product_uuid: Uuid,
    pub product_name: String,
    pub mae: f64,
    pub mape: f64,
    pub accuracy_score: f64,
}

impl From<ForecastAccuracy> for ForecastAccuracyResponse {
    fn from(accuracy: ForecastAccuracy) -> Self {
        Self {
            method: accuracy.method,
            avg_mae: accuracy
                .avg_mae
                .map(|d| d.to_f64().unwrap_or(0.0))
                .unwrap_or(0.0),
            avg_mape: accuracy
                .avg_mape
                .map(|d| d.to_f64().unwrap_or(0.0))
                .unwrap_or(0.0),
            accuracy_score: accuracy.accuracy_score,
            forecast_count: accuracy.forecast_count,
            best_performing_products: vec![], // Will be populated separately
            worst_performing_products: vec![], // Will be populated separately
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProductForecastStatsResponse {
    pub product_uuid: Uuid,
    pub product_name: String,
    pub total_forecasts: i64,
    pub avg_forecast_qty: f64,
    pub avg_mae: f64,
    pub avg_mape: f64,
    pub best_method: String,
    pub best_method_accuracy: f64,
    pub forecast_trend: String, // increasing, decreasing, stable
}

impl From<ProductForecastStats> for ProductForecastStatsResponse {
    fn from(stats: ProductForecastStats) -> Self {
        Self {
            product_uuid: stats.product_uuid,
            product_name: stats.product_name.unwrap_or_default(),
            total_forecasts: stats.total_forecasts,
            avg_forecast_qty: stats.avg_forecast_qty.to_f64().unwrap_or(0.0),
            avg_mae: 0.0,  // Default value since field doesn't exist in model
            avg_mape: 0.0, // Default value since field doesn't exist in model
            best_method: stats.best_method.unwrap_or_default(),
            best_method_accuracy: stats.best_accuracy.unwrap_or(0.0),
            forecast_trend: "stable".to_string(), // Default value since field doesn't exist in model
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ForecastComparisonResponse {
    pub product_uuid: Uuid,
    pub product_name: String,
    pub date_ts: NaiveDate,
    pub actual_qty: Option<f64>,
    pub forecasts: Vec<ForecastMethodResultResponse>,
    pub best_forecast: Option<ForecastMethodResultResponse>,
}

#[derive(Debug, Serialize)]
pub struct ForecastMethodResultResponse {
    pub method: String,
    pub forecast_qty: f64,
    pub conf_low: Option<f64>,
    pub conf_high: Option<f64>,
    pub mae: Option<f64>,
    pub mape: Option<f64>,
    pub accuracy_score: Option<f64>,
}

impl From<ForecastMethodResult> for ForecastMethodResultResponse {
    fn from(result: ForecastMethodResult) -> Self {
        Self {
            method: result.method,
            forecast_qty: result.forecast_qty.to_f64().unwrap_or(0.0),
            conf_low: result.conf_low.map(|d| d.to_f64().unwrap_or(0.0)),
            conf_high: result.conf_high.map(|d| d.to_f64().unwrap_or(0.0)),
            mae: None,  // Default value since field doesn't exist in model
            mape: None, // Default value since field doesn't exist in model
            accuracy_score: result.accuracy,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ForecastTrendResponse {
    pub product_uuid: Uuid,
    pub product_name: String,
    pub method: String,
    pub trend_data: Vec<ForecastPointResponse>,
    pub trend_direction: String,  // up, down, stable
    pub confidence_trend: String, // increasing, decreasing, stable
    pub avg_accuracy: f64,
}

#[derive(Debug, Serialize)]
pub struct ForecastPointResponse {
    pub date_ts: NaiveDate,
    pub forecast_qty: f64,
    pub conf_low: Option<f64>,
    pub conf_high: Option<f64>,
    pub actual_qty: Option<f64>,
    pub accuracy: Option<f64>,
}

impl From<ForecastPoint> for ForecastPointResponse {
    fn from(point: ForecastPoint) -> Self {
        Self {
            date_ts: DateTime::from_timestamp(point.date_ts, 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
                .date_naive(),
            forecast_qty: point.forecast_qty.to_f64().unwrap_or(0.0),
            conf_low: point.conf_low.map(|d| d.to_f64().unwrap_or(0.0)),
            conf_high: point.conf_high.map(|d| d.to_f64().unwrap_or(0.0)),
            actual_qty: None, // Default value since field doesn't exist in model
            accuracy: None,   // Default value since field doesn't exist in model
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GenerateForecastResponse {
    pub generated_forecasts: i64,
    pub products_processed: i64,
    pub methods_used: Vec<String>,
    pub date_range: DateRangeResponse,
    pub summary: ForecastGenerationSummaryResponse,
}

#[derive(Debug, Serialize)]
pub struct DateRangeResponse {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_days: i64,
}

#[derive(Debug, Serialize)]
pub struct ForecastGenerationSummaryResponse {
    pub avg_forecast_qty: f64,
    pub total_forecast_qty: f64,
    pub avg_confidence_interval: f64,
    pub methods_performance: Vec<MethodPerformanceResponse>,
}

#[derive(Debug, Serialize)]
pub struct MethodPerformanceResponse {
    pub method: String,
    pub forecasts_generated: i64,
    pub avg_forecast_qty: f64,
    pub avg_confidence_width: f64,
}

// Enhanced Validation functions
pub fn validate_date_range(date: &NaiveDate) -> Result<(), ValidationError> {
    let current_date = chrono::Utc::now().date_naive();
    let min_date = chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

    if *date < min_date {
        return Err(ValidationError::new("Date cannot be before 2020-01-01"));
    }

    if *date
        > current_date
            .checked_add_days(chrono::Days::new(365))
            .unwrap_or(current_date)
    {
        return Err(ValidationError::new(
            "Date cannot be more than 1 year in the future",
        ));
    }

    Ok(())
}

pub fn validate_forecast_method(method: &str) -> Result<(), ValidationError> {
    let valid_methods = [
        "moving_average",
        "exponential_smoothing",
        "linear_regression",
        "arima",
        "seasonal_arima",
        "prophet",
    ];
    if valid_methods.contains(&method) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid forecast method. Must be one of: moving_average, exponential_smoothing, linear_regression, arima, seasonal_arima, prophet"))
    }
}

pub fn validate_forecast_methods(methods: &Vec<String>) -> Result<(), ValidationError> {
    let valid_methods = [
        "moving_average",
        "exponential_smoothing",
        "linear_regression",
        "arima",
        "seasonal_arima",
        "prophet",
    ];
    for method in methods {
        if !valid_methods.contains(&method.as_str()) {
            return Err(ValidationError::new("Invalid forecast method. Must be one of: moving_average, exponential_smoothing, linear_regression, arima, seasonal_arima, prophet"));
        }
    }
    Ok(())
}

pub fn validate_forecast_sort_by(sort_by: &str) -> Result<(), ValidationError> {
    let valid_sorts = [
        "date_ts",
        "forecast_qty",
        "mae",
        "mape",
        "created_at",
        "updated_at",
    ];
    if valid_sorts.contains(&sort_by) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid sort_by field. Must be one of: date_ts, forecast_qty, mae, mape, created_at, updated_at"))
    }
}

pub fn validate_sort_order(sort_order: &str) -> Result<(), ValidationError> {
    let valid_orders = ["asc", "desc"];
    if valid_orders.contains(&sort_order) {
        Ok(())
    } else {
        Err(ValidationError::new(
            "Invalid sort_order. Must be either 'asc' or 'desc'",
        ))
    }
}

pub fn validate_forecast_params(params: &serde_json::Value) -> Result<(), ValidationError> {
    if let Some(obj) = params.as_object() {
        // Validate specific parameters based on method
        if let Some(alpha) = obj.get("alpha") {
            if let Some(alpha_val) = alpha.as_f64() {
                if alpha_val < 0.0 || alpha_val > 1.0 {
                    return Err(ValidationError::new(
                        "Alpha parameter must be between 0 and 1",
                    ));
                }
            }
        }

        if let Some(beta) = obj.get("beta") {
            if let Some(beta_val) = beta.as_f64() {
                if beta_val < 0.0 || beta_val > 1.0 {
                    return Err(ValidationError::new(
                        "Beta parameter must be between 0 and 1",
                    ));
                }
            }
        }

        if let Some(gamma) = obj.get("gamma") {
            if let Some(gamma_val) = gamma.as_f64() {
                if gamma_val < 0.0 || gamma_val > 1.0 {
                    return Err(ValidationError::new(
                        "Gamma parameter must be between 0 and 1",
                    ));
                }
            }
        }

        if let Some(seasonal_periods) = obj.get("seasonal_periods") {
            if let Some(periods_val) = seasonal_periods.as_i64() {
                if periods_val < 1 || periods_val > 365 {
                    return Err(ValidationError::new(
                        "Seasonal periods must be between 1 and 365",
                    ));
                }
            }
        }
    }

    Ok(())
}

// Legacy validation functions for backward compatibility
pub fn validate_sort_field(field: &str) -> Result<(), ValidationError> {
    validate_forecast_sort_by(field)
}
