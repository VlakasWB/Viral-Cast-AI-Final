use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateIngredientSchema {
    pub name: String,
    pub base_uom_uuid: Uuid,
    pub minimum_stock: Option<Decimal>,
    pub shelf_life_days: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateIngredientSchema {
    pub name: Option<String>,
    pub base_uom_uuid: Option<Uuid>,
    pub minimum_stock: Option<Decimal>,
    pub shelf_life_days: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetIngredientSchema {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub search: Option<String>,
    pub locale: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IngredientLocaleQuery {
    pub locale: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UomInfo {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedIngredientSchema {
    pub uuid: Uuid,
    pub name: String,
    pub base_uom_uuid: UomInfo,
    pub minimum_stock: Option<Decimal>,
    pub shelf_life_days: Option<i32>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}
