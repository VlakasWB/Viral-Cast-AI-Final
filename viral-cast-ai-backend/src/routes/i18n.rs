use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::handlers::i18n::{resolve_translation, upsert_translation};
use crate::AppState;

pub fn create_i18n_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/i18n/resolve", get(resolve_translation))
        .route("/api/i18n/upsert", post(upsert_translation))
        .with_state(app_state)
}
