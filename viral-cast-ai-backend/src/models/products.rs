use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, FromRow)]
pub struct ProductsModel {
    pub uuid: Uuid,
    pub category_uuid: Uuid,
    pub name: String,
    pub sku: Option<String>,
    pub price: Decimal,
    pub recipe_sets_uuid: Option<Uuid>,
    pub status: String, // 'ACTIVE' | 'INACTIVE'
    pub image_url: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}
