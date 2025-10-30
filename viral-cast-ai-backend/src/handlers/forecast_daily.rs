use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use rust_decimal::prelude::ToPrimitive;
use serde_json::{json, Value};
use sqlx::types::Decimal;
use sqlx::Row;
use uuid::Uuid;
use validator::Validate;

use crate::{dto::forecast_daily::*, models::forecast_daily::*, AppState};

// Create forecast daily record
pub async fn create_forecast_daily(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateForecastRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(errors) = body.validate() {
        let error_map = errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let messages: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .unwrap_or(&std::borrow::Cow::Borrowed("Validation error"))
                            .to_string()
                    })
                    .collect();
                (field.to_string(), messages)
            })
            .collect::<std::collections::HashMap<String, Vec<String>>>();

        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Validation failed",
                "errors": error_map
            })),
        ));
    }

    // Validate forecast method
    if validate_forecast_method(&body.method).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Invalid forecast method"
            })),
        ));
    }

    // Convert NaiveDate to epoch milliseconds
    let date_ts_ms = body
        .date_ts
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp_millis();

    // Check if record already exists for this product, date, and method
    let existing_record = sqlx::query!(
        "SELECT uuid FROM forecast_daily WHERE product_uuid = $1 AND date_ts = $2 AND method = $3 AND deleted_at = 0",
        body.product_uuid,
        date_ts_ms,
        body.method
    )
    .fetch_optional(&data.db)
    .await;

    match existing_record {
        Ok(Some(_)) => {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({
                    "status": "error",
                    "message": "Forecast record already exists for this product, date, and method"
                })),
            ));
        }
        Ok(None) => {}
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Database error: {}", e)
                })),
            ));
        }
    }

    // Create new forecast daily record
    let forecast_daily_id = Uuid::new_v4();

    // Convert f64 values to Decimal
    let forecast_qty_dec =
        rust_decimal::Decimal::from_f64_retain(body.forecast_qty).unwrap_or_default();
    let conf_low_dec = body
        .conf_low
        .map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or_default());
    let conf_high_dec = body
        .conf_high
        .map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or_default());
    let mae_dec = body
        .mae
        .map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or_default());
    let mape_dec = body
        .mape
        .map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or_default());

    let query_result = sqlx::query!(
        r#"
        INSERT INTO forecast_daily (uuid, product_uuid, date_ts, method, window_size, params, 
                                   forecast_qty, conf_low, conf_high, mae, mape)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING uuid, product_uuid, date_ts, method, window_size, params, 
                  forecast_qty, conf_low, conf_high, mae, mape, created_at, updated_at
        "#,
        forecast_daily_id,
        body.product_uuid,
        date_ts_ms,
        body.method,
        body.window_size,
        body.params,
        forecast_qty_dec,
        conf_low_dec,
        conf_high_dec,
        mae_dec,
        mape_dec
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(record) => {
            let forecast_daily = ForecastDaily {
                uuid: record.uuid,
                product_uuid: record.product_uuid,
                date_ts: record.date_ts,
                method: record.method,
                window_size: record.window_size,
                params: record.params,
                forecast_qty: record.forecast_qty,
                conf_low: record.conf_low,
                conf_high: record.conf_high,
                mae: record.mae,
                mape: record.mape,
                created_at: record.created_at.or(Some(0)),
                updated_at: record.updated_at.or(Some(0)),
                deleted_at: None,
            };

            let response = ForecastDailyResponse::from(forecast_daily);

            Ok(Json(json!({
                "status": "success",
                "message": "Forecast daily record created successfully",
                "data": response
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to create forecast daily record: {}", e)
            })),
        )),
    }
}

