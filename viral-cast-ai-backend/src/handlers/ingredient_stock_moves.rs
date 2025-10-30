use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::dto::api::ApiResponse;
use crate::repository::ingredient_stock_moves as ingredient_stock_moves_repo;
use crate::{
    dto::ingredient_stock_moves::{
        CreateIngredientStockMoveSchema, GetIngredientStockMoveSchema, IngredientStockMoveResponse,
        UpdateIngredientStockMoveSchema,
    },
    AppState,
};

// Handler untuk membuat pergerakan stok bahan baru
// ID: Handler untuk membuat data pergerakan stok bahan baru
// EN: Handler to create new ingredient stock movement
pub async fn create_ingredient_stock_move_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateIngredientStockMoveSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    // Validasi payload
    if let Err(e) = body.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "fail",
                "message": "Invalid ref_type. Allowed: PURCHASE, PRODUCTION, ADJUSTMENT, WASTE, RETURN",
                "errors": e,
            })),
        ));
    }

    let uuid = Uuid::new_v4();
    let current_time = chrono::Utc::now().timestamp_millis();

    match ingredient_stock_moves_repo::create_ingredient_stock_move(
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
                message: "Pergerakan stok bahan berhasil dibuat".to_string(),
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

// Memperbarui pergerakan stok bahan
pub async fn update_ingredient_stock_move_handler(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateIngredientStockMoveSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    // Validasi payload
    if let Err(e) = body.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "fail",
                "message": "Invalid ref_type. Allowed: PURCHASE, PRODUCTION, ADJUSTMENT, WASTE, RETURN",
                "errors": e,
            })),
        ));
    }

    let current_time = chrono::Utc::now().timestamp_millis();

    match ingredient_stock_moves_repo::update_ingredient_stock_move(
        &data.db,
        id,
        body,
        current_time,
    )
    .await
    {
        Ok(Some(response)) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Pergerakan stok bahan berhasil diperbarui".to_string(),
                data: Some(response),
                errors: serde_json::json!({}),
            };
            Ok((StatusCode::OK, Json(json_response)))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "fail",
                "message": "Pergerakan stok bahan tidak ditemukan"
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("{:?}", e)
            })),
        )),
    }
}

// Handler untuk mendapatkan daftar pergerakan stok bahan
// ID: Handler untuk mendapatkan daftar pergerakan stok bahan dengan paginasi dan filter
// EN: Handler to get paginated list of ingredient stock movements with filters
pub async fn get_ingredient_stock_moves_handler(
    Query(opts): Query<GetIngredientStockMoveSchema>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let page = opts.page.unwrap_or(1);
    let limit = opts.limit.unwrap_or(10);

    match ingredient_stock_moves_repo::list_ingredient_stock_moves(&data.db, page, limit, opts)
        .await
    {
        Ok((stock_moves, total)) => {
            let total_pages = ((total + limit as i64 - 1) / limit as i64).max(1);
            let offset = (page - 1) * limit;
            let total_displayed_records = stock_moves.len() as i64;
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
                message: "Daftar pergerakan stok bahan berhasil diambil".to_string(),
                data: json!({
                    "ingredient_stock_moves": stock_moves,
                    "meta": {
                        "page": page,
                        "limit": limit,
                        "total_records": total,
                        "total_pages": total_pages,
                        "total_displayed_records": total_displayed_records,
                        "total_remaining_records": total_remaining_records,
                        "has_prev": has_prev,
                        "has_next": has_next,
                        "prev_page": prev_page,
                        "next_page": next_page
                    }
                }),
                errors: serde_json::json!({}),
            };
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

// Handler untuk mendapatkan pergerakan stok bahan berdasarkan UUID
// ID: Handler untuk mendapatkan detail pergerakan stok bahan berdasarkan UUID
// EN: Handler to get ingredient stock movement details by UUID
pub async fn get_ingredient_stock_move_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match ingredient_stock_moves_repo::get_ingredient_stock_move_by_uuid(&data.db, id).await {
        Ok(Some(stock_move)) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Pergerakan stok bahan berhasil diambil".to_string(),
                data: Some(stock_move),
                errors: serde_json::json!({}),
            };
            Ok(Json(json_response))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Pergerakan stok bahan dengan ID: {} tidak ditemukan", id)
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

// Handler untuk menghapus pergerakan stok bahan (soft delete)
// ID: Handler untuk menghapus (soft delete) data pergerakan stok bahan
// EN: Handler to soft delete ingredient stock movement data
pub async fn delete_ingredient_stock_move_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let current_time = chrono::Utc::now().timestamp_millis();

    match ingredient_stock_moves_repo::soft_delete_ingredient_stock_move(&data.db, id, current_time)
        .await
    {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Pergerakan stok bahan dengan ID: {} tidak ditemukan", id)
                });
                return Err((StatusCode::NOT_FOUND, Json(error_response)));
            }

            let json_response = serde_json::json!({
                "status": "success",
                "message": "Pergerakan stok bahan berhasil dihapus"
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
