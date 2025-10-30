use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProductSchema {
    pub category_uuid: Uuid,
    pub name: String,
    pub sku: Option<String>,
    pub price: Decimal,
    pub recipe_sets_uuid: Option<Uuid>,
    pub status: Option<String>, // 'ACTIVE' | 'INACTIVE', defaults to 'ACTIVE'
    pub image_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProductSchema {
    pub category_uuid: Option<Uuid>,
    pub name: Option<String>,
    pub sku: Option<String>,
    pub price: Option<Decimal>,
    pub recipe_sets_uuid: Option<Uuid>,
    pub status: Option<String>, // 'ACTIVE' | 'INACTIVE'
    pub image_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetProductSchema {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub search: Option<String>,
    pub category_uuid: Option<Uuid>,
    pub status: Option<String>,
    pub locale: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProductLocaleQuery {
    pub locale: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedProductSchema {
    pub uuid: Uuid,
    pub category_uuid: Uuid,
    pub name: String,
    pub sku: Option<String>,
    pub price: Decimal,
    pub recipe_sets_uuid: Option<Uuid>,
    pub current_recipe_name: Option<String>, // Recipe name if assigned
    pub current_recipe_yield_qty: Option<Decimal>, // Recipe yield quantity if assigned
    pub status: String,
    pub image_url: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}