// Get forecast daily record by ID
pub async fn get_forecast_daily_by_id(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let query_result = sqlx::query(
        r#"
        SELECT fd.uuid, fd.product_uuid, fd.date_ts, fd.method, fd.window_size, fd.params,
               fd.forecast_qty, fd.conf_low, fd.conf_high, fd.mae, fd.mape,
               fd.created_at, fd.updated_at,
               p.name as "product_name: Option<String>", p.sku as product_sku
        FROM forecast_daily fd
        LEFT JOIN products p ON fd.product_uuid = p.uuid
        WHERE fd.uuid = $1 AND fd.deleted_at = 0
        "#,
    )
    .bind(id)
    .fetch_optional(&data.db)
    .await;

    match query_result {
        Ok(Some(row)) => {
            let forecast_daily = ForecastDailyWithProduct {
                uuid: row.get("uuid"),
                product_uuid: row.get("product_uuid"),
                product_name: row.get("product_name"),
                product_sku: row.get("product_sku"),
                date_ts: row.get("date_ts"),
                method: row.get("method"),
                window_size: row.get("window_size"),
                params: row.get("params"),
                forecast_qty: row.get("forecast_qty"),
                conf_low: row.get("conf_low"),
                conf_high: row.get("conf_high"),
                mae: row.get("mae"),
                mape: row.get("mape"),
                accuracy_score: None, // Default value since not calculated here
                created_at: row.get::<Option<i64>, _>("created_at").or(Some(0)),
                updated_at: row.get::<Option<i64>, _>("updated_at").or(Some(0)),
            };

            let response = ForecastDailyWithProductResponse::from(forecast_daily);

            Ok(Json(json!({
                "status": "success",
                "data": response
            })))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Forecast daily record not found"
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Database error: {}", e)
            })),
        )),
    }
}

// Get forecast daily records with filtering and pagination
pub async fn get_forecast_daily(
    State(data): State<Arc<AppState>>,
    Query(params): Query<ForecastListRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(errors) = params.validate() {
        let error_map = errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let messages: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .unwrap_or(&std::borrow::Cow::Borrowed("Validation error"))
                            .to_string()
                    })
                    .collect();
                (field.to_string(), messages)
            })
            .collect::<std::collections::HashMap<String, Vec<String>>>();

        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Validation failed",
                "errors": error_map
            })),
        ));
    }

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);
    let offset = (page - 1) * page_size;
    let sort_by = params.sort_by.unwrap_or_else(|| "date_ts".to_string());
    let sort_order = params.sort_order.unwrap_or_else(|| "desc".to_string());

    // Extract filter parameters for reuse
    let product_uuid_filter = params.product_uuid;
    let method_filter = params.method.as_ref();
    let start_date_filter = params.start_date;
    let end_date_filter = params.end_date;

    // Convert dates to timestamps if provided
    let start_ts =
        start_date_filter.map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis());
    let end_ts = end_date_filter.map(|d| {
        d.and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc()
            .timestamp_millis()
    });

    // Validate sort field
    let sort_field = if validate_forecast_sort_by(&sort_by).is_ok() {
        sort_by
    } else {
        "date_ts".to_string()
    };

    let sort_direction = if sort_order.to_lowercase() == "asc" {
        "ASC"
    } else {
        "DESC"
    };

    // Build count query with QueryBuilder
    let mut qb_count = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        "SELECT COUNT(*) FROM forecast_daily fd WHERE fd.deleted_at = 0",
    );

    if let Some(product_uuid) = product_uuid_filter {
        qb_count
            .push(" AND fd.product_uuid = ")
            .push_bind(product_uuid);
    }

    if let Some(method) = method_filter {
        qb_count.push(" AND fd.method = ").push_bind(method);
    }

    if let Some(start_ts) = start_ts {
        qb_count.push(" AND fd.date_ts >= ").push_bind(start_ts);
    }

    if let Some(end_ts) = end_ts {
        qb_count.push(" AND fd.date_ts <= ").push_bind(end_ts);
    }

    let total_result = qb_count
        .build_query_scalar::<i64>()
        .fetch_one(&data.db)
        .await;

    let total = match total_result {
        Ok(count) => count,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to get total count: {}", e)
                })),
            ));
        }
    };

    // Build select query with QueryBuilder
    let mut qb_select = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"
        SELECT fd.uuid, fd.product_uuid, fd.date_ts, fd.method, fd.window_size, fd.params,
               fd.forecast_qty, fd.conf_low, fd.conf_high, fd.mae, fd.mape,
               fd.created_at, fd.updated_at,
               p.name as "product_name: Option<String>", 
               p.sku as "product_sku: Option<String>"
        FROM forecast_daily fd
        LEFT JOIN products p ON fd.product_uuid = p.uuid
        WHERE fd.deleted_at = 0
        "#,
    );

    if let Some(product_uuid) = product_uuid_filter {
        qb_select
            .push(" AND fd.product_uuid = ")
            .push_bind(product_uuid);
    }

    if let Some(method) = method_filter {
        qb_select.push(" AND fd.method = ").push_bind(method);
    }

    if let Some(start_ts) = start_ts {
        qb_select.push(" AND fd.date_ts >= ").push_bind(start_ts);
    }

    if let Some(end_ts) = end_ts {
        qb_select.push(" AND fd.date_ts <= ").push_bind(end_ts);
    }

    qb_select
        .push(" ORDER BY fd.")
        .push(&sort_field)
        .push(" ")
        .push(&sort_direction);
    qb_select
        .push(" LIMIT ")
        .push_bind(page_size)
        .push(" OFFSET ")
        .push_bind(offset);

    let records_result = qb_select.build().fetch_all(&data.db).await;

    match records_result {
        Ok(rows) => {
            let forecast_daily: Vec<ForecastDailyWithProductResponse> = rows
                .iter()
                .map(|row| {
                    let model = ForecastDailyWithProduct {
                        uuid: row.get("uuid"),
                        product_uuid: row.get("product_uuid"),
                        product_name: row.get("product_name"),
                        product_sku: row.get("product_sku"),
                        date_ts: row.get("date_ts"),
                        method: row.get("method"),
                        window_size: row.get("window_size"),
                        params: row.get("params"),
                        forecast_qty: row.get("forecast_qty"),
                        conf_low: row.get("conf_low"),
                        conf_high: row.get("conf_high"),
                        mae: row.get("mae"),
                        mape: row.get("mape"),
                        accuracy_score: None,
                        created_at: row.try_get::<i64, _>("created_at").ok(),
                        updated_at: row.try_get::<i64, _>("updated_at").ok(),
                    };
                    ForecastDailyWithProductResponse::from(model)
                })
                .collect();

            let total_pages = (total + page_size - 1) / page_size;

            let response = ForecastListResponse {
                data: forecast_daily,
                total,
                page,
                page_size,
                total_pages,
            };

            Ok(Json(json!({
                "status": "success",
                "data": response
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to fetch forecast daily records: {}", e)
            })),
        )),
    }
}

