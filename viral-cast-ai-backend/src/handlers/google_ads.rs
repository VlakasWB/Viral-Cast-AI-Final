use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use crate::dto::google_ads::GoogleAdsSearchRequest;
use crate::services::google_ads::{GoogleAdsClient, GoogleAdsError};
use crate::AppState;

pub async fn list_campaigns_handler(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let client = GoogleAdsClient::new();
    match client.list_campaigns(None).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("{}", e))),
    }
}

pub async fn search_handler(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<GoogleAdsSearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let client = GoogleAdsClient::new();
    match client
        .search(req.customer_id.clone(), req.query.clone(), req.page_size)
        .await
    {
        Ok(v) => Ok(Json(v)),
        Err(GoogleAdsError::MissingEnv(name)) => Err((
            StatusCode::UNAUTHORIZED,
            format!("Missing configuration: {}", name),
        )),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("{}", e))),
    }
}
