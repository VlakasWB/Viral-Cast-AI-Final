use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{
    handlers::ai::{
        chat_with_ai, chat_with_ai_unlimited, get_ai_configuration, get_detailed_token_usage,
        get_token_monitoring_alerts, get_token_usage, get_token_usage_history,
        get_user_input_controls, update_ai_configuration, update_user_input_controls,
    },
    AppState,
};

pub fn create_ai_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/ai/chat", post(chat_with_ai))
        .route("/api/ai/chat/unlimited", post(chat_with_ai_unlimited))
        .route("/api/ai/config", get(get_ai_configuration))
        .route("/api/ai/config", put(update_ai_configuration))
        .route("/api/ai/token-usage", get(get_token_usage))
        .route(
            "/api/ai/token-usage/detailed",
            get(get_detailed_token_usage),
        )
        .route("/api/ai/token-usage/history", get(get_token_usage_history))
        .route(
            "/api/ai/token-usage/alerts",
            get(get_token_monitoring_alerts),
        )
        .route("/api/ai/input-controls", get(get_user_input_controls))
        .route("/api/ai/input-controls", put(update_user_input_controls))
        .with_state(app_state)
}