// Update forecast daily record
pub async fn update_forecast_daily(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateForecastRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(errors) = body.validate() {
        let error_map = errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let messages: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .unwrap_or(&std::borrow::Cow::Borrowed("Validation error"))
                            .to_string()
                    })
                    .collect();
                (field.to_string(), messages)
            })
            .collect::<std::collections::HashMap<String, Vec<String>>>();

        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Validation failed",
                "errors": error_map
            })),
        ));
    }

    // Check if record exists
    let existing_record = sqlx::query!(
        "SELECT uuid FROM forecast_daily WHERE uuid = $1 AND deleted_at = 0",
        id
    )
    .fetch_optional(&data.db)
    .await;

    match existing_record {
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "status": "error",
                    "message": "Forecast daily record not found"
                })),
            ));
        }
        Ok(Some(_)) => {}
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Database error: {}", e)
                })),
            ));
        }
    }

    // Build update query with QueryBuilder
    let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE forecast_daily SET ");
    let mut sep = qb.separated(", ");
    let mut has_updates = false;

    if let Some(forecast_qty) = body.forecast_qty {
        let forecast_qty_dec =
            rust_decimal::Decimal::from_f64_retain(forecast_qty).unwrap_or_default();
        sep.push("forecast_qty = ").push_bind(forecast_qty_dec);
        has_updates = true;
    }

    if let Some(conf_low) = body.conf_low {
        let conf_low_dec = rust_decimal::Decimal::from_f64_retain(conf_low).unwrap_or_default();
        sep.push("conf_low = ").push_bind(conf_low_dec);
        has_updates = true;
    }

    if let Some(conf_high) = body.conf_high {
        let conf_high_dec = rust_decimal::Decimal::from_f64_retain(conf_high).unwrap_or_default();
        sep.push("conf_high = ").push_bind(conf_high_dec);
        has_updates = true;
    }

    if let Some(mae) = body.mae {
        let mae_dec = rust_decimal::Decimal::from_f64_retain(mae).unwrap_or_default();
        sep.push("mae = ").push_bind(mae_dec);
        has_updates = true;
    }

    if let Some(mape) = body.mape {
        let mape_dec = rust_decimal::Decimal::from_f64_retain(mape).unwrap_or_default();
        sep.push("mape = ").push_bind(mape_dec);
        has_updates = true;
    }

    if !has_updates {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "No fields to update"
            })),
        ));
    }

    // Always set updated_at
    sep.push("updated_at = EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)::bigint");

    qb.push(" WHERE uuid = ")
        .push_bind(id)
        .push(" AND deleted_at = 0");
    qb.push(" RETURNING uuid, product_uuid, date_ts, method, window_size, params, forecast_qty, conf_low, conf_high, mae, mape, created_at, updated_at");

    let query_result = qb.build().fetch_one(&data.db).await;

    match query_result {
        Ok(record) => {
            let forecast_daily = ForecastDaily {
                uuid: record.get("uuid"),
                product_uuid: record.get("product_uuid"),
                date_ts: record.get("date_ts"),
                method: record.get("method"),
                window_size: record.get("window_size"),
                params: record.get("params"),
                forecast_qty: record.get("forecast_qty"),
                conf_low: record.get("conf_low"),
                conf_high: record.get("conf_high"),
                mae: record.get("mae"),
                mape: record.get("mape"),
                created_at: record.get::<Option<i64>, _>("created_at").or(Some(0)),
                updated_at: record.get::<Option<i64>, _>("updated_at").or(Some(0)),
                deleted_at: None,
            };

            let response = ForecastDailyResponse::from(forecast_daily);

            Ok(Json(json!({
                "status": "success",
                "message": "Forecast daily record updated successfully",
                "data": response
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to update forecast daily record: {}", e)
            })),
        )),
    }
}

