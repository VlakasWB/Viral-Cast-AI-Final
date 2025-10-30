use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers::forecast_daily::*, middleware::jwt::auth, AppState};

pub fn create_forecast_daily_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_forecast_daily))
        .route("/", get(get_forecast_daily))
        .route("/:id", get(get_forecast_daily_by_id))
        .route("/:id", put(update_forecast_daily))
        .route("/:id", delete(delete_forecast_daily))
        .route("/generate", post(generate_forecast))
        .route("/accuracy", get(get_forecast_accuracy))
        .route("/trend", get(get_forecast_trend))
        .layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .with_state(app_state)
}
