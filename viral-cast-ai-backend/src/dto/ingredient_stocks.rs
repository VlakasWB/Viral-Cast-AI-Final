use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// DTO untuk membuat stok bahan
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIngredientStockSchema {
    pub ingredient_stock_move_uuid: Uuid,
    pub total_quantity: Option<Decimal>,
    pub total_value: Option<Decimal>,
    pub current_cost: Option<Decimal>,
    pub avg_cost: Option<Decimal>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}

// DTO untuk memperbarui stok bahan
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateIngredientStockSchema {
    pub ingredient_stock_move_uuid: Option<Uuid>,
    pub total_quantity: Option<Decimal>,
    pub total_value: Option<Decimal>,
    pub current_cost: Option<Decimal>,
    pub avg_cost: Option<Decimal>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}

// DTO untuk respons stok bahan
#[derive(Debug, Serialize, Deserialize)]
pub struct IngredientStockResponse {
    pub uuid: Uuid,
    pub ingredient_stock_move_uuid: Uuid,
    pub ingredient_catalog_uuid: Uuid,
    pub ingredient_name: Option<String>,
    pub total_quantity: Decimal,
    pub total_value: Decimal,
    pub current_cost: Option<Decimal>,
    pub avg_cost: Option<Decimal>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}

// DTO untuk query parameter
#[derive(Debug, Deserialize)]
pub struct GetIngredientStockSchema {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub ingredient_catalog_uuid: Option<Uuid>,
    pub include_deleted: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}