// Delete forecast daily record
pub async fn delete_forecast_daily(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let query_result = sqlx::query!(
        "UPDATE forecast_daily SET deleted_at = EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)::bigint WHERE uuid = $1 AND deleted_at = 0",
        id
    )
    .execute(&data.db)
    .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "status": "error",
                        "message": "Forecast daily record not found"
                    })),
                ))
            } else {
                Ok(Json(json!({
                    "status": "success",
                    "message": "Forecast daily record deleted successfully"
                })))
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to delete forecast daily record: {}", e)
            })),
        )),
    }
}

// Generate forecast for a product
pub async fn generate_forecast(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GenerateForecastRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(errors) = body.validate() {
        let error_map = errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let messages: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .unwrap_or(&std::borrow::Cow::Borrowed("Validation error"))
                            .to_string()
                    })
                    .collect();
                (field.to_string(), messages)
            })
            .collect::<std::collections::HashMap<String, Vec<String>>>();

        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Validation failed",
                "errors": error_map
            })),
        ));
    }

    // Validate forecast methods
    for method in &body.methods {
        if validate_forecast_method(method).is_err() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": format!("Invalid forecast method: {}", method)
                })),
            ));
        }
    }

    // Convert NaiveDate to timestamp for database queries
    let start_timestamp = body
        .start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp()
        * 1000;
    let end_timestamp = body
        .end_date
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc()
        .timestamp()
        * 1000;
    let window_size = body.window_size.unwrap_or(30) as i64;

    // Get historical sales data for each product
    let mut generated_forecasts = 0;
    let mut products_processed = 0;

    for product_uuid in &body.product_uuids {
        let historical_data = sqlx::query!(
            r#"
            SELECT 
                (EXTRACT(EPOCH FROM to_timestamp(oi.created_at / 1000)::date)::bigint * 1000) AS "date_ts!: i64",
                COALESCE(SUM(oi.qty), 0)::numeric AS "qty!: Decimal"
            FROM order_items oi
            JOIN orders o ON oi.order_uuid = o.uuid
            WHERE oi.product_uuid = $1 
              AND oi.created_at BETWEEN $2 AND $3
              AND oi.deleted_at = 0
              AND o.deleted_at = 0
              AND o.status = 'PAID'
            GROUP BY to_timestamp(oi.created_at / 1000)::date
            ORDER BY 1 DESC
            LIMIT $4
            "#,
            product_uuid,
            start_timestamp,
            end_timestamp,
            window_size
        )
        .fetch_all(&data.db)
        .await;

        match historical_data {
            Ok(rows) => {
                if rows.len() < window_size as usize {
                    continue; // Skip this product if insufficient data
                }

                products_processed += 1;

                // Generate forecasts for each method
                for method in &body.methods {
                    // Simple moving average forecast (placeholder implementation)
                    let avg_qty: f64 = rows
                        .iter()
                        .map(|r| r.qty.to_f64().unwrap_or(0.0))
                        .sum::<f64>()
                        / rows.len() as f64;
                    let std_dev = {
                        let variance = rows
                            .iter()
                            .map(|r| (r.qty.to_f64().unwrap_or(0.0) - avg_qty).powi(2))
                            .sum::<f64>()
                            / rows.len() as f64;
                        variance.sqrt()
                    };

                    let forecast_qty = avg_qty;
                    let conf_low = forecast_qty - (1.96 * std_dev);
                    let conf_high = forecast_qty + (1.96 * std_dev);

                    // Calculate MAE and MAPE (simplified)
                    let mae = std_dev * 0.8;
                    let mape = if avg_qty > 0.0 {
                        (mae / avg_qty) * 100.0
                    } else {
                        0.0
                    };

                    // Create forecast record for next day
                    let forecast_id = Uuid::new_v4();
                    let forecast_date = body.end_date.succ_opt().unwrap_or(body.end_date);
                    let forecast_timestamp = forecast_date
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_utc()
                        .timestamp()
                        * 1000;

                    let params_json = json!({
                        "window_size": window_size,
                        "confidence_level": body.confidence_level.unwrap_or(0.95)
                    });

                    // Convert f64 values to Decimal for database
                    let forecast_qty_dec =
                        rust_decimal::Decimal::from_f64_retain(forecast_qty).unwrap_or_default();
                    let conf_low_dec =
                        rust_decimal::Decimal::from_f64_retain(conf_low).unwrap_or_default();
                    let conf_high_dec =
                        rust_decimal::Decimal::from_f64_retain(conf_high).unwrap_or_default();
                    let mae_dec = rust_decimal::Decimal::from_f64_retain(mae).unwrap_or_default();
                    let mape_dec = rust_decimal::Decimal::from_f64_retain(mape).unwrap_or_default();

                    let query_result = sqlx::query!(
                        r#"
                        INSERT INTO forecast_daily (uuid, product_uuid, date_ts, method, window_size, params, 
                                                   forecast_qty, conf_low, conf_high, mae, mape, deleted_at)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                        "#,
                        forecast_id,
                        product_uuid,
                        forecast_timestamp,
                        method,
                        window_size as i32,
                        params_json,
                        forecast_qty_dec,
                        conf_low_dec,
                        conf_high_dec,
                        mae_dec,
                        mape_dec,
                        0i64 // deleted_at = 0 (not deleted)
                    )
                    .execute(&data.db)
                    .await;

                    if query_result.is_ok() {
                        generated_forecasts += 1;
                    }
                }
            }
            Err(_) => continue, // Skip this product on error
        }
    }

    let response = GenerateForecastResponse {
        generated_forecasts,
        products_processed,
        methods_used: body.methods.clone(),
        date_range: DateRangeResponse {
            start_date: body.start_date,
            end_date: body.end_date,
            total_days: (body.end_date - body.start_date).num_days(),
        },
        summary: ForecastGenerationSummaryResponse {
            avg_forecast_qty: 0.0,        // Calculate if needed
            total_forecast_qty: 0.0,      // Calculate if needed
            avg_confidence_interval: 0.0, // Calculate if needed
            methods_performance: vec![],  // Calculate if needed
        },
    };

    Ok(Json(json!({
        "status": "success",
        "message": "Forecasts generated successfully",
        "data": response
    })))
}

