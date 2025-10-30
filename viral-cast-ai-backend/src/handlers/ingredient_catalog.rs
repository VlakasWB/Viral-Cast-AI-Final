use std::sync::Arc;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::dto::i18n::I18nEntity;
use crate::dto::ingredient_market_prices::CreateIngredientMarketPriceSchema;
use crate::models::ingredient_catalog::IngredientCatalogModel;
use crate::repository::i18n as i18n_repository;
use crate::repository::ingredient_catalog as ingredient_catalog_repository;
use crate::repository::ingredient_market_prices as ingredient_market_prices_repository;
use crate::{
    dto::{
        api::ApiResponse,
        ingredient_catalog::{
            CreateIngredientSchema, GetIngredientSchema, IngredientLocaleQuery,
            ProcessedIngredientSchema, UomInfo, UpdateIngredientSchema,
        },
    },
    AppState,
};

pub async fn create_ingredient_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateIngredientSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let ingredient_uuid = Uuid::new_v4();
    let current_time = chrono::Utc::now().timestamp_millis();

    match ingredient_catalog_repository::create_ingredient(
        &data.db,
        ingredient_uuid,
        body,
        current_time,
    )
    .await
    {
        Ok(ingredient_response) => {
            // Log harga pasar saat POST/CREATE ingredient_catalog (tanpa memerlukan PUT/PATCH)
            let market_price_uuid = Uuid::new_v4();
            let _ = ingredient_market_prices_repository::create_ingredient_market_price(
                &data.db,
                market_price_uuid,
                CreateIngredientMarketPriceSchema {
                    ingredient_catalog_uuid: ingredient_uuid,
                    name: Some(ingredient_response.name.clone()),
                    price: None,
                    effective_at: current_time,
                    unit_of_measure_code: None,
                    unit_of_measure_name: None,
                },
                current_time,
            )
            .await;

            let json_response = ApiResponse {
                code: 201,
                status: "success".to_string(),
                message: "Ingredient created successfully".to_string(),
                data: Some(ingredient_response),
                errors: serde_json::json!({}),
            };
            Ok((StatusCode::CREATED, Json(json_response)))
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "Ingredient dengan nama tersebut sudah ada",
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("{:?}", e)
                })),
            ))
        }
    }
}

pub async fn get_ingredients_handler(
    Query(opts): Query<GetIngredientSchema>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let page = opts.page.unwrap_or(1);
    let limit = opts.limit.unwrap_or(10);
    let search_term = opts.search.as_ref().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    match ingredient_catalog_repository::list_ingredients(
        &data.db,
        page,
        limit,
        search_term,
        opts.sort_by.clone(),
        opts.sort_order.clone(),
    )
    .await
    {
        Ok((processed_ingredients, total)) => {
            let mut processed_ingredients = processed_ingredients;
            // Overlay terjemahan untuk name jika locale diberikan
            if let Some(locale_raw) = opts.locale.as_ref() {
                let locale = locale_raw.to_lowercase();
                for p in &mut processed_ingredients {
                    if let Ok(Some(tr)) = i18n_repository::get_translation(
                        &data.db,
                        &I18nEntity::IngredientCatalog,
                        p.uuid,
                        &locale,
                    )
                    .await
                    {
                        if let Some(name_translated) = tr
                            .get("fields")
                            .and_then(|f| f.get("name"))
                            .and_then(|v| v.as_str())
                        {
                            p.name = name_translated.to_string();
                        }
                    }
                }
            }
            let total_pages = ((total + limit as i64 - 1) / limit as i64).max(1);
            let offset = (page - 1) * limit;
            let total_displayed_records = processed_ingredients.len() as i64;
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
                message: "Ingredients fetched successfully".to_string(),
                data: json!({
                    "ingredient_catalog": processed_ingredients,
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
                "status": "fail",
                "message": "Something bad happened while fetching all ingredients",
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn get_ingredient_handler(
    Path(id): Path<Uuid>,
    Query(q): Query<IngredientLocaleQuery>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match ingredient_catalog_repository::get_ingredient_by_uuid(&data.db, id).await {
        Ok(Some(mut ingredient_response)) => {
            // Overlay terjemahan untuk name jika locale diberikan
            if let Some(locale_raw) = q.locale.as_ref() {
                let locale = locale_raw.to_lowercase();
                if let Ok(Some(tr)) = i18n_repository::get_translation(
                    &data.db,
                    &I18nEntity::IngredientCatalog,
                    ingredient_response.uuid,
                    &locale,
                )
                .await
                {
                    if let Some(name_translated) = tr
                        .get("fields")
                        .and_then(|f| f.get("name"))
                        .and_then(|v| v.as_str())
                    {
                        ingredient_response.name = name_translated.to_string();
                    }
                }
            }
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Ingredient fetched successfully".to_string(),
                data: Some(ingredient_response),
                errors: serde_json::json!({}),
            };
            Ok(Json(json_response))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Ingredient with ID: {} not found", id)
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("{:?}", e)
            })),
        )),
    }
}

pub async fn update_ingredient_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateIngredientSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    match ingredient_catalog_repository::update_ingredient(&data.db, id, body, current_time).await {
        Ok(Some(updated_ingredient_response)) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Ingredient updated successfully".to_string(),
                data: Some(updated_ingredient_response),
                errors: serde_json::json!({}),
            };
            Ok(Json(json_response))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Ingredient with ID: {} not found", id)
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "Ingredient with that name already exists",
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("{:?}", e)
                })),
            ))
        }
    }
}

pub async fn delete_ingredient_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    match ingredient_catalog_repository::soft_delete_ingredient(&data.db, id, current_time).await {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Ingredient with ID: {} not found", id)
                });
                return Err((StatusCode::NOT_FOUND, Json(error_response)));
            }

            let json_response = serde_json::json!({
                "status": "success",
                "message": "Ingredient deleted successfully"
            });

            Ok(Json(json_response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("{:?}", e)
            })),
        )),
    }
}
