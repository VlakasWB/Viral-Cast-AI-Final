use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct IngredientMarketPriceModel {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub ingredient_catalog_uuid: Uuid,
    pub price: Option<f64>,
    pub effective_at: i64,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: i64,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}
