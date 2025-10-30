use axum::http::HeaderMap;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use reqwest::Client;
use rust_decimal::prelude::ToPrimitive;
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::{
        api::{ApiResponse, ErrorResponse},
        payments::*,
    },
    AppState,
};

use crate::services::xendit as xnd;

pub async fn create_payment(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<CreatePaymentRequest>,
) -> Result<Json<ApiResponse<PaymentResponse>>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Validation error: {:?}", errors),
            }),
        ));
    }

    // Check if order exists (runtime query)
    let order_exists = sqlx::query("SELECT 1 FROM orders WHERE uuid = $1 AND deleted_at = 0")
        .bind(payload.order_uuid)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: format!("Database error: {}", e),
                }),
            )
        })?;

    if order_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: "Order not found".to_string(),
            }),
        ));
    }

    let payment_uuid = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp_millis();
    let paid_at = payload.paid_at.unwrap_or(now);

    sqlx::query(
        r#"
        INSERT INTO payments (uuid, order_uuid, method, amount, paid_at, external_ref, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(payment_uuid)
    .bind(payload.order_uuid)
    .bind(payload.method.to_uppercase())
    .bind(payload.amount)
    .bind(paid_at)
    .bind(payload.external_ref)
    .bind(now)
    .bind(now)
    .execute(&data.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Failed to create payment: {}", e),
            }),
        )
    })?;

    let payment = get_payment_by_id_internal(&data.db, payment_uuid).await?;

    Ok(Json(ApiResponse {
        code: 201,
        status: "success".to_string(),
        message: "Payment created successfully".to_string(),
        data: payment,
        errors: serde_json::json!(null),
    }))
}

pub async fn get_payment_by_id(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<PaymentResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let payment = get_payment_by_id_internal(&data.db, id).await?;

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Payment retrieved successfully".to_string(),
        data: payment,
        errors: serde_json::json!(null),
    }))
}

async fn get_payment_by_id_internal(
    pool: &PgPool,
    id: Uuid,
) -> Result<PaymentResponse, (StatusCode, Json<ErrorResponse>)> {
    let row_opt = sqlx::query(
        r#"
        SELECT 
            p.*,
            o.order_no
        FROM payments p
        LEFT JOIN orders o ON p.order_uuid = o.uuid
        WHERE p.uuid = $1 AND p.deleted_at = 0
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )
    })?;

    let row = match row_opt {
        Some(r) => r,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: "Payment not found".to_string(),
                }),
            ))
        }
    };

    Ok(PaymentResponse {
        uuid: row.get("uuid"),
        order_uuid: row.get("order_uuid"),
        order_no: row.get::<Option<String>, _>("order_no"),
        method: row.get("method"),
        amount: row.get("amount"),
        paid_at: row.get("paid_at"),
        external_ref: row.get("external_ref"),
        created_at: row.get::<Option<i64>, _>("created_at").or(Some(0)),
        updated_at: row.get::<Option<i64>, _>("updated_at").or(Some(0)),
    })
}

