use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, FromRow)]
pub struct IngredientCatalogModel {
    pub uuid: Uuid,
    pub name: String,
    pub base_uom_uuid: Uuid,
    pub minimum_stock: Option<Decimal>,
    pub shelf_life_days: Option<i32>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}