// Get forecast accuracy analysis
pub async fn get_forecast_accuracy(
    State(data): State<Arc<AppState>>,
    Query(params): Query<ForecastAccuracyRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(errors) = params.validate() {
        let error_map = errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let messages: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .unwrap_or(&std::borrow::Cow::Borrowed("Validation error"))
                            .to_string()
                    })
                    .collect();
                (field.to_string(), messages)
            })
            .collect::<std::collections::HashMap<String, Vec<String>>>();

        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Validation failed",
                "errors": error_map
            })),
        ));
    }

    // Convert NaiveDate to timestamp for database queries
    let start_timestamp = params
        .start_date
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() * 1000);
    let end_timestamp = params
        .end_date
        .map(|d| d.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp() * 1000);

    // Get forecast accuracy data
    let query_result = sqlx::query!(
        r#"
        SELECT 
            fd.method,
            AVG(fd.mae) as avg_mae,
            AVG(fd.mape) as avg_mape,
            COUNT(*) as forecast_count,
            (100.0 - AVG(fd.mape)) as accuracy_score
        FROM forecast_daily fd
        WHERE ($1::uuid IS NULL OR fd.product_uuid = $1)
        AND ($2::text IS NULL OR fd.method = $2)
        AND ($3::bigint IS NULL OR fd.date_ts >= $3)
        AND ($4::bigint IS NULL OR fd.date_ts <= $4)
        AND fd.deleted_at = 0
        GROUP BY fd.method
        ORDER BY accuracy_score DESC
        "#,
        params.product_uuid,
        params.method,
        start_timestamp,
        end_timestamp
    )
    .fetch_all(&data.db)
    .await;

    match query_result {
        Ok(rows) => {
            let first_row = rows.first();
            let response = ForecastAccuracyResponse {
                method: first_row.map(|r| r.method.clone()).unwrap_or_default(),
                avg_mae: first_row
                    .and_then(|r| r.avg_mae)
                    .map(|d| d.to_f64().unwrap_or(0.0))
                    .unwrap_or(0.0),
                avg_mape: first_row
                    .and_then(|r| r.avg_mape)
                    .map(|d| d.to_f64().unwrap_or(0.0))
                    .unwrap_or(0.0),
                accuracy_score: first_row
                    .and_then(|r| r.accuracy_score)
                    .map(|d| d.to_f64().unwrap_or(0.0))
                    .unwrap_or(0.0),
                forecast_count: first_row
                    .map(|r| r.forecast_count.unwrap_or(0) as i64)
                    .unwrap_or(0i64),
                best_performing_products: vec![], // Implement if needed
                worst_performing_products: vec![], // Implement if needed
            };

            Ok(Json(json!({
                "status": "success",
                "data": response
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to get forecast accuracy: {}", e)
            })),
        )),
    }
}