pub async fn get_payments(
    State(data): State<Arc<AppState>>,
    Query(params): Query<PaymentListRequest>,
) -> Result<Json<ApiResponse<PaymentListResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10).min(100);
    let offset = (page - 1) * limit;

    let mut query = "SELECT p.*, o.order_no FROM payments p LEFT JOIN orders o ON p.order_uuid = o.uuid WHERE p.deleted_at = 0".to_string();
    let mut conditions = Vec::new();
    let mut bind_count = 0;

    if params.order_uuid.is_some() {
        bind_count += 1;
        conditions.push(format!("p.order_uuid = ${}", bind_count));
    }

    if params.method.is_some() {
        bind_count += 1;
        conditions.push(format!("p.method = ${}", bind_count));
    }

    if params.date_from.is_some() {
        bind_count += 1;
        conditions.push(format!("p.created_at >= ${}", bind_count));
    }

    if params.date_to.is_some() {
        bind_count += 1;
        conditions.push(format!("p.created_at <= ${}", bind_count));
    }

    if params.search.is_some() {
        bind_count += 1;
        conditions.push(format!(
            "(o.order_no ILIKE ${} OR p.external_ref ILIKE ${})",
            bind_count, bind_count
        ));
    }

    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY p.created_at DESC");

    // Count total
    let count_query = format!(
        "SELECT COUNT(*) as total FROM payments p LEFT JOIN orders o ON p.order_uuid = o.uuid WHERE p.deleted_at = 0{}",
        if conditions.is_empty() {
            "".to_string()
        } else {
            format!(" AND {}", conditions.join(" AND "))
        }
    );

    let mut count_query_builder = sqlx::query(&count_query);
    let query_string = format!("{} LIMIT {} OFFSET {}", query, limit, offset);
    let mut query_builder = sqlx::query(&query_string);

    // Bind parameters for both queries
    if let Some(order_uuid) = params.order_uuid {
        count_query_builder = count_query_builder.bind(order_uuid);
        query_builder = query_builder.bind(order_uuid);
    }

    if let Some(method) = &params.method {
        let method_upper = method.to_uppercase();
        count_query_builder = count_query_builder.bind(method_upper.clone());
        query_builder = query_builder.bind(method_upper);
    }

    if let Some(date_from) = params.date_from {
        count_query_builder = count_query_builder.bind(date_from);
        query_builder = query_builder.bind(date_from);
    }

    if let Some(date_to) = params.date_to {
        count_query_builder = count_query_builder.bind(date_to);
        query_builder = query_builder.bind(date_to);
    }

    if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        count_query_builder = count_query_builder.bind(search_pattern.clone());
        query_builder = query_builder.bind(search_pattern.clone());
    }

    let total: i64 = count_query_builder
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: format!("Database error: {}", e),
                }),
            )
        })?
        .get("total");

    let rows = query_builder.fetch_all(&data.db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )
    })?;

    let payments: Vec<PaymentSummaryResponse> = rows
        .into_iter()
        .map(|row| PaymentSummaryResponse {
            uuid: row.get("uuid"),
            order_uuid: row.get("order_uuid"),
            order_no: row.get("order_no"),
            method: row.get("method"),
            amount: row.get("amount"),
            paid_at: row.get("paid_at"),
            external_ref: row.get("external_ref"),
            created_at: row.get::<Option<i64>, _>("created_at").or(Some(0)),
        })
        .collect();

    let total_pages = (total as f64 / limit as f64).ceil() as usize;

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Payments retrieved successfully".to_string(),
        data: PaymentListResponse {
            payments,
            total,
            page,
            limit,
            total_pages,
        },
        errors: serde_json::json!(null),
    }))
}

pub async fn update_payment(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePaymentRequest>,
) -> Result<Json<ApiResponse<PaymentResponse>>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Validation error: {:?}", errors),
            }),
        ));
    }

    let now = chrono::Utc::now().timestamp_millis();

    let mut set_clauses = Vec::new();
    let mut bind_count = 0;

    if payload.method.is_some() {
        bind_count += 1;
        set_clauses.push(format!("method = ${}", bind_count));
    }

    if payload.amount.is_some() {
        bind_count += 1;
        set_clauses.push(format!("amount = ${}", bind_count));
    }

    if payload.paid_at.is_some() {
        bind_count += 1;
        set_clauses.push(format!("paid_at = ${}", bind_count));
    }

    if payload.external_ref.is_some() {
        bind_count += 1;
        set_clauses.push(format!("external_ref = ${}", bind_count));
    }

    if set_clauses.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: "No fields to update".to_string(),
            }),
        ));
    }

    bind_count += 1;
    set_clauses.push(format!("updated_at = ${}", bind_count));

    bind_count += 1;
    let query = format!(
        "UPDATE payments SET {} WHERE uuid = ${} AND deleted_at = 0",
        set_clauses.join(", "),
        bind_count
    );

    let mut query_builder = sqlx::query(&query);

    if let Some(method) = &payload.method {
        query_builder = query_builder.bind(method.to_uppercase());
    }

    if let Some(amount) = payload.amount {
        query_builder = query_builder.bind(amount);
    }

    if let Some(paid_at) = payload.paid_at {
        query_builder = query_builder.bind(paid_at);
    }

    if let Some(external_ref) = &payload.external_ref {
        query_builder = query_builder.bind(external_ref);
    }

    query_builder = query_builder.bind(now).bind(id);

    let result = query_builder.execute(&data.db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: "Payment not found".to_string(),
            }),
        ));
    }

    let payment = get_payment_by_id_internal(&data.db, id).await?;

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Payment updated successfully".to_string(),
        data: payment,
        errors: serde_json::json!(null),
    }))
}

