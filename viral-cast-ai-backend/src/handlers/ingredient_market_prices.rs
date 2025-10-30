use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::repository::ingredient_market_prices as ingredient_market_prices_repository;
use crate::{
    dto::{
        api::ApiResponse,
        ingredient_market_prices::{
            CreateIngredientMarketPriceSchema, GetIngredientMarketPriceSchema,
            UpdateIngredientMarketPriceSchema,
        },
    },
    AppState,
};

// Handler untuk membuat harga pasar bahan baru
pub async fn create_ingredient_market_price_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateIngredientMarketPriceSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let uuid = Uuid::new_v4();
    let current_time = chrono::Utc::now().timestamp_millis();

    match ingredient_market_prices_repository::create_ingredient_market_price(
        &data.db,
        uuid,
        body,
        current_time,
    )
    .await
    {
        Ok(response) => {
            let json_response = ApiResponse {
                code: 201,
                status: "success".to_string(),
                message: "Harga pasar bahan berhasil dibuat".to_string(),
                data: Some(response),
                errors: serde_json::json!({}),
            };
            Ok((StatusCode::CREATED, Json(json_response)))
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

// Handler untuk mendapatkan daftar harga pasar bahan
pub async fn get_ingredient_market_prices_handler(
    Query(opts): Query<GetIngredientMarketPriceSchema>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let page = opts.page.unwrap_or(1);
    let limit = opts.limit.unwrap_or(10);

    match ingredient_market_prices_repository::list_ingredient_market_prices(
        &data.db,
        page,
        limit,
        opts.ingredient_catalog_uuid,
    )
    .await
    {
        Ok((prices, total)) => {
            let total_pages = ((total + limit as i64 - 1) / limit as i64).max(1);
            let offset = (page - 1) * limit;
            let total_displayed_records = prices.len() as i64;
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
                message: "Daftar harga pasar bahan berhasil diambil".to_string(),
                data: json!({
                    "ingredient_market_prices": prices,
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
                errors: serde_json::json!({}),
            };
            Ok((StatusCode::OK, Json(json_response)))
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

// Handler untuk mendapatkan harga pasar bahan berdasarkan UUID
pub async fn get_ingredient_market_price_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match ingredient_market_prices_repository::get_ingredient_market_price_by_uuid(&data.db, id)
        .await
    {
        Ok(Some(price)) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Harga pasar bahan berhasil diambil".to_string(),
                data: Some(price),
                errors: serde_json::json!({}),
            };
            Ok(Json(json_response))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Harga pasar bahan dengan ID: {} tidak ditemukan", id)
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

// Handler untuk memperbarui harga pasar bahan
pub async fn update_ingredient_market_price_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateIngredientMarketPriceSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    match ingredient_market_prices_repository::update_ingredient_market_price(
        &data.db,
        id,
        body,
        current_time,
    )
    .await
    {
        Ok(Some(updated_price)) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Harga pasar bahan berhasil diperbarui".to_string(),
                data: Some(updated_price),
                errors: serde_json::json!({}),
            };
            Ok(Json(json_response))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Harga pasar bahan dengan ID: {} tidak ditemukan", id)
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

// Handler untuk menghapus harga pasar bahan (soft delete)
pub async fn delete_ingredient_market_price_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    match ingredient_market_prices_repository::soft_delete_ingredient_market_price(
        &data.db,
        id,
        current_time,
    )
    .await
    {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Harga pasar bahan dengan ID: {} tidak ditemukan", id)
                });
                return Err((StatusCode::NOT_FOUND, Json(error_response)));
            }

            let json_response = serde_json::json!({
                "status": "success",
                "message": "Harga pasar bahan berhasil dihapus"
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
