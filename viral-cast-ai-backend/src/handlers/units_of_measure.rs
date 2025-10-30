use std::sync::Arc;

use crate::repository::units_of_measure as units_repo;
use crate::{
    dto::api::ApiResponse,
    dto::units_of_measure::{
        CreateUnitOfMeasureSchema, GetUnitOfMeasureSchema, ProcessedUnitOfMeasureSchema,
        SearchUnitOfMeasureSchema, UnitOfMeasureListResponse, UpdateUnitOfMeasureSchema,
    },
    middleware::jwt::JWTAuthMiddleware,
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

pub async fn create_uom_handler(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateUnitOfMeasureSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match units_repo::create_unit_of_measure(&data.db, body).await {
        Ok(processed_uom) => {
            let response = ApiResponse {
                code: 201,
                status: "CREATED".to_string(),
                message: "UOM created successfully".to_string(),
                data: processed_uom,
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
                    "message": "UOM with this code already exists",
                    "data": {},
                    "errors": {"code": "UOM code must be unique"},
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

pub async fn get_uoms_handler(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
    Query(params): Query<SearchUnitOfMeasureSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Jika parameter pencarian diberikan, gunakan fungsi search
    if params.search.is_some()
        || params.code.is_some()
        || params.name.is_some()
        || params.page.is_some()
        || params.limit.is_some()
        || params.sort_by.is_some()
        || params.sort_order.is_some()
    {
        match units_repo::search_units_of_measure(&data.db, params).await {
            Ok(result) => {
                let response = ApiResponse {
                    code: 200,
                    status: "OK".to_string(),
                    message: "UOMs retrieved successfully".to_string(),
                    data: result,
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
    } else {
        // Jika tidak ada parameter pencarian, gunakan fungsi list biasa
        match units_repo::list_units_of_measure(&data.db).await {
            Ok(processed_uoms) => {
                // Buat format response yang sama dengan search untuk konsistensi
                let count = processed_uoms.len() as i64;
                let result = UnitOfMeasureListResponse {
                    items: processed_uoms,
                    pagination: crate::dto::units_of_measure::PaginationInfo {
                        current_page: 1,
                        total_pages: 1,
                        total_items: count,
                        items_per_page: count,
                    },
                };

                let response = ApiResponse {
                    code: 200,
                    status: "OK".to_string(),
                    message: "UOMs retrieved successfully".to_string(),
                    data: result,
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
}

pub async fn get_uom_handler(
    Path(uom_uuid): Path<Uuid>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match units_repo::get_unit_of_measure_by_uuid(&data.db, uom_uuid).await {
        Ok(Some(processed_uom)) => {
            let response = ApiResponse {
                code: 200,
                status: "OK".to_string(),
                message: "UOM retrieved successfully".to_string(),
                data: processed_uom,
                errors: json!({}),
            };

            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => {
            let error_response = json!({
                "code": 404,
                "status": "NOT_FOUND",
                "message": "UOM not found",
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

pub async fn update_uom_handler(
    Path(uom_uuid): Path<Uuid>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateUnitOfMeasureSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    match units_repo::update_unit_of_measure(&data.db, uom_uuid, body, current_time).await {
        Ok(Some(uom)) => {
            let processed_uom = ProcessedUnitOfMeasureSchema {
                uuid: uom.uuid,
                code: uom.code,
                name: uom.name,
                created_at: uom.created_at,
                updated_at: uom.updated_at,
            };

            let response = ApiResponse {
                code: 200,
                status: "OK".to_string(),
                message: "UOM updated successfully".to_string(),
                data: processed_uom,
                errors: json!({}),
            };

            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => {
            let error_response = json!({
                "code": 404,
                "status": "NOT_FOUND",
                "message": "UOM not found",
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
                    "message": "UOM with this code already exists",
                    "data": {},
                    "errors": {"code": "UOM code must be unique"},
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

pub async fn delete_uom_handler(
    Path(uom_uuid): Path<Uuid>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    match units_repo::soft_delete_unit_of_measure(&data.db, uom_uuid, current_time).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                let response = ApiResponse {
                    code: 200,
                    status: "OK".to_string(),
                    message: "UOM deleted successfully".to_string(),
                    data: json!({}),
                    errors: json!({}),
                };

                Ok((StatusCode::OK, Json(response)))
            } else {
                let error_response = json!({
                    "code": 404,
                    "status": "NOT_FOUND",
                    "message": "UOM not found",
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