pub async fn delete_payment(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ErrorResponse>)> {
    let now = chrono::Utc::now().timestamp_millis();

    let result =
        sqlx::query("UPDATE payments SET deleted_at = $1 WHERE uuid = $2 AND deleted_at = 0")
            .bind(now)
            .bind(id)
            .execute(&data.db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        status: "error".to_string(),
                        message: format!("Database error: {}", e),
                    }),
                )
            })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: "Payment not found".to_string(),
            }),
        ));
    }

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Payment deleted successfully".to_string(),
        data: serde_json::json!({"deleted": true}),
        errors: serde_json::json!(null),
    }))
}

pub async fn get_payment_stats(
    State(data): State<Arc<AppState>>,
    Query(params): Query<PaymentStatsRequest>,
) -> Result<Json<ApiResponse<PaymentStatsResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let mut conditions = vec!["p.deleted_at = 0".to_string()];
    let mut bind_count = 0;

    if params.date_from.is_some() {
        bind_count += 1;
        conditions.push(format!("p.created_at >= ${}", bind_count));
    }

    if params.date_to.is_some() {
        bind_count += 1;
        conditions.push(format!("p.created_at <= ${}", bind_count));
    }

    if params.method.is_some() {
        bind_count += 1;
        conditions.push(format!("p.method = ${}", bind_count));
    }

    if params.order_uuid.is_some() {
        bind_count += 1;
        conditions.push(format!("p.order_uuid = ${}", bind_count));
    }

    let where_clause = conditions.join(" AND ");

    // Get overall stats
    let stats_query = format!(
        r#"
        SELECT 
            COUNT(*) as total_payments,
            COALESCE(SUM(amount), 0) as total_amount,
            COALESCE(AVG(amount), 0) as avg_payment_amount
        FROM payments p
        WHERE {}
        "#,
        where_clause
    );

    let mut stats_query_builder = sqlx::query(&stats_query);

    if let Some(date_from) = params.date_from {
        stats_query_builder = stats_query_builder.bind(date_from);
    }

    if let Some(date_to) = params.date_to {
        stats_query_builder = stats_query_builder.bind(date_to);
    }

    if let Some(method) = &params.method {
        stats_query_builder = stats_query_builder.bind(method.to_uppercase());
    }

    if let Some(order_uuid) = params.order_uuid {
        stats_query_builder = stats_query_builder.bind(order_uuid);
    }

    let stats_row = stats_query_builder.fetch_one(&data.db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )
    })?;

    let total_payments: i64 = stats_row.get("total_payments");
    let total_amount: rust_decimal::Decimal = stats_row.get("total_amount");
    let avg_payment_amount: rust_decimal::Decimal = stats_row.get("avg_payment_amount");

    // Get payments by method
    let method_query = format!(
        r#"
        SELECT 
            method,
            COUNT(*) as count,
            COALESCE(SUM(amount), 0) as total_amount
        FROM payments p
        WHERE {}
        GROUP BY method
        ORDER BY total_amount DESC
        "#,
        where_clause
    );

    let mut method_query_builder = sqlx::query(&method_query);

    if let Some(date_from) = params.date_from {
        method_query_builder = method_query_builder.bind(date_from);
    }

    if let Some(date_to) = params.date_to {
        method_query_builder = method_query_builder.bind(date_to);
    }

    if let Some(method) = &params.method {
        method_query_builder = method_query_builder.bind(method.to_uppercase());
    }

    if let Some(order_uuid) = params.order_uuid {
        method_query_builder = method_query_builder.bind(order_uuid);
    }

    let method_rows = method_query_builder
        .fetch_all(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: format!("Database error: {}", e),
                }),
            )
        })?;

    let payments_by_method: Vec<PaymentMethodCount> = method_rows
        .into_iter()
        .map(|row| {
            let method_total: rust_decimal::Decimal = row.get("total_amount");
            let percentage = if total_amount > rust_decimal::Decimal::ZERO {
                (method_total / total_amount * rust_decimal::Decimal::from(100))
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            PaymentMethodCount {
                method: row.get("method"),
                count: row.get("count"),
                total_amount: method_total,
                percentage,
            }
        })
        .collect();

    // Get daily stats (last 30 days or within date range)
    let daily_query = format!(
        r#"
        SELECT 
            DATE(to_timestamp(created_at / 1000)) as date,
            COUNT(*) as payments_count,
            COALESCE(SUM(amount), 0) as total_amount,
            COALESCE(AVG(amount), 0) as avg_amount
        FROM payments p
        WHERE {}
        GROUP BY DATE(to_timestamp(created_at / 1000))
        ORDER BY date DESC
        LIMIT 30
        "#,
        where_clause
    );

    let mut daily_query_builder = sqlx::query(&daily_query);

    if let Some(date_from) = params.date_from {
        daily_query_builder = daily_query_builder.bind(date_from);
    }

    if let Some(date_to) = params.date_to {
        daily_query_builder = daily_query_builder.bind(date_to);
    }

    if let Some(method) = &params.method {
        daily_query_builder = daily_query_builder.bind(method.to_uppercase());
    }

    if let Some(order_uuid) = params.order_uuid {
        daily_query_builder = daily_query_builder.bind(order_uuid);
    }

    let daily_rows = daily_query_builder.fetch_all(&data.db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )
    })?;

    let daily_stats: Vec<DailyPaymentStats> = daily_rows
        .into_iter()
        .map(|row| DailyPaymentStats {
            date: row.get::<chrono::NaiveDate, _>("date").to_string(),
            payments_count: row.get("payments_count"),
            total_amount: row.get("total_amount"),
            avg_amount: row.get("avg_amount"),
        })
        .collect();

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Payment statistics retrieved successfully".to_string(),
        data: PaymentStatsResponse {
            total_payments,
            total_amount,
            avg_payment_amount,
            payments_by_method,
            daily_stats,
        },
        errors: serde_json::json!(null),
    }))
}

