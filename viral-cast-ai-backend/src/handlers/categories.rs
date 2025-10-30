use std::sync::Arc;

use crate::{
    dto::api::{ApiResponse, Pagination as ApiPagination},
    dto::categories::{
        CreateCategorySchema, ListCategoryQuery, ListCategoryResponse, ProcessedCategorySchema,
        UpdateCategorySchema,
    },
    middleware::jwt::JWTAuthMiddleware,
    repository::categories as category_repository,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use uuid::Uuid;

pub async fn create_category_handler(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateCategorySchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = category_repository::create_category(&data.db, body.name).await;

    match query_result {
        Ok(category) => {
            let processed_category = ProcessedCategorySchema {
                uuid: category.uuid,
                name: category.name,
                created_at: category.created_at.or(Some(0)),
                updated_at: category.updated_at.or(Some(0)),
            };

            let response = ApiResponse {
                code: 201,
                status: "CREATED".to_string(),
                message: "Category created successfully".to_string(),
                data: processed_category,
                errors: json!({}),
            };

            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = json!({
                    "code": 409,
                    "status": "CONFLICT",
                    "message": "Category with this name already exists",
                    "data": {},
                    "errors": {"name": "Category name must be unique"},
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }

            let error_response = json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Database error: {}", e),
                "data": {},
                "errors": {},
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn get_categories_handler(
    Extension(_jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
    Query(params): Query<ListCategoryQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    let query_result = category_repository::list_categories(&data.db, &params, page, limit).await;

    match query_result {
        Ok((categories, total)) => {
            let processed_categories: Vec<ProcessedCategorySchema> = categories
                .into_iter()
                .map(|category| ProcessedCategorySchema {
                    uuid: category.uuid,
                    name: category.name,
                    created_at: category.created_at.or(Some(0)),
                    updated_at: category.updated_at.or(Some(0)),
                })
                .collect();

            let total_displayed_records = processed_categories.len() as i64;
            let total_pages = if total == 0 {
                0
            } else {
                ((total + limit - 1) / limit).max(1)
            };
            let offset = (page - 1) * limit;
            let total_remaining_records = if total > offset + total_displayed_records {
                total - (offset + total_displayed_records)
            } else {
                0
            };

            let next_page = if total_pages > 0 && page < total_pages {
                Some(page + 1)
            } else {
                None
            };
            let prev_page = if total_pages > 0 && page > 1 {
                Some(page - 1)
            } else {
                None
            };

            let pagination = ApiPagination {
                current_page: page,
                total_pages,
                next_page,
                prev_page,
                total_available_records: total,
                total_displayed_records,
                total_remaining_records,
            };

            let response_data = ListCategoryResponse {
                items: processed_categories,
                pagination,
            };

            let response = ApiResponse {
                code: 200,
                status: "OK".to_string(),
                message: "Categories retrieved successfully".to_string(),
                data: response_data,
                errors: json!({}),
            };

            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            let error_response = json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Database error: {}", e),
                "data": {},
                "errors": {},
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn get_category_handler(
    Path(category_uuid): Path<Uuid>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = category_repository::find_category_by_uuid(&data.db, category_uuid).await;

    match query_result {
        Ok(Some(category)) => {
            let processed_category = ProcessedCategorySchema {
                uuid: category.uuid,
                name: category.name,
                created_at: category.created_at.or(Some(0)),
                updated_at: category.updated_at.or(Some(0)),
            };

            let response = ApiResponse {
                code: 200,
                status: "OK".to_string(),
                message: "Category retrieved successfully".to_string(),
                data: processed_category,
                errors: json!({}),
            };

            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => {
            let error_response = json!({
                "code": 404,
                "status": "NOT_FOUND",
                "message": "Category not found",
                "data": {},
                "errors": {},
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Database error: {}", e),
                "data": {},
                "errors": {},
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn update_category_handler(
    Path(category_uuid): Path<Uuid>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateCategorySchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let query_result =
        category_repository::update_category(&data.db, category_uuid, body.name, current_time)
            .await;

    match query_result {
        Ok(Some(category)) => {
            let processed_category = ProcessedCategorySchema {
                uuid: category.uuid,
                name: category.name,
                created_at: category.created_at.or(Some(0)),
                updated_at: category.updated_at.or(Some(0)),
            };

            let response = ApiResponse {
                code: 200,
                status: "OK".to_string(),
                message: "Category updated successfully".to_string(),
                data: processed_category,
                errors: json!({}),
            };

            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => {
            let error_response = json!({
                "code": 404,
                "status": "NOT_FOUND",
                "message": "Category not found",
                "data": {},
                "errors": {},
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = json!({
                    "code": 409,
                    "status": "CONFLICT",
                    "message": "Category with this name already exists",
                    "data": {},
                    "errors": {"name": "Category name must be unique"},
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }

            let error_response = json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Database error: {}", e),
                "data": {},
                "errors": {},
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn delete_category_handler(
    Path(category_uuid): Path<Uuid>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let query_result =
        category_repository::soft_delete_category(&data.db, category_uuid, current_time).await;

    match query_result {
        Ok(rows) => {
            if rows > 0 {
                let response = ApiResponse {
                    code: 200,
                    status: "OK".to_string(),
                    message: "Category deleted successfully".to_string(),
                    data: json!({}),
                    errors: json!({}),
                };

                Ok((StatusCode::OK, Json(response)))
            } else {
                let error_response = json!({
                    "code": 404,
                    "status": "NOT_FOUND",
                    "message": "Category not found",
                    "data": {},
                    "errors": {},
                });
                Err((StatusCode::NOT_FOUND, Json(error_response)))
            }
        }
        Err(e) => {
            let error_response = json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Database error: {}", e),
                "data": {},
                "errors": {},
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
