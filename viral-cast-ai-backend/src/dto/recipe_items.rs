use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRecipeItemSchema {
    pub recipe_sets_uuid: Uuid,
    pub ingredient_stocks_uuid: Uuid,
    pub quantity: Decimal,
    pub waste_percent: Option<Decimal>, // defaults to 0
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRecipeItemSchema {
    pub quantity: Option<Decimal>,
    pub waste_percent: Option<Decimal>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRecipeItemSchema {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub recipe_sets_uuid: Option<Uuid>,
    pub ingredient_stocks_uuid: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedRecipeItemSchema {
    pub uuid: Uuid,
    pub recipe_sets_uuid: Uuid,
    pub ingredient_stocks_uuid: Uuid,
    pub quantity: Decimal,
    pub waste_percent: Option<Decimal>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}