pub async fn get_payments_by_order(
    State(data): State<Arc<AppState>>,
    Path(order_uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<PaymentsByOrderResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Get order details (runtime query)
    let order_row_opt =
        sqlx::query("SELECT uuid, order_no, total FROM orders WHERE uuid = $1 AND deleted_at = 0")
            .bind(order_uuid)
            .fetch_optional(&data.db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        status: "error".to_string(),
                        message: format!("Database error: {}", e),
                    }),
                )
            })?;

    let order_row = match order_row_opt {
        Some(r) => r,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: "Order not found".to_string(),
                }),
            ))
        }
    };

    let order_no: String = order_row.get("order_no");
    let order_total: rust_decimal::Decimal = order_row.get("total");

    // Get payments for this order (runtime query)
    let payment_rows = sqlx::query(
        r#"
        SELECT 
            uuid, order_uuid, method, amount, paid_at, external_ref, created_at, updated_at
        FROM payments 
        WHERE order_uuid = $1 AND deleted_at = 0
        ORDER BY created_at DESC
        "#,
    )
    .bind(order_uuid)
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )
    })?;

    let payment_responses: Vec<PaymentResponse> = payment_rows
        .into_iter()
        .map(|row| PaymentResponse {
            uuid: row.get("uuid"),
            order_uuid: row.get("order_uuid"),
            order_no: Some(order_no.clone()),
            method: row.get("method"),
            amount: row.get("amount"),
            paid_at: row.get("paid_at"),
            external_ref: row.get("external_ref"),
            created_at: row.get::<Option<i64>, _>("created_at").or(Some(0)),
            updated_at: row.get::<Option<i64>, _>("updated_at").or(Some(0)),
        })
        .collect();

    let total_paid: rust_decimal::Decimal = payment_responses.iter().map(|p| p.amount).sum();

    let remaining_amount = order_total - total_paid;

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Payments for order retrieved successfully".to_string(),
        data: PaymentsByOrderResponse {
            order_uuid,
            order_no: order_no,
            order_total: order_total,
            total_paid,
            remaining_amount,
            payments: payment_responses,
        },
        errors: serde_json::json!(null),
    }))
}

// ===== QRIS (Xendit) Handlers =====

