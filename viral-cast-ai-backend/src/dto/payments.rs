use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePaymentRequest {
    pub order_uuid: Uuid,
    pub method: String,
    pub amount: rust_decimal::Decimal,
    pub paid_at: Option<i64>,
    #[validate(length(
        max = 100,
        message = "External reference must not exceed 100 characters"
    ))]
    pub external_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePaymentRequest {
    pub method: Option<String>,
    pub amount: Option<rust_decimal::Decimal>,
    pub paid_at: Option<i64>,
    #[validate(length(
        max = 100,
        message = "External reference must not exceed 100 characters"
    ))]
    pub external_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentListRequest {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub order_uuid: Option<Uuid>,
    pub method: Option<String>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub search: Option<String>, // Search by order_no or external_ref
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub uuid: Uuid,
    pub order_uuid: Uuid,
    pub order_no: Option<String>,
    pub method: String,
    pub amount: rust_decimal::Decimal,
    pub paid_at: Option<i64>,
    pub external_ref: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSummaryResponse {
    pub uuid: Uuid,
    pub order_uuid: Uuid,
    pub order_no: String,
    pub method: String,
    pub amount: rust_decimal::Decimal,
    pub paid_at: Option<i64>,
    pub external_ref: Option<String>,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentListResponse {
    pub payments: Vec<PaymentSummaryResponse>,
    pub total: i64,
    pub page: usize,
    pub limit: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentStatsRequest {
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub method: Option<String>,
    pub order_uuid: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentStatsResponse {
    pub total_payments: i64,
    pub total_amount: rust_decimal::Decimal,
    pub avg_payment_amount: rust_decimal::Decimal,
    pub payments_by_method: Vec<PaymentMethodCount>,
    pub daily_stats: Vec<DailyPaymentStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethodCount {
    pub method: String,
    pub count: i64,
    pub total_amount: rust_decimal::Decimal,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPaymentStats {
    pub date: String,
    pub payments_count: i64,
    pub total_amount: rust_decimal::Decimal,
    pub avg_amount: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentsByOrderRequest {
    pub order_uuid: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentsByOrderResponse {
    pub order_uuid: Uuid,
    pub order_no: String,
    pub order_total: rust_decimal::Decimal,
    pub total_paid: rust_decimal::Decimal,
    pub remaining_amount: rust_decimal::Decimal,
    pub payments: Vec<PaymentResponse>,
}

// Custom validation function for payment method
fn validate_payment_method(method: &str) -> Result<(), ValidationError> {
    let valid_methods = ["CASH", "CARD", "QRIS", "TRANSFER"];
    if valid_methods.contains(&method.to_uppercase().as_str()) {
        Ok(())
    } else {
        Err(ValidationError::new(
            "Invalid payment method. Must be one of: CASH, CARD, QRIS, TRANSFER",
        ))
    }
}

// ===== QRIS DTOs =====
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateQrisRequest {
    pub order_uuid: Uuid,
    pub amount: rust_decimal::Decimal,
    #[validate(length(max = 100))]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditQrisInfo {
    pub id: String,
    pub reference_id: String,
    pub status: String,
    pub qr_string: Option<String>,
    pub qr_code_url: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrisCreateResponse {
    pub payment_uuid: Uuid,
    pub order_uuid: Uuid,
    pub method: String,
    pub amount: rust_decimal::Decimal,
    pub external_ref: String,
    pub xendit: XenditQrisInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrisStatusResponse {
    pub payment_uuid: Option<Uuid>,
    pub external_ref: String,
    pub status: String,
    pub paid: bool,
}
