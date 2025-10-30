use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub uuid: Uuid,
    pub order_uuid: Uuid,
    pub method: String,
    pub amount: rust_decimal::Decimal,
    pub paid_at: Option<i64>,
    pub external_ref: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    Cash,
    Card,
    Qris,
    Transfer,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentMethod::Cash => "CASH",
            PaymentMethod::Card => "CARD",
            PaymentMethod::Qris => "QRIS",
            PaymentMethod::Transfer => "TRANSFER",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "CASH" => Some(PaymentMethod::Cash),
            "CARD" => Some(PaymentMethod::Card),
            "QRIS" => Some(PaymentMethod::Qris),
            "TRANSFER" => Some(PaymentMethod::Transfer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentWithOrder {
    // Payment fields
    pub uuid: Uuid,
    pub order_uuid: Uuid,
    pub method: String,
    pub amount: rust_decimal::Decimal,
    pub paid_at: Option<i64>,
    pub external_ref: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
    // Order fields
    pub order_no: Option<String>,
    pub order_total: Option<rust_decimal::Decimal>,
    pub order_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSummary {
    pub uuid: Uuid,
    pub order_uuid: Uuid,
    pub order_no: String,
    pub method: String,
    pub amount: rust_decimal::Decimal,
    pub paid_at: Option<i64>,
    pub external_ref: Option<String>,
    pub created_at: Option<i64>,
}
