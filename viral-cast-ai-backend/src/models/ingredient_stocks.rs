use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct IngredientStockModel {
    pub uuid: Uuid,
    pub ingredient_stock_moves_uuid: Uuid,
    pub ingredient_catalog_uuid: Uuid,
    pub ingredient_name: Option<String>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
    pub total_quantity: Decimal,
    pub total_value: Decimal,
    pub current_cost: Option<Decimal>,
    pub avg_cost: Option<Decimal>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: i64,
}
