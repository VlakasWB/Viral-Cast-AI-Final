use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(
        min = 1,
        max = 30,
        message = "Order number must be between 1 and 30 characters"
    ))]
    pub order_no: String,
    pub cashier_uuid: Option<Uuid>,
    pub subtotal: rust_decimal::Decimal,
    pub discount: Option<rust_decimal::Decimal>,
    pub tax: Option<rust_decimal::Decimal>,
    pub total: rust_decimal::Decimal,
    pub net_profit: Option<rust_decimal::Decimal>,
    #[validate(length(min = 1, message = "Order must have at least one item"))]
    pub items: Vec<CreateOrderItemRequest>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOrderItemRequest {
    pub product_uuid: Uuid,
    pub qty: rust_decimal::Decimal,
    pub unit_price: rust_decimal::Decimal,
    pub unit_cost: Option<rust_decimal::Decimal>,
    pub line_total: rust_decimal::Decimal,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateOrderRequest {
    pub cashier_uuid: Option<Uuid>,
    pub status: Option<String>,
    pub subtotal: Option<rust_decimal::Decimal>,
    pub discount: Option<rust_decimal::Decimal>,
    pub tax: Option<rust_decimal::Decimal>,
    pub total: Option<rust_decimal::Decimal>,
    pub net_profit: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderListRequest {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub status: Option<String>,
    pub cashier_uuid: Option<Uuid>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    pub uuid: Uuid,
    pub order_no: String,
    pub cashier_uuid: Option<Uuid>,
    pub status: String,
    pub subtotal: rust_decimal::Decimal,
    pub discount: rust_decimal::Decimal,
    pub tax: rust_decimal::Decimal,
    pub total: rust_decimal::Decimal,
    pub net_profit: Option<rust_decimal::Decimal>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub items: Option<Vec<OrderItemResponse>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemResponse {
    pub uuid: Uuid,
    pub order_uuid: Uuid,
    pub product_uuid: Uuid,
    pub qty: rust_decimal::Decimal,
    pub unit_price: rust_decimal::Decimal,
    pub unit_cost: Option<rust_decimal::Decimal>,
    pub line_total: rust_decimal::Decimal,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub product_price: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderSummaryResponse {
    pub uuid: Uuid,
    pub order_no: String,
    pub cashier_uuid: Option<Uuid>,
    pub status: String,
    pub total: rust_decimal::Decimal,
    pub net_profit: Option<rust_decimal::Decimal>,
    pub created_at: Option<i64>,
    pub items_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderListResponse {
    pub orders: Vec<OrderSummaryResponse>,
    pub total: i64,
    pub page: usize,
    pub limit: usize,
    pub total_pages: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderStatsRequest {
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub cashier_uuid: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderStatsResponse {
    pub total_orders: i64,
    pub total_revenue: rust_decimal::Decimal,
    pub total_profit: rust_decimal::Decimal,
    pub avg_order_value: rust_decimal::Decimal,
    pub orders_by_status: Vec<OrderStatusCount>,
    pub daily_stats: Vec<DailyOrderStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderStatusCount {
    pub status: String,
    pub count: i64,
    pub total_amount: rust_decimal::Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyOrderStats {
    pub date: String,
    pub orders_count: i64,
    pub total_revenue: rust_decimal::Decimal,
    pub total_profit: rust_decimal::Decimal,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
}

fn validate_order_status(status: &str) -> Result<(), ValidationError> {
    match status.to_uppercase().as_str() {
        "DRAFT" | "PAID" | "CANCELLED" | "REFUNDED" => Ok(()),
        _ => Err(ValidationError::new(
            "Invalid order status. Must be one of: DRAFT, PAID, CANCELLED, REFUNDED",
        )),
    }
}
