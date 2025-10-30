use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::dto::ingredient_stocks::{
    CreateIngredientStockSchema, GetIngredientStockSchema, UpdateIngredientStockSchema,
};
use crate::repository::ingredient_stocks;
use crate::AppState;

// Handler untuk membuat stok bahan baru
pub async fn create_ingredient_stock_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<CreateIngredientStockSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match ingredient_stocks::create_ingredient_stock(&app_state.db, &body).await {
        Ok(ingredient_stock) => Ok((StatusCode::CREATED, Json(ingredient_stock))),
        Err(e) => {
            eprintln!("Error creating ingredient stock: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to create ingredient stock: {}", e)
                })),
            ))
        }
    }
}

// Handler untuk mendapatkan daftar stok bahan
pub async fn get_ingredient_stocks_handler(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<GetIngredientStockSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match ingredient_stocks::get_ingredient_stocks(&app_state.db, &params).await {
        Ok((ingredient_stocks, count)) => Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "count": count,
                "data": ingredient_stocks
            })),
        )),
        Err(e) => {
            eprintln!("Error fetching ingredient stocks: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to fetch ingredient stocks: {}", e)
                })),
            ))
        }
    }
}

// Handler untuk mendapatkan stok bahan berdasarkan UUID
pub async fn get_ingredient_stock_handler(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match ingredient_stocks::get_ingredient_stock_by_uuid(&app_state.db, id).await {
        Ok(Some(ingredient_stock)) => Ok((StatusCode::OK, Json(ingredient_stock))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Ingredient stock dengan ID: {} tidak ditemukan", id)
            })),
        )),
        Err(e) => {
            eprintln!("Error fetching ingredient stock: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to fetch ingredient stock: {}", e)
                })),
            ))
        }
    }
}

// Handler untuk memperbarui stok bahan
pub async fn update_ingredient_stock_handler(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateIngredientStockSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match ingredient_stocks::update_ingredient_stock(&app_state.db, id, &body).await {
        Ok(ingredient_stock) => Ok((StatusCode::OK, Json(ingredient_stock))),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Ingredient stock dengan ID: {} tidak ditemukan", id)
            })),
        )),
        Err(e) => {
            eprintln!("Error updating ingredient stock: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to update ingredient stock: {}", e)
                })),
            ))
        }
    }
}

// Handler untuk menghapus stok bahan
pub async fn delete_ingredient_stock_handler(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match ingredient_stocks::delete_ingredient_stock(&app_state.db, id).await {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "message": "Ingredient stock deleted successfully"
            })),
        )),
        Err(e) => {
            eprintln!("Error deleting ingredient stock: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to delete ingredient stock: {}", e)
                })),
            ))
        }
    }
}
