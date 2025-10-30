use serde::{Deserialize, Serialize};
use uuid::Uuid;

// DTO untuk membuat harga pasar bahan
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIngredientMarketPriceSchema {
    pub ingredient_catalog_uuid: Uuid,
    pub name: Option<String>,
    pub price: Option<f64>,
    pub effective_at: i64,
    // optional, derived from ingredient_catalog; accepted for client convenience
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}

// DTO untuk memperbarui harga pasar bahan
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateIngredientMarketPriceSchema {
    pub ingredient_catalog_uuid: Option<Uuid>,
    pub name: Option<String>,
    pub price: Option<f64>,
    pub effective_at: Option<i64>,
    // optional, derived from ingredient_catalog; accepted for client convenience
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}

// DTO untuk respons harga pasar bahan
#[derive(Debug, Serialize, Deserialize)]
pub struct IngredientMarketPriceResponse {
    pub uuid: Uuid,
    pub ingredient_catalog_uuid: Uuid,
    pub name: Option<String>,
    pub price: Option<f64>,
    pub effective_at: i64,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
}

// DTO untuk query parameter
#[derive(Debug, Deserialize)]
pub struct GetIngredientMarketPriceSchema {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub ingredient_catalog_uuid: Option<Uuid>,
}