async fn create_qris_payment_internal(
    data: Arc<AppState>,
    Json(payload): Json<CreateQrisRequest>,
    use_sandbox: bool,
) -> Result<Json<ApiResponse<QrisCreateResponse>>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Validation error: {:?}", errors),
            }),
        ));
    }

    // secret key selection
    let secret_key = if use_sandbox {
        data.env.xendit_secret_key_sandbox.as_deref()
    } else {
        data.env.xendit_secret_key_live.as_deref()
    }
    .ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: "Xendit secret key is not configured".to_string(),
            }),
        )
    })?;

    let callback_url = data.env.xendit_qris_callback_url.as_deref();

    let mut tx = data.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Failed to begin transaction: {}", e),
            }),
        )
    })?;

    // Lock order row to prevent race conditions (runtime query)
    let order_row_opt = sqlx::query(
        "SELECT uuid, order_no, total FROM orders WHERE uuid = $1 AND deleted_at = 0 FOR UPDATE",
    )
    .bind(payload.order_uuid)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )
    })?;

    let order_row = match order_row_opt {
        Some(r) => r,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: "Order not found".to_string(),
                }),
            ))
        }
    };

    // Check existing QRIS payment and lock it if exists
    let existing = sqlx::query(
        r#"SELECT uuid, external_ref, paid_at FROM payments WHERE order_uuid = $1 AND method = 'QRIS' AND deleted_at = 0 ORDER BY created_at DESC LIMIT 1 FOR UPDATE"#
    )
    .bind(payload.order_uuid)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { status: "error".to_string(), message: format!("Database error: {}", e) }),
    ))?;

    let client = Client::new();

    // Either reuse existing QR code or create a new one
    let (payment_uuid, external_ref, xendit_info) = if let Some(row) = existing {
        // If already completed, just return status without creating new QR
        let existing_ext_ref: Option<String> = row.get("external_ref");
        let existing_uuid: Uuid = row.get("uuid");
        let qr = xnd::get_qr_code(
            &client,
            secret_key,
            existing_ext_ref.as_deref().unwrap_or(""),
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: format!("Xendit get QR error: {}", e),
                }),
            )
        })?;

        let info = XenditQrisInfo {
            id: qr.id.clone(),
            reference_id: qr.reference_id.clone(),
            status: qr.status.clone(),
            qr_string: qr.qr_string.clone(),
            qr_code_url: qr.qr_code_url.clone(),
            expires_at: qr.expires_at.clone(),
        };

        (existing_uuid, qr.id, info)
    } else {
        // Create new QR code
        let order_no: String = order_row.get("order_no");
        let reference_id = format!("ORDER-{}", order_no);
        let amount_i64 = payload.amount.to_i64().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: "Invalid amount for QRIS".to_string(),
                }),
            )
        })?;

        let qr = xnd::create_qr_code(&client, secret_key, &reference_id, amount_i64, callback_url)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        status: "error".to_string(),
                        message: format!("Xendit create QR error: {}", e),
                    }),
                )
            })?;

        let now = chrono::Utc::now().timestamp_millis();
        let new_payment_uuid = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO payments (uuid, order_uuid, method, amount, paid_at, external_ref, created_at, updated_at)
            VALUES ($1, $2, 'QRIS', $3, $4, $5, $6, $7)
            "#
        )
        .bind(new_payment_uuid)
        .bind(payload.order_uuid)
        .bind(payload.amount)
        .bind(Option::<i64>::None)
        .bind(Some(qr.id.clone()))
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { status: "error".to_string(), message: format!("Failed to insert payment: {}", e) }),
        ))?;

        let info = XenditQrisInfo {
            id: qr.id.clone(),
            reference_id: qr.reference_id.clone(),
            status: qr.status.clone(),
            qr_string: qr.qr_string.clone(),
            qr_code_url: qr.qr_code_url.clone(),
            expires_at: qr.expires_at.clone(),
        };

        (new_payment_uuid, qr.id, info)
    };

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Failed to commit transaction: {}", e),
            }),
        )
    })?;

    Ok(Json(ApiResponse {
        code: 201,
        status: "success".to_string(),
        message: "QRIS created".to_string(),
        data: QrisCreateResponse {
            payment_uuid,
            order_uuid: payload.order_uuid,
            method: "QRIS".to_string(),
            amount: payload.amount,
            external_ref,
            xendit: xendit_info,
        },
        errors: serde_json::json!(null),
    }))
}

pub async fn create_qris_payment_sandbox(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<CreateQrisRequest>,
) -> Result<Json<ApiResponse<QrisCreateResponse>>, (StatusCode, Json<ErrorResponse>)> {
    create_qris_payment_internal(data, Json(payload), true).await
}

pub async fn create_qris_payment_live(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<CreateQrisRequest>,
) -> Result<Json<ApiResponse<QrisCreateResponse>>, (StatusCode, Json<ErrorResponse>)> {
    create_qris_payment_internal(data, Json(payload), false).await
}

