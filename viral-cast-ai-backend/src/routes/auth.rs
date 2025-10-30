use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    handlers::auth::{
        get_me_handler, health_checker_handler, login_user_handler, logout_handler,
        refresh_access_token_handler, register_user_handler,
    },
    middleware::jwt::auth,
    AppState,
};

pub fn create_auth_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/healthchecker", get(health_checker_handler))
        .route("/api/v1/auth/register", post(register_user_handler))
        .route("/api/v1/auth/login", post(login_user_handler))
        .route("/api/v1/auth/refresh", get(refresh_access_token_handler))
        .route(
            "/api/v1/auth/logout",
            post(logout_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/users/me",
            get(get_me_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}
