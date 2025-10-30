use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Order {
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
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct OrderItem {
    pub uuid: Uuid,
    pub order_uuid: Uuid,
    pub product_uuid: Uuid,
    pub qty: rust_decimal::Decimal,
    pub unit_price: rust_decimal::Decimal,
    pub unit_cost: Option<rust_decimal::Decimal>,
    pub line_total: rust_decimal::Decimal,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderWithItems {
    #[serde(flatten)]
    pub order: Order,
    pub items: Vec<OrderItemWithProduct>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrderItemWithProduct {
    pub uuid: Uuid,
    pub order_uuid: Uuid,
    pub product_uuid: Uuid,
    pub qty: rust_decimal::Decimal,
    pub unit_price: rust_decimal::Decimal,
    pub unit_cost: Option<rust_decimal::Decimal>,
    pub line_total: rust_decimal::Decimal,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
    // Product details
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub product_price: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderStatus {
    Draft,
    Paid,
    Cancelled,
    Refunded,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Draft => "DRAFT",
            OrderStatus::Paid => "PAID",
            OrderStatus::Cancelled => "CANCELLED",
            OrderStatus::Refunded => "REFUNDED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DRAFT" => Some(OrderStatus::Draft),
            "PAID" => Some(OrderStatus::Paid),
            "CANCELLED" => Some(OrderStatus::Cancelled),
            "REFUNDED" => Some(OrderStatus::Refunded),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrderSummary {
    pub uuid: Uuid,
    pub order_no: String,
    pub cashier_uuid: Option<Uuid>,
    pub status: String,
    pub total: rust_decimal::Decimal,
    pub net_profit: Option<rust_decimal::Decimal>,
    pub created_at: Option<i64>,
    pub items_count: Option<i64>,
}
