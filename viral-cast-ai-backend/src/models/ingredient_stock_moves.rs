use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct IngredientStockMoveModel {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub ingredient_catalog_uuid: Uuid,
    pub quantity: f64,
    pub price: Option<f64>,
    pub price_updated_at: Option<i64>,
    pub effective_at: i64,        // tanggal masuk stok
    pub expiry_at: Option<i64>,   // tanggal kadaluarsa stok
    pub ref_type: Option<String>, // 'PURCHASE' | 'PRODUCTION' | 'ADJUSTMENT' | 'WASTE' | 'RETURN'
    pub ref_uuid: Option<Uuid>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: i64,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}