async fn get_qris_status_internal(
    data: Arc<AppState>,
    Path(external_ref): Path<String>,
    use_sandbox: bool,
) -> Result<Json<ApiResponse<QrisStatusResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let secret_key = if use_sandbox {
        data.env.xendit_secret_key_sandbox.as_deref()
    } else {
        data.env.xendit_secret_key_live.as_deref()
    }
    .ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: "Xendit secret key is not configured".to_string(),
            }),
        )
    })?;

    let client = Client::new();
    let qr = xnd::get_qr_code(&client, secret_key, &external_ref)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: format!("Xendit get QR error: {}", e),
                }),
            )
        })?;

    let mut tx = data.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Failed to begin transaction: {}", e),
            }),
        )
    })?;

    let payment_row = sqlx::query(
        r#"SELECT uuid, paid_at FROM payments WHERE external_ref = $1 AND method = 'QRIS' AND deleted_at = 0 FOR UPDATE"#
    )
    .bind(&external_ref)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { status: "error".to_string(), message: format!("Database error: {}", e) }),
    ))?;

    let mut payment_uuid_opt: Option<Uuid> = None;
    if let Some(pr) = payment_row {
        let pr_uuid: Uuid = pr.get("uuid");
        let pr_paid_at: Option<i64> = pr.get("paid_at");
        payment_uuid_opt = Some(pr_uuid);
        if qr.status.to_uppercase() == "COMPLETED" && pr_paid_at.is_none() {
            let now = chrono::Utc::now().timestamp_millis();
            sqlx::query("UPDATE payments SET paid_at = $1, updated_at = $2 WHERE uuid = $3")
                .bind(now)
                .bind(now)
                .bind(pr_uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            status: "error".to_string(),
                            message: format!("Database error: {}", e),
                        }),
                    )
                })?;
        }
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Failed to commit transaction: {}", e),
            }),
        )
    })?;

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "QRIS status".to_string(),
        data: {
            let status_str = qr.status.clone();
            QrisStatusResponse {
                payment_uuid: payment_uuid_opt,
                external_ref: external_ref,
                status: status_str.clone(),
                paid: status_str.eq_ignore_ascii_case("COMPLETED"),
            }
        },
        errors: serde_json::json!(null),
    }))
}

pub async fn get_qris_status_sandbox(
    State(data): State<Arc<AppState>>,
    Path(external_ref): Path<String>,
) -> Result<Json<ApiResponse<QrisStatusResponse>>, (StatusCode, Json<ErrorResponse>)> {
    get_qris_status_internal(data, Path(external_ref), true).await
}

pub async fn get_qris_status_live(
    State(data): State<Arc<AppState>>,
    Path(external_ref): Path<String>,
) -> Result<Json<ApiResponse<QrisStatusResponse>>, (StatusCode, Json<ErrorResponse>)> {
    get_qris_status_internal(data, Path(external_ref), false).await
}

pub async fn xendit_webhook_handler(
    State(data): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ErrorResponse>)> {
    let token_hdr = headers
        .get("x-callback-token")
        .and_then(|h| h.to_str().ok());
    let mut expected = vec![];
    if let Some(t) = &data.env.xendit_callback_token_sandbox {
        expected.push(t.clone());
    }
    if let Some(t) = &data.env.xendit_callback_token_live {
        expected.push(t.clone());
    }
    if !xnd::validate_callback_token(token_hdr, &expected) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: "Invalid callback token".to_string(),
            }),
        ));
    }

    let qr_id = body
        .pointer("/data/id")
        .and_then(|v| v.as_str())
        .or_else(|| body.pointer("/qr_code/id").and_then(|v| v.as_str()))
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: "Invalid webhook payload (missing qr id)".to_string(),
                }),
            )
        })?;

    let status = body
        .pointer("/data/status")
        .and_then(|v| v.as_str())
        .or_else(|| body.pointer("/qr_code/status").and_then(|v| v.as_str()))
        .unwrap_or("ACTIVE");

    let mut tx = data.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Failed to begin transaction: {}", e),
            }),
        )
    })?;

    let payment_row = sqlx::query(
        r#"SELECT uuid, paid_at FROM payments WHERE external_ref = $1 AND method = 'QRIS' AND deleted_at = 0 FOR UPDATE"#
    )
    .bind(qr_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { status: "error".to_string(), message: format!("Database error: {}", e) }),
    ))?;

    if let Some(pr) = payment_row {
        let pr_uuid: Uuid = pr.get("uuid");
        let pr_paid_at: Option<i64> = pr.get("paid_at");
        if status.to_uppercase() == "COMPLETED" && pr_paid_at.is_none() {
            let now = chrono::Utc::now().timestamp_millis();
            sqlx::query("UPDATE payments SET paid_at = $1, updated_at = $2 WHERE uuid = $3")
                .bind(now)
                .bind(now)
                .bind(pr_uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            status: "error".to_string(),
                            message: format!("Database error: {}", e),
                        }),
                    )
                })?;
        }
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Failed to commit transaction: {}", e),
            }),
        )
    })?;

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Webhook processed".to_string(),
        data: serde_json::json!({"ok": true}),
        errors: serde_json::json!(null),
    }))
}
