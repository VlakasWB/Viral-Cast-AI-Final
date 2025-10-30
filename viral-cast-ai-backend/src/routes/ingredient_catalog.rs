use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    handlers::ingredient_catalog::{
        create_ingredient_handler, delete_ingredient_handler, get_ingredient_handler,
        get_ingredients_handler, update_ingredient_handler,
    },
    middleware::jwt::auth,
    AppState,
};

pub fn create_ingredients_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/v1/ingredient-catalog",
            post(create_ingredient_handler),
        )
        .route("/api/v1/ingredient-catalog", get(get_ingredients_handler))
        .route(
            "/api/v1/ingredient-catalog/:id",
            get(get_ingredient_handler),
        )
        .route(
            "/api/v1/ingredient-catalog/:id",
            put(update_ingredient_handler),
        )
        .route(
            "/api/v1/ingredient-catalog/:id",
            delete(delete_ingredient_handler),
        )
        .layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .with_state(app_state)
}
