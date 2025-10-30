use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{
    handlers::ingredient_market_prices::{
        create_ingredient_market_price_handler, delete_ingredient_market_price_handler,
        get_ingredient_market_price_handler, get_ingredient_market_prices_handler,
        update_ingredient_market_price_handler,
    },
    middleware::jwt::auth,
    AppState,
};

pub fn create_ingredient_market_prices_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/v1/ingredient-market-prices",
            post(create_ingredient_market_price_handler),
        )
        .route(
            "/api/v1/ingredient-market-prices",
            get(get_ingredient_market_prices_handler),
        )
        .route(
            "/api/v1/ingredient-market-prices/:id",
            get(get_ingredient_market_price_handler),
        )
        .route(
            "/api/v1/ingredient-market-prices/:id",
            put(update_ingredient_market_price_handler),
        )
        .route(
            "/api/v1/ingredient-market-prices/:id",
            delete(delete_ingredient_market_price_handler),
        )
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .with_state(app_state)
}