// Get forecast trend
pub async fn get_forecast_trend(
    State(data): State<Arc<AppState>>,
    Query(params): Query<ForecastTrendRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(errors) = params.validate() {
        let error_map = errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let messages: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .unwrap_or(&std::borrow::Cow::Borrowed("Validation error"))
                            .to_string()
                    })
                    .collect();
                (field.to_string(), messages)
            })
            .collect::<std::collections::HashMap<String, Vec<String>>>();

        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Validation failed",
                "errors": error_map
            })),
        ));
    }

    // Convert NaiveDate to timestamp for database queries
    let start_timestamp = params
        .start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp()
        * 1000;
    let end_timestamp = params
        .end_date
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc()
        .timestamp()
        * 1000;

    let query_result = sqlx::query!(
        r#"
        SELECT 
            fd.date_ts,
            fd.forecast_qty,
            fd.conf_low,
            fd.conf_high,
            fd.method,
            p.name as "product_name: Option<String>"
        FROM forecast_daily fd
        LEFT JOIN products p ON fd.product_uuid = p.uuid
        WHERE fd.product_uuid = $1 
        AND fd.date_ts BETWEEN $2 AND $3
        AND ($4::text IS NULL OR fd.method = $4)
        AND fd.deleted_at = 0
        ORDER BY fd.date_ts
        "#,
        params.product_uuid,
        start_timestamp,
        end_timestamp,
        params.method
    )
    .fetch_all(&data.db)
    .await;

    match query_result {
        Ok(rows) => {
            let trend_data: Vec<ForecastPointResponse> = rows
                .iter()
                .map(|row| ForecastPointResponse {
                    date_ts: chrono::DateTime::from_timestamp(row.date_ts / 1000, 0)
                        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap())
                        .date_naive(),
                    forecast_qty: row.forecast_qty.to_f64().unwrap_or(0.0),
                    conf_low: row.conf_low.map(|v| v.to_f64().unwrap_or(0.0)),
                    conf_high: row.conf_high.map(|v| v.to_f64().unwrap_or(0.0)),
                    actual_qty: None, // Would need to join with sales_daily
                    accuracy: None,   // Would need to calculate
                })
                .collect();

            let product_name = match rows.first() {
                Some(row) => match &row.product_name {
                    Some(name) => name.clone(),
                    None => String::new(),
                },
                None => String::new(),
            };

            let response = ForecastTrendResponse {
                product_uuid: params.product_uuid,
                product_name,
                method: params.method.unwrap_or_default(),
                trend_data,
                trend_direction: "stable".to_string(), // Calculate if needed
                confidence_trend: "stable".to_string(), // Calculate if needed
                avg_accuracy: 0.0,                     // Calculate if needed
            };

            Ok(Json(json!({
                "status": "success",
                "data": response
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to get forecast trend: {}", e)
            })),
        )),
    }
}
