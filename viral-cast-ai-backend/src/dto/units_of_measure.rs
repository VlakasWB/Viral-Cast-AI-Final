use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUnitOfMeasureSchema {
    pub code: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUnitOfMeasureSchema {
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchUnitOfMeasureSchema {
    pub search: Option<String>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct GetUnitOfMeasureSchema {
    pub uuid: Uuid,
    pub code: Option<String>,
    pub name: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedUnitOfMeasureSchema {
    pub uuid: Uuid,
    pub code: Option<String>,
    pub name: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginationInfo {
    pub current_page: i64,
    pub total_pages: i64,
    pub total_items: i64,
    pub items_per_page: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnitOfMeasureListResponse {
    pub items: Vec<ProcessedUnitOfMeasureSchema>,
    pub pagination: PaginationInfo,
}
