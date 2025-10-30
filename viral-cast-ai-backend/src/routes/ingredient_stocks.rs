use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::handlers::ingredient_stocks::{
    create_ingredient_stock_handler, delete_ingredient_stock_handler, get_ingredient_stock_handler,
    get_ingredient_stocks_handler, update_ingredient_stock_handler,
};
use crate::AppState;

pub fn create_ingredient_stocks_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/v1/ingredient-stocks",
            post(create_ingredient_stock_handler),
        )
        .route(
            "/api/v1/ingredient-stocks",
            get(get_ingredient_stocks_handler),
        )
        .route(
            "/api/v1/ingredient-stocks/:id",
            get(get_ingredient_stock_handler),
        )
        .route(
            "/api/v1/ingredient-stocks/:id",
            put(update_ingredient_stock_handler),
        )
        .route(
            "/api/v1/ingredient-stocks/:id",
            delete(delete_ingredient_stock_handler),
        )
        .with_state(app_state)
}
