use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRecipeSetSchema {
    pub name: String,                    // Made required since recipe should have a name
    pub yield_quantity: Option<Decimal>, // defaults to 1
    pub effective_from: Option<i64>,
    pub effective_to: Option<i64>,
    pub is_active: Option<bool>, // defaults to true
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRecipeSetSchema {
    pub name: Option<String>,
    pub yield_quantity: Option<Decimal>,
    pub effective_from: Option<i64>,
    pub effective_to: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRecipeSetSchema {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub is_active: Option<bool>,
    pub name: Option<String>,
    pub search: Option<String>, // Added search by recipe name
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedRecipeSetSchema {
    pub uuid: Uuid,
    pub name: String,
    pub yield_quantity: Decimal,
    pub effective_from: Option<i64>,
    pub effective_to: Option<i64>,
    pub is_active: bool,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}
