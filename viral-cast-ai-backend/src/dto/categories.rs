use crate::dto::api::Pagination;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateCategorySchema {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCategorySchema {
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct GetCategorySchema {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedCategorySchema {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ListCategoryQuery {
    pub search: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub created_from: Option<i64>,
    pub created_to: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCategoryResponse {
    pub items: Vec<ProcessedCategorySchema>,
    pub pagination: Pagination,
}
