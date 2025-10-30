use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

// DTO untuk membuat pergerakan stok bahan
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateIngredientStockMoveSchema {
    pub name: Option<String>,
    pub ingredient_catalog_uuid: Uuid,
    pub quantity: f64,
    pub price: Option<f64>,
    pub price_updated_at: Option<i64>,
    pub effective_at: i64,      // tanggal masuk stok
    pub expiry_at: Option<i64>, // tanggal kadaluarsa stok
    #[validate(custom(function = "validate_ref_type"))]
    pub ref_type: Option<String>, // 'PURCHASE' | 'PRODUCTION' | 'ADJUSTMENT' | 'WASTE' | 'RETURN'
    pub ref_uuid: Option<Uuid>,
    // optional, derived via ingredient_catalog -> units_of_measure; accepted for client convenience
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}

// DTO untuk memperbarui pergerakan stok bahan
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateIngredientStockMoveSchema {
    pub name: Option<String>,
    pub ingredient_catalog_uuid: Option<Uuid>,
    pub quantity: Option<f64>,
    pub price: Option<f64>,
    pub price_updated_at: Option<i64>,
    pub effective_at: Option<i64>,
    pub expiry_at: Option<i64>,
    #[validate(custom(function = "validate_ref_type"))]
    pub ref_type: Option<String>,
    pub ref_uuid: Option<Uuid>,
    // optional, derived via ingredient_catalog -> units_of_measure; accepted for client convenience
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}

// Respon DTO (diambil dari repository)
#[derive(Debug, Serialize, Deserialize)]
pub struct IngredientStockMoveResponse {
    pub uuid: Uuid,
    pub ingredient_catalog_uuid: Uuid,
    pub quantity: f64,
    pub price: Option<f64>,
    pub price_updated_at: Option<i64>,
    pub effective_at: i64,
    pub expiry_at: Option<i64>,
    pub ref_type: Option<String>,
    pub ref_uuid: Option<Uuid>,
    pub name: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}

// DTO untuk query parameter
#[derive(Debug, Deserialize)]
pub struct GetIngredientStockMoveSchema {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub ingredient_catalog_uuid: Option<Uuid>,
    pub ref_type: Option<String>,
    pub ref_uuid: Option<Uuid>,
    pub from_date: Option<i64>,
    pub to_date: Option<i64>,
    pub name: Option<String>,
    pub search: Option<String>,
    pub sort_direction: Option<String>, // "asc" atau "desc" untuk kolom effective_at
}

fn validate_ref_type(value: &str) -> Result<(), ValidationError> {
    match value.to_ascii_uppercase().as_str() {
        "PURCHASE" | "PRODUCTION" | "ADJUSTMENT" | "WASTE" | "RETURN" => Ok(()),
        _ => Err(ValidationError::new(
            "Invalid ref_type. Allowed: PURCHASE, PRODUCTION, ADJUSTMENT, WASTE, RETURN",
        )),
    }
}
