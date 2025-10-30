use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::models::orders::{Order, OrderItemWithProduct};
use crate::{
    dto::{
        api::{ApiResponse, ErrorResponse},
        orders::*,
    },
    AppState,
};

pub async fn create_order(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<ApiResponse<OrderResponse>>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Validation error: {:?}", errors),
            }),
        ));
    }

    let mut tx = data.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )
    })?;

    // Check if order_no already exists
    let existing_order =
        sqlx::query("SELECT uuid FROM orders WHERE order_no = $1 AND deleted_at = 0")
            .bind(&payload.order_no)
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

    if existing_order.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: "Order number already exists".to_string(),
            }),
        ));
    }

    let order_uuid = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp_millis();

    // Create order
    let discount = payload
        .discount
        .unwrap_or_else(|| rust_decimal::Decimal::ZERO);
    let tax = payload.tax.unwrap_or_else(|| rust_decimal::Decimal::ZERO);

    sqlx::query(
        r#"
        INSERT INTO orders (uuid, order_no, cashier_uuid, status, subtotal, discount, tax, total, net_profit, created_at, updated_at)
        VALUES ($1, $2, $3, 'DRAFT', $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(order_uuid)
    .bind(&payload.order_no)
    .bind(payload.cashier_uuid)
    .bind(payload.subtotal)
    .bind(discount)
    .bind(tax)
    .bind(payload.total)
    .bind(payload.net_profit)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Failed to create order: {}", e),
            }),
        )
    })?;

    // Create order items
    for item in &payload.items {
        let item_uuid = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO order_items (uuid, order_uuid, product_uuid, qty, unit_price, unit_cost, line_total, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(item_uuid)
        .bind(order_uuid)
        .bind(item.product_uuid)
        .bind(item.qty)
        .bind(item.unit_price)
        .bind(item.unit_cost)
        .bind(item.line_total)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: format!("Failed to create order item: {}", e),
                }),
            )
        })?;
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

    // Fetch the created order with items
    let order = get_order_by_id_internal(&data.db, order_uuid).await?;

    Ok(Json(ApiResponse {
        code: 201,
        status: "success".to_string(),
        message: "Order created successfully".to_string(),
        data: order,
        errors: serde_json::json!(null),
    }))
}

pub async fn get_order_by_id(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<OrderResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let order = get_order_by_id_internal(&data.db, id).await?;

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Order retrieved successfully".to_string(),
        data: order,
        errors: serde_json::json!(null),
    }))
}

async fn get_order_by_id_internal(
    pool: &PgPool,
    id: Uuid,
) -> Result<OrderResponse, (StatusCode, Json<ErrorResponse>)> {
    // Get order
    let order =
        sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE uuid = $1 AND deleted_at = 0")
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
            })?
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        status: "error".to_string(),
                        message: "Order not found".to_string(),
                    }),
                )
            })?;

    // Get order items with product details
    let items = sqlx::query_as::<_, OrderItemWithProduct>(
        r#"
        SELECT 
            oi.*,
            p.name as product_name,
            p.sku as product_sku,
            p.price as product_price
        FROM order_items oi
        LEFT JOIN products p ON oi.product_uuid = p.uuid
        WHERE oi.order_uuid = $1 AND oi.deleted_at = 0
        ORDER BY oi.created_at
        "#,
    )
    .bind(id)
    .fetch_all(pool)
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

    let item_responses: Vec<OrderItemResponse> = items
        .into_iter()
        .map(|item| OrderItemResponse {
            uuid: item.uuid,
            order_uuid: item.order_uuid,
            product_uuid: item.product_uuid,
            qty: item.qty,
            unit_price: item.unit_price,
            unit_cost: item.unit_cost,
            line_total: item.line_total,
            created_at: item.created_at.or(Some(0)),
            updated_at: item.updated_at.or(Some(0)),
            product_name: item.product_name,
            product_sku: item.product_sku,
            product_price: item.product_price,
        })
        .collect();

    Ok(OrderResponse {
        uuid: order.uuid,
        order_no: order.order_no,
        cashier_uuid: order.cashier_uuid,
        status: order.status,
        subtotal: order.subtotal,
        discount: order.discount,
        tax: order.tax,
        total: order.total,
        net_profit: order.net_profit,
        created_at: order.created_at.or(Some(0)),
        updated_at: order.updated_at.or(Some(0)),
        items: Some(item_responses),
    })
}

