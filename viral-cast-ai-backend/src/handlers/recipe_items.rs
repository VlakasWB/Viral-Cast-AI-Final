use std::sync::Arc;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use sqlx::query_as;
use uuid::Uuid;

use crate::repository::recipe_items as recipe_items_repository;
use crate::{
    dto::{
        api::ApiResponse,
        recipe_items::{
            CreateRecipeItemSchema, GetRecipeItemSchema, ProcessedRecipeItemSchema,
            UpdateRecipeItemSchema,
        },
    },
    AppState,
};

pub async fn create_recipe_item_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateRecipeItemSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let item_uuid = Uuid::new_v4();
    let current_time = chrono::Utc::now().timestamp_millis();

    if body.quantity <= Decimal::ZERO {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Quantity must be greater than 0"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    if let Some(waste_percent) = body.waste_percent {
        if waste_percent < Decimal::ZERO || waste_percent > Decimal::from(1) {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Waste percentage must be between 0 and 1"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    match recipe_items_repository::create_recipe_item(&data.db, item_uuid, body, current_time).await
    {
        Ok(recipe_item_response) => {
            let json_response = ApiResponse {
                code: 201,
                status: "success".to_string(),
                message: "Recipe item created successfully".to_string(),
                data: json!({"recipe_item": recipe_item_response}),
                errors: json!(null),
            };
            Ok((StatusCode::CREATED, Json(json_response)))
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "Recipe item with that ingredient already exists for this recipe",
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }

            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", e),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn get_recipe_items_handler(
    Query(opts): Query<GetRecipeItemSchema>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let page = opts.page.unwrap_or(1);
    let limit = opts.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    match recipe_items_repository::list_recipe_items(&data.db, opts).await {
        Ok(recipe_item_responses) => {
            let total = recipe_item_responses.len() as i64;
            let total_pages = ((total + limit as i64 - 1) / limit as i64).max(1);
            let total_displayed_records = recipe_item_responses.len() as i64;
            let total_remaining_records = 0;
            let has_prev = page > 1;
            let has_next = (page as i64) < total_pages;
            let prev_page = if has_prev { Some(page - 1) } else { None };
            let next_page = if has_next { Some(page + 1) } else { None };

            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Recipe items retrieved successfully".to_string(),
                data: json!({
                    "recipe_items": recipe_item_responses,
                    "pagination": {
                        "page": page,
                        "limit": limit,
                        "total": total,
                        "total_pages": total_pages,
                        "has_prev": has_prev,
                        "has_next": has_next,
                        "prev_page": prev_page,
                        "next_page": next_page,
                        "total_displayed_records": total_displayed_records,
                        "total_remaining_records": total_remaining_records
                    }
                }),
                errors: json!(null),
            };
            Ok((StatusCode::OK, Json(json_response)))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": "Something bad happened while fetching all recipe items",
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn get_recipe_item_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match recipe_items_repository::get_recipe_item_by_uuid(&data.db, id).await {
        Ok(Some(recipe_item_response)) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Recipe item retrieved successfully".to_string(),
                data: json!({"recipe_item": recipe_item_response}),
                errors: json!(null),
            };
            Ok((StatusCode::OK, Json(json_response)))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Recipe item with ID: {} not found", id)
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn update_recipe_item_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateRecipeItemSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    if let Some(quantity) = body.quantity {
        if quantity <= Decimal::ZERO {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Quantity must be greater than 0"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    if let Some(waste_percent) = body.waste_percent {
        if waste_percent < Decimal::ZERO || waste_percent > Decimal::from(1) {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Waste percentage must be between 0 and 1"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    match recipe_items_repository::update_recipe_item(&data.db, id, body, current_time).await {
        Ok(Some(updated_recipe_item)) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Recipe item updated successfully".to_string(),
                data: json!({"recipe_item": updated_recipe_item}),
                errors: json!(null),
            };
            Ok((StatusCode::OK, Json(json_response)))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Recipe item with ID: {} not found", id)
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn delete_recipe_item_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    match recipe_items_repository::soft_delete_recipe_item(&data.db, id, current_time).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, Json(json!({})))),
        Ok(false) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Recipe item with ID: {} not found", id)
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
