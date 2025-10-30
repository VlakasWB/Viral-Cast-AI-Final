use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    handlers::categories::{
        create_category_handler, delete_category_handler, get_categories_handler,
        get_category_handler, update_category_handler,
    },
    middleware::jwt::auth,
    AppState,
};

pub fn create_categories_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/v1/categories",
            post(create_category_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/categories",
            get(get_categories_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/categories/:id",
            get(get_category_handler)
                .patch(update_category_handler)
                .delete(delete_category_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}
