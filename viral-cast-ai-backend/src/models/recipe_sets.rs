use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, FromRow)]
pub struct RecipeSetsModel {
    pub uuid: Uuid,
    pub name: String, // Made required since recipes should have names
    pub yield_quantity: Option<Decimal>,
    pub effective_from: Option<i64>,
    pub effective_to: Option<i64>,
    pub is_active: bool,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}
