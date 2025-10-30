use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use crate::{
    handlers::ingredient_stock_moves::{
        create_ingredient_stock_move_handler, delete_ingredient_stock_move_handler,
        get_ingredient_stock_move_handler, get_ingredient_stock_moves_handler,
        update_ingredient_stock_move_handler,
    },
    middleware::jwt::auth,
    AppState,
};

pub fn create_ingredient_stock_moves_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/v1/ingredient-stock-moves",
            post(create_ingredient_stock_move_handler),
        )
        .route(
            "/api/v1/ingredient-stock-moves",
            get(get_ingredient_stock_moves_handler),
        )
        .route(
            "/api/v1/ingredient-stock-moves/:id",
            get(get_ingredient_stock_move_handler),
        )
        .route(
            "/api/v1/ingredient-stock-moves/:id",
            patch(update_ingredient_stock_move_handler),
        )
        .route(
            "/api/v1/ingredient-stock-moves/:id",
            delete(delete_ingredient_stock_move_handler),
        )
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .with_state(app_state)
}