pub async fn get_orders(
    State(data): State<Arc<AppState>>,
    Query(params): Query<OrderListRequest>,
) -> Result<Json<ApiResponse<OrderListResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10).min(100);
    let offset = (page - 1) * limit;

    let mut query = "SELECT o.*, COUNT(oi.uuid) as items_count FROM orders o LEFT JOIN order_items oi ON o.uuid = oi.order_uuid AND oi.deleted_at = 0 WHERE o.deleted_at = 0".to_string();
    let mut conditions = Vec::new();
    let mut bind_count = 0;

    if let Some(status) = &params.status {
        bind_count += 1;
        conditions.push(format!("o.status = ${}", bind_count));
    }

    if let Some(cashier_uuid) = params.cashier_uuid {
        bind_count += 1;
        conditions.push(format!("o.cashier_uuid = ${}", bind_count));
    }

    if let Some(date_from) = params.date_from {
        bind_count += 1;
        conditions.push(format!("o.created_at >= ${}", bind_count));
    }

    if let Some(date_to) = params.date_to {
        bind_count += 1;
        conditions.push(format!("o.created_at <= ${}", bind_count));
    }

    if let Some(search) = &params.search {
        bind_count += 1;
        conditions.push(format!("o.order_no ILIKE ${}", bind_count));
    }

    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" GROUP BY o.uuid ORDER BY o.created_at DESC");

    // Count total
    let count_query = format!(
        "SELECT COUNT(DISTINCT o.uuid) as total FROM orders o WHERE o.deleted_at = 0{}",
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
    if let Some(status) = &params.status {
        count_query_builder = count_query_builder.bind(status);
        query_builder = query_builder.bind(status);
    }

    if let Some(cashier_uuid) = params.cashier_uuid {
        count_query_builder = count_query_builder.bind(cashier_uuid);
        query_builder = query_builder.bind(cashier_uuid);
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
        query_builder = query_builder.bind(search_pattern);
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

    let orders: Vec<OrderSummaryResponse> = rows
        .into_iter()
        .map(|row| OrderSummaryResponse {
            uuid: row.get("uuid"),
            order_no: row.get("order_no"),
            cashier_uuid: row.get("cashier_uuid"),
            status: row.get("status"),
            total: row.get("total"),
            net_profit: row.get("net_profit"),
            created_at: row.get::<Option<i64>, _>("created_at").or(Some(0)),
            items_count: row.get("items_count"),
        })
        .collect();

    let total_pages = (total as f64 / limit as f64).ceil() as usize;

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Orders retrieved successfully".to_string(),
        data: OrderListResponse {
            orders,
            total,
            page,
            limit,
            total_pages,
        },
        errors: serde_json::json!(null),
    }))
}

pub async fn update_order(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrderRequest>,
) -> Result<Json<ApiResponse<OrderResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let now = chrono::Utc::now().timestamp_millis();

    let mut set_clauses = Vec::new();
    let mut bind_count = 0;

    if payload.cashier_uuid.is_some() {
        bind_count += 1;
        set_clauses.push(format!("cashier_uuid = ${}", bind_count));
    }

    if let Some(status) = &payload.status {
        // Validate status
        if !["DRAFT", "PAID", "CANCELLED", "REFUNDED"].contains(&status.to_uppercase().as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    message: "Invalid status".to_string(),
                }),
            ));
        }
        bind_count += 1;
        set_clauses.push(format!("status = ${}", bind_count));
    }

    if payload.subtotal.is_some() {
        bind_count += 1;
        set_clauses.push(format!("subtotal = ${}", bind_count));
    }

    if payload.discount.is_some() {
        bind_count += 1;
        set_clauses.push(format!("discount = ${}", bind_count));
    }

    if payload.tax.is_some() {
        bind_count += 1;
        set_clauses.push(format!("tax = ${}", bind_count));
    }

    if payload.total.is_some() {
        bind_count += 1;
        set_clauses.push(format!("total = ${}", bind_count));
    }

    if payload.net_profit.is_some() {
        bind_count += 1;
        set_clauses.push(format!("net_profit = ${}", bind_count));
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
        "UPDATE orders SET {} WHERE uuid = ${} AND deleted_at = 0",
        set_clauses.join(", "),
        bind_count
    );

    let mut query_builder = sqlx::query(&query);

    if let Some(cashier_uuid) = payload.cashier_uuid {
        query_builder = query_builder.bind(cashier_uuid);
    }

    if let Some(status) = &payload.status {
        query_builder = query_builder.bind(status);
    }

    if let Some(subtotal) = payload.subtotal {
        query_builder = query_builder.bind(subtotal);
    }

    if let Some(discount) = payload.discount {
        query_builder = query_builder.bind(discount);
    }

    if let Some(tax) = payload.tax {
        query_builder = query_builder.bind(tax);
    }

    if let Some(total) = payload.total {
        query_builder = query_builder.bind(total);
    }

    if let Some(net_profit) = payload.net_profit {
        query_builder = query_builder.bind(net_profit);
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
                message: "Order not found".to_string(),
            }),
        ));
    }

    let order = get_order_by_id_internal(&data.db, id).await?;

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Order updated successfully".to_string(),
        data: order,
        errors: serde_json::json!(null),
    }))
}

pub async fn update_order_status(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrderStatusRequest>,
) -> Result<Json<ApiResponse<OrderResponse>>, (StatusCode, Json<ErrorResponse>)> {
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

    let result = sqlx::query(
        "UPDATE orders SET status = $1, updated_at = $2 WHERE uuid = $3 AND deleted_at = 0",
    )
    .bind(payload.status.to_uppercase())
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
                message: "Order not found".to_string(),
            }),
        ));
    }

    let order = get_order_by_id_internal(&data.db, id).await?;

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Order status updated successfully".to_string(),
        data: order,
        errors: serde_json::json!(null),
    }))
}

