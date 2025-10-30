use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::jwt::JWTAuthMiddleware;
use crate::repository::stores as stores_repository;
use crate::{
    dto::{
        api::ApiResponse,
        stores::{CreateStoreSchema, GetStoreSchema, ProcessedStore, UpdateStoreSchema},
    },
    AppState,
};

pub async fn create_store_handler(
    State(state): State<Arc<AppState>>,
    Extension(jwt_auth): Extension<JWTAuthMiddleware>,
    Json(body): Json<CreateStoreSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_uuid = jwt_auth.user.uuid;

    // Jika user sudah memiliki store, kembalikan 409 CONFLICT
    match stores_repository::check_user_has_store(&state.db, user_uuid).await {
        Ok(true) => {
            let error_response = serde_json::json!({
                "code": 409,
                "status": "CONFLICT",
                "message": "Store sudah ada untuk user ini",
                "data": {},
                "errors": {},
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
        Ok(false) => {}
        Err(e) => {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "error",
                "message": format!("Gagal mengecek kepemilikan store user: {:?}", e),
                "data": {},
                "errors": {},
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    }

    match stores_repository::create_store(&state.db, body).await {
        Ok(processed) => {
            // Update user profile dengan store_uuid baru
            if let Err(e) =
                stores_repository::update_user_profile_store(&state.db, user_uuid, processed.uuid)
                    .await
            {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "code": 500,
                        "status": "error",
                        "message": format!("Store berhasil dibuat tetapi gagal update profil user: {:?}", e),
                        "data": {},
                        "errors": {}
                    })),
                ));
            }

            Ok(Json(ApiResponse::<ProcessedStore> {
                code: 201,
                status: "success".to_string(),
                message: "Store berhasil dibuat dan profil user diperbarui".to_string(),
                data: processed,
                errors: serde_json::json!({}),
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "status": "error",
                "message": format!("Gagal membuat store: {:?}", e),
                "data": {},
                "errors": {}
            })),
        )),
    }
}

pub async fn get_my_store_handler(
    State(state): State<Arc<AppState>>,
    Extension(jwt_auth): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_uuid = jwt_auth.user.uuid;
    match stores_repository::get_store_by_user_uuid(&state.db, user_uuid).await {
        Ok(Some(store)) => Ok(Json(ApiResponse::<ProcessedStore> {
            code: 200,
            status: "success".to_string(),
            message: "Store fetched".to_string(),
            data: store,
            errors: serde_json::json!({}),
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "code": 404,
                "status": "error",
                "message": "Store not found",
                "data": {},
                "errors": {}
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "status": "error",
                "message": format!("Failed to fetch store: {:?}", e),
                "data": {},
                "errors": {}
            })),
        )),
    }
}

pub async fn get_store_handler(
    State(state): State<Arc<AppState>>,
    Path(store_uuid): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match stores_repository::get_store_by_uuid(&state.db, store_uuid).await {
        Ok(Some(processed)) => Ok(Json(ApiResponse::<ProcessedStore> {
            code: 200,
            status: "success".to_string(),
            message: "Store fetched".to_string(),
            data: processed,
            errors: serde_json::json!({}),
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "code": 404,
                "status": "error",
                "message": "Store not found",
                "data": {},
                "errors": {}
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "status": "error",
                "message": format!("Failed to fetch store: {:?}", e),
                "data": {},
                "errors": {}
            })),
        )),
    }
}

pub async fn update_store_handler_put(
    State(state): State<Arc<AppState>>,
    Path(store_uuid): Path<Uuid>,
    Json(body): Json<UpdateStoreSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match stores_repository::update_store(&state.db, store_uuid, body).await {
        Ok(processed) => Ok(Json(ApiResponse::<ProcessedStore> {
            code: 200,
            status: "success".to_string(),
            message: "Store updated".to_string(),
            data: processed,
            errors: serde_json::json!({}),
        })),
        Err(e) => {
            let (status, code, message) = match e {
                sqlx::Error::RowNotFound => {
                    (StatusCode::NOT_FOUND, 404, "Store not found".to_string())
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    500,
                    format!("Failed to update store: {:?}", e),
                ),
            };
            Err((
                status,
                Json(json!({
                    "code": code,
                    "status": "error",
                    "message": message,
                    "data": {},
                    "errors": {}
                })),
            ))
        }
    }
}

pub async fn update_store_handler_patch(
    State(state): State<Arc<AppState>>,
    Path(store_uuid): Path<Uuid>,
    Json(body): Json<UpdateStoreSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Same behavior as PUT in this implementation (partial fields allowed)
    update_store_handler_put(State(state), Path(store_uuid), Json(body)).await
}
