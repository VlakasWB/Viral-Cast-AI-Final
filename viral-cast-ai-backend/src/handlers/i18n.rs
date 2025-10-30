use std::sync::Arc;

use axum::{extract::Query, extract::State, http::StatusCode, response::Json};
use validator::Validate;

use crate::dto::api::{ApiResponse, ErrorResponse};
use crate::dto::i18n::{
    ResolveTranslationQuery, TranslationResultResponse, UpsertTranslationSchema,
};
use crate::repository::i18n as i18n_repo;
use crate::AppState;

pub async fn resolve_translation(
    State(data): State<Arc<AppState>>,
    Query(params): Query<ResolveTranslationQuery>,
) -> Result<Json<ApiResponse<TranslationResultResponse>>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(e) = params.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Validation error: {}", e),
            }),
        ));
    }

    match i18n_repo::get_translation(&data.db, &params.entity, params.uuid, &params.locale).await {
        Ok(Some(value)) => {
            let response = TranslationResultResponse {
                entity: serde_json::to_string(&params.entity)
                    .unwrap_or_else(|_| "unknown".to_string()),
                uuid: params.uuid,
                locale: params.locale.to_lowercase(),
                fields: value
                    .get("fields")
                    .cloned()
                    .unwrap_or(serde_json::json!({})),
            };
            Ok(Json(ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Translation resolved successfully".to_string(),
                data: response,
                errors: serde_json::json!(null),
            }))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: "Translation not found for given entity/uuid/locale".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )),
    }
}

pub async fn upsert_translation(
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpsertTranslationSchema>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(e) = body.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Validation error: {}", e),
            }),
        ));
    }

    match i18n_repo::upsert_translation(&data.db, &body).await {
        Ok(result) => Ok(Json(ApiResponse {
            code: 200,
            status: "success".to_string(),
            message: "Translation upserted successfully".to_string(),
            data: result,
            errors: serde_json::json!(null),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            }),
        )),
    }
}
