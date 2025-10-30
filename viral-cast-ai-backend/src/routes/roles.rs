use std::sync::Arc;

use axum::{middleware, routing::get, Router};

use crate::{
    handlers::roles::{get_all_roles_handler, get_roles_handler},
    middleware::jwt::auth,
    AppState,
};

pub fn create_roles_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/v1/roles",
            get(get_all_roles_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/roles/:id",
            get(get_roles_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}
