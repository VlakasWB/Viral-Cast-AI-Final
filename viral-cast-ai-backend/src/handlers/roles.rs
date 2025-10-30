use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::repository::roles as roles_repository;
use crate::{dto::roles::GetRolesSchema, AppState};

pub async fn get_all_roles_handler(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match roles_repository::list_roles(&data.db).await {
        Ok(response) => {
            let json_response = serde_json::json!({
                "code": 200,
                "status": "OK",
                "message": "Get data roles successfully",
                "data": response,
                "errors": {},
            });
            Ok(Json(json_response))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": "Something bad happened while fetching all roles items",
                "data": [],
                "errors": {},
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn get_roles_handler(
    Path(number): Path<i32>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match roles_repository::get_role_by_number(&data.db, number).await {
        Ok(Some(roles)) => {
            let roles_response = serde_json::json!({
                "code": 200,
                "status": "OK",
                "message": "Get data roles by number successfully",
                "data": roles,
                "errors": {}
            });
            Ok(Json(roles_response))
        }
        Ok(None) => {
            let error_response = serde_json::json!({
                "code": 404,
                "status": "NOT_FOUND",
                "message": format!("Roles with number: {} not found", number),
                "data": {},
                "errors": {}
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": "Something bad happened while fetching role by number",
                "data": {},
                "errors": {}
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