pub async fn delete_order(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ErrorResponse>)> {
    let now = chrono::Utc::now().timestamp_millis();

    let mut tx = data.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )
    })?;

    // Soft delete order items
    sqlx::query("UPDATE order_items SET deleted_at = $1 WHERE order_uuid = $2 AND deleted_at = 0")
        .bind(now)
        .bind(id)
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

    // Soft delete order
    let result =
        sqlx::query("UPDATE orders SET deleted_at = $1 WHERE uuid = $2 AND deleted_at = 0")
            .bind(now)
            .bind(id)
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

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: "Order not found".to_string(),
            }),
        ));
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
        message: "Order deleted successfully".to_string(),
        data: serde_json::json!({"deleted": true}),
        errors: serde_json::json!(null),
    }))
}

pub async fn get_order_stats(
    State(data): State<Arc<AppState>>,
    Query(params): Query<OrderStatsRequest>,
) -> Result<Json<ApiResponse<OrderStatsResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let mut conditions = vec!["o.deleted_at = 0".to_string()];
    let mut bind_count = 0;

    if let Some(date_from) = params.date_from {
        bind_count += 1;
        conditions.push(format!("o.created_at >= ${}", bind_count));
    }

    if let Some(date_to) = params.date_to {
        bind_count += 1;
        conditions.push(format!("o.created_at <= ${}", bind_count));
    }

    if let Some(cashier_uuid) = params.cashier_uuid {
        bind_count += 1;
        conditions.push(format!("o.cashier_uuid = ${}", bind_count));
    }

    if let Some(status) = &params.status {
        bind_count += 1;
        conditions.push(format!("o.status = ${}", bind_count));
    }

    let where_clause = conditions.join(" AND ");

    // Get overall stats
    let stats_query = format!(
        r#"
        SELECT 
            COUNT(*) as total_orders,
            COALESCE(SUM(total), 0) as total_revenue,
            COALESCE(SUM(net_profit), 0) as total_profit,
            COALESCE(AVG(total), 0) as avg_order_value
        FROM orders o
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

    if let Some(cashier_uuid) = params.cashier_uuid {
        stats_query_builder = stats_query_builder.bind(cashier_uuid);
    }

    if let Some(status) = &params.status {
        stats_query_builder = stats_query_builder.bind(status);
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

    let total_orders: i64 = stats_row.get("total_orders");
    let total_revenue: rust_decimal::Decimal = stats_row.get("total_revenue");
    let total_profit: rust_decimal::Decimal = stats_row.get("total_profit");
    let avg_order_value: rust_decimal::Decimal = stats_row.get("avg_order_value");

    // Get orders by status
    let status_query = format!(
        r#"
        SELECT 
            status,
            COUNT(*) as count,
            COALESCE(SUM(total), 0) as total_amount
        FROM orders o
        WHERE {}
        GROUP BY status
        ORDER BY status
        "#,
        where_clause
    );

    let mut status_query_builder = sqlx::query(&status_query);

    if let Some(date_from) = params.date_from {
        status_query_builder = status_query_builder.bind(date_from);
    }

    if let Some(date_to) = params.date_to {
        status_query_builder = status_query_builder.bind(date_to);
    }

    if let Some(cashier_uuid) = params.cashier_uuid {
        status_query_builder = status_query_builder.bind(cashier_uuid);
    }

    if let Some(status) = &params.status {
        status_query_builder = status_query_builder.bind(status);
    }

    let status_rows = status_query_builder
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

    let orders_by_status: Vec<OrderStatusCount> = status_rows
        .into_iter()
        .map(|row| OrderStatusCount {
            status: row.get("status"),
            count: row.get("count"),
            total_amount: row.get("total_amount"),
        })
        .collect();

    // Get daily stats (last 30 days or within date range)
    let daily_query = format!(
        r#"
        SELECT 
            DATE(to_timestamp(created_at / 1000)) as date,
            COUNT(*) as orders_count,
            COALESCE(SUM(total), 0) as total_revenue,
            COALESCE(SUM(net_profit), 0) as total_profit
        FROM orders o
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

    if let Some(cashier_uuid) = params.cashier_uuid {
        daily_query_builder = daily_query_builder.bind(cashier_uuid);
    }

    if let Some(status) = &params.status {
        daily_query_builder = daily_query_builder.bind(status);
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

    let daily_stats: Vec<DailyOrderStats> = daily_rows
        .into_iter()
        .map(|row| DailyOrderStats {
            date: row.get::<chrono::NaiveDate, _>("date").to_string(),
            orders_count: row.get("orders_count"),
            total_revenue: row.get("total_revenue"),
            total_profit: row.get("total_profit"),
        })
        .collect();

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Order statistics retrieved successfully".to_string(),
        data: OrderStatsResponse {
            total_orders,
            total_revenue,
            total_profit,
            avg_order_value,
            orders_by_status,
            daily_stats,
        },
        errors: serde_json::json!(null),
    }))
}
