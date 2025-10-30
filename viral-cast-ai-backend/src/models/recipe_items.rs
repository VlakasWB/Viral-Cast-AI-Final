use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, FromRow)]
pub struct RecipeItemsModel {
    pub uuid: Uuid,
    pub recipe_sets_uuid: Uuid,
    pub ingredient_stocks_uuid: Uuid,
    pub quantity: Decimal,
    pub waste_percent: Option<Decimal>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}
