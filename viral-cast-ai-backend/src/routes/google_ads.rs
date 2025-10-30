use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{handlers::google_ads::*, AppState};

pub fn create_google_ads_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/google-ads/campaigns", get(list_campaigns_handler))
        .route("/api/google-ads/search", post(search_handler))
        .with_state(app_state)
}
