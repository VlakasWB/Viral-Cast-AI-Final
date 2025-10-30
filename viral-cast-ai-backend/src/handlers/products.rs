use std::sync::Arc;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::dto::i18n::I18nEntity;
use crate::repository::i18n as i18n_repository;
use crate::repository::products as products_repository;
use crate::{
    dto::{
        api::ApiResponse,
        products::{
            CreateProductSchema, GetProductSchema, ProcessedProductSchema, ProductLocaleQuery,
            UpdateProductSchema,
        },
    },
    AppState,
};

pub async fn create_product_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateProductSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let product_uuid = Uuid::new_v4();
    let current_time = chrono::Utc::now().timestamp_millis();

    // Validate status if provided
    if let Some(ref status) = body.status {
        if status != "ACTIVE" && status != "INACTIVE" {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Status must be either 'ACTIVE' or 'INACTIVE'"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    // Validate price is positive
    if body.price <= Decimal::ZERO {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Price must be greater than 0"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let query_result =
        products_repository::create_product(&data.db, product_uuid, body, current_time).await;

    match query_result {
        Ok(product_response) => {
            let json_response = ApiResponse {
                code: 201,
                status: "success".to_string(),
                message: "Product created successfully".to_string(),
                data: json!({"product": product_response}),
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
                    "message": "Product with that SKU already exists",
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

pub async fn get_products_handler(
    Query(opts): Query<GetProductSchema>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let page = opts.page.unwrap_or(1);
    let limit = opts.limit.unwrap_or(50);

    let query_result = products_repository::list_products(
        &data.db,
        opts.search.clone(),
        page,
        limit,
        opts.sort_by.clone(),
        opts.sort_order.clone(),
    )
    .await;

    if let Err(e) = query_result {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Something bad happened while fetching all products: {}", e),
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let (mut products, total) = query_result.unwrap();

    // Overlay terjemahan untuk name jika locale diberikan
    if let Some(locale_raw) = opts.locale.as_ref() {
        let locale = locale_raw.to_lowercase();
        for p in &mut products {
            if let Ok(Some(tr)) =
                i18n_repository::get_translation(&data.db, &I18nEntity::Product, p.uuid, &locale)
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
    let total_displayed_records = products.len() as i64;
    let offset = (page - 1) * limit;
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
        message: "Products retrieved successfully".to_string(),
        data: json!({
            "products": products,
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

pub async fn get_product_handler(
    Path(id): Path<Uuid>,
    Query(q): Query<ProductLocaleQuery>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let query_result = products_repository::get_product_by_uuid(&data.db, id).await;

    match query_result {
        Ok(Some(mut product_response)) => {
            // Overlay terjemahan untuk name jika locale diberikan
            if let Some(locale_raw) = q.locale.as_ref() {
                let locale = locale_raw.to_lowercase();
                if let Ok(Some(tr)) = i18n_repository::get_translation(
                    &data.db,
                    &I18nEntity::Product,
                    product_response.uuid,
                    &locale,
                )
                .await
                {
                    if let Some(name_translated) = tr
                        .get("fields")
                        .and_then(|f| f.get("name"))
                        .and_then(|v| v.as_str())
                    {
                        product_response.name = name_translated.to_string();
                    }
                }
            }

            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Product retrieved successfully".to_string(),
                data: json!({"product": product_response}),
                errors: json!(null),
            };
            Ok((StatusCode::OK, Json(json_response)))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Product with ID: {} not found", id)
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

pub async fn update_product_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateProductSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    // Validate status if provided
    if let Some(ref status) = body.status {
        if status != "ACTIVE" && status != "INACTIVE" {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Status must be either 'ACTIVE' or 'INACTIVE'"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    // Validate price if provided
    if let Some(price) = body.price {
        if price <= Decimal::ZERO {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Price must be greater than 0"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    let update_result = products_repository::update_product(&data.db, id, body, current_time).await;

    match update_result {
        Ok(Some(updated_product)) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Product updated successfully".to_string(),
                data: json!({"product": updated_product}),
                errors: json!(null),
            };
            Ok((StatusCode::OK, Json(json_response)))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Product with ID: {} not found", id)
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

pub async fn delete_product_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    let query_result = products_repository::soft_delete_product(&data.db, id, current_time).await;

    match query_result {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Product with ID: {} not found", id)
                });
                return Err((StatusCode::NOT_FOUND, Json(error_response)));
            }

            Ok((StatusCode::NO_CONTENT, Json(json!({}))))
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
