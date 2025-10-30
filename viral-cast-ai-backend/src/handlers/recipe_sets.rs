use std::sync::Arc;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::repository::recipe_sets as recipe_sets_repository;
use crate::{
    dto::{
        api::ApiResponse,
        recipe_sets::{
            CreateRecipeSetSchema, GetRecipeSetSchema, ProcessedRecipeSetSchema,
            UpdateRecipeSetSchema,
        },
    },
    AppState,
};

pub async fn create_recipe_set_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateRecipeSetSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let recipe_set_uuid = Uuid::new_v4();
    let current_time = chrono::Utc::now().timestamp_millis();

    let yield_quantity = body.yield_quantity.unwrap_or(Decimal::from(1));
    let is_active = body.is_active.unwrap_or(true);

    if yield_quantity <= Decimal::ZERO {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Yield quantity must be greater than 0"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    if let (Some(from), Some(to)) = (body.effective_from, body.effective_to) {
        if to < from {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Effective_to must be greater than or equal to effective_from"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    match recipe_sets_repository::create_recipe_set(&data.db, recipe_set_uuid, body, current_time)
        .await
    {
        Ok(recipe_set_response) => {
            let json_response = ApiResponse {
                code: 201,
                status: "success".to_string(),
                message: "Recipe set created successfully".to_string(),
                data: json!({"recipe_set": recipe_set_response}),
                errors: json!(null),
            };
            Ok((StatusCode::CREATED, Json(json_response)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", e),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn get_recipe_sets_handler(
    Query(opts): Query<GetRecipeSetSchema>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let page = opts.page.unwrap_or(1);
    let limit = opts.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    match recipe_sets_repository::list_recipe_sets(&data.db, opts).await {
        Ok((recipe_set_responses, total)) => {
            let total_pages = ((total + limit as i64 - 1) / limit as i64).max(1);
            let total_displayed_records = recipe_set_responses.len() as i64;
            let total_remaining_records = if total >= (offset as i64 + total_displayed_records) {
                total - (offset as i64 + total_displayed_records)
            } else {
                0
            };
            let has_prev = page > 1;
            let has_next = (page as i64) < total_pages;
            let prev_page = if has_prev { Some(page - 1) } else { None };
            let next_page = if has_next { Some(page + 1) } else { None };

            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Recipe sets retrieved successfully".to_string(),
                data: json!({
                    "recipe_sets": recipe_set_responses,
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
                "message": "Something bad happened while fetching all recipe sets",
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn get_recipe_set_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match recipe_sets_repository::get_recipe_set_by_uuid(&data.db, id).await {
        Ok(Some(recipe_set_response)) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Recipe set retrieved successfully".to_string(),
                data: json!({"recipe_set": recipe_set_response}),
                errors: json!(null),
            };
            Ok((StatusCode::OK, Json(json_response)))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Recipe set with ID: {} not found", id)
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

pub async fn update_recipe_set_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateRecipeSetSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    if let Some(yield_quantity) = body.yield_quantity {
        if yield_quantity <= Decimal::ZERO {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Yield quantity must be greater than 0"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    // Get current for effective range validation
    let current = recipe_sets_repository::get_recipe_set_model_by_uuid(&data.db, id).await;
    let Some(existing) = (match current {
        Ok(c) => c,
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    }) else {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Recipe set with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    };

    let effective_from = body.effective_from.or(existing.effective_from);
    let effective_to = body.effective_to.or(existing.effective_to);

    if let (Some(from), Some(to)) = (effective_from, effective_to) {
        if to < from {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Effective_to must be greater than or equal to effective_from"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    match recipe_sets_repository::update_recipe_set(&data.db, id, body, current_time).await {
        Ok(Some(updated_recipe_set)) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Recipe set updated successfully".to_string(),
                data: json!({"recipe_set": updated_recipe_set}),
                errors: json!(null),
            };
            Ok((StatusCode::OK, Json(json_response)))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Recipe set with ID: {} not found", id)
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

pub async fn delete_recipe_set_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    match recipe_sets_repository::soft_delete_recipe_set(&data.db, id, current_time).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, Json(json!({})))),
        Ok(false) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Recipe set with ID: {} not found", id)
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
