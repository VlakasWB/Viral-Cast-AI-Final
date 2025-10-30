use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use tracing::error;

use crate::{
    dto::{
        api::ApiResponse,
        trend_news::{
            TrendNewsArticleResponse, TrendNewsListResponse, TrendNewsQuery,
            TrendNewsSourceResponse, TrendNewsSyncRequest, TrendNewsSyncResponse,
        },
    },
    repository::trend_news::TrendNewsListParams,
    services::trend_news::{
        list_latest_trends, sync_serper_trends, TrendNewsServiceError, TrendNewsSyncOptions,
        TrendNewsSyncResult,
    },
    AppState,
};

pub async fn list_trend_news(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TrendNewsQuery>,
) -> Result<Json<ApiResponse<TrendNewsListResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(20).clamp(1, 50);
    let page = params.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    if params.refresh.unwrap_or(false) {
        let sync_options = TrendNewsSyncOptions {
            query: params.q.clone(),
            country: params.country.clone(),
            language: params.language.clone(),
            limit: Some(limit as u32),
            refresh_only_with_images: params.refresh_with_images.unwrap_or(false),
        };

        if let Err(err) = sync_serper_trends(&state.db, &state.env, sync_options).await {
            error!(error = ?err, "Failed to refresh trend news from Serper.dev");
        }
    }

    let source_codes = params.source.as_ref().map(|s| {
        s.split(',')
            .filter_map(|code| {
                let trimmed = code.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_lowercase())
                }
            })
            .collect::<Vec<_>>()
    });

    let list_params = TrendNewsListParams {
        source_codes,
        category: params.category.clone(),
        country: params.country.clone(),
        language: params.language.clone(),
        has_image: params.has_image,
        query: params.q.clone(),
        limit: limit as i64,
        offset: offset as i64,
        only_active_sources: true,
        include_deleted: false,
    };

    let (items, total) = match list_latest_trends(&state.db, list_params).await {
        Ok(result) => result,
        Err(err) => {
            error!(error = ?err, "Failed to retrieve trend news articles");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let response = TrendNewsListResponse {
        items: items
            .into_iter()
            .map(TrendNewsArticleResponse::from)
            .collect(),
        total,
        page,
        page_size: limit,
    };

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Trend news retrieved successfully".to_string(),
        data: response,
        errors: serde_json::json!({}),
    }))
}

pub async fn sync_trend_news(
    State(state): State<Arc<AppState>>,
    axum::Json(payload): axum::Json<TrendNewsSyncRequest>,
) -> Result<Json<ApiResponse<TrendNewsSyncResponse>>, StatusCode> {
    let options = TrendNewsSyncOptions {
        query: payload.query.clone(),
        country: payload.country.clone(),
        language: payload.language.clone(),
        limit: payload.limit,
        refresh_only_with_images: payload.only_with_images.unwrap_or(false),
    };

    match sync_serper_trends(&state.db, &state.env, options).await {
        Ok(result) => {
            let TrendNewsSyncResult { source, articles } = result;
            let items: Vec<TrendNewsArticleResponse> = articles
                .into_iter()
                .map(TrendNewsArticleResponse::from)
                .collect();
            let count = items.len();
            let response = TrendNewsSyncResponse {
                source: TrendNewsSourceResponse::from(&source),
                items,
                fetched_at: chrono::Utc::now().timestamp_millis(),
                count,
            };

            Ok(Json(ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Trend news synchronized successfully".to_string(),
                data: response,
                errors: serde_json::json!({}),
            }))
        }
        Err(TrendNewsServiceError::MissingApiKey) => Ok(Json(ApiResponse {
            code: 400,
            status: "error".to_string(),
            message: "SERPER_API_KEY is not configured".to_string(),
            data: TrendNewsSyncResponse {
                source: TrendNewsSourceResponse {
                    code: "".to_string(),
                    name: "".to_string(),
                    source_type: "".to_string(),
                    base_url: None,
                    description: None,
                    metadata: None,
                },
                items: vec![],
                fetched_at: chrono::Utc::now().timestamp_millis(),
                count: 0,
            },
            errors: serde_json::json!({}),
        })),
        Err(err) => {
            error!(error = ?err, "Failed to synchronize trend news");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
