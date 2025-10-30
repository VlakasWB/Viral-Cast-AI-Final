use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{handlers::trend_news::*, AppState};

pub fn create_trend_news_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/trend-news", get(list_trend_news))
        .route("/api/trend-news/sync", post(sync_trend_news))
        .with_state(app_state)
}
