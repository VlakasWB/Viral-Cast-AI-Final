use axum::{
    middleware,
    routing::{get, patch, post, put},
    Router,
};

use crate::{
    handlers::store_ingredient_predictions::{
        generate_store_ingredient_predictions_handler, get_store_ingredient_predictions_handler,
    },
    handlers::store_product_predictions::{
        generate_store_product_predictions_handler, get_store_product_predictions_handler,
    },
    handlers::stores::{
        create_store_handler, get_my_store_handler, get_store_handler, update_store_handler_patch,
        update_store_handler_put,
    },
    middleware::jwt::auth,
    AppState,
};

pub fn create_stores_router(app_state: std::sync::Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/stores", get(get_my_store_handler))
        .route("/api/v1/stores", post(create_store_handler))
        .route("/api/v1/stores/:id", get(get_store_handler))
        .route("/api/v1/stores/:id", put(update_store_handler_put))
        .route("/api/v1/stores/:id", patch(update_store_handler_patch))
        .route(
            "/api/v1/stores/predictions",
            get(get_store_product_predictions_handler),
        )
        .route(
            "/api/v1/stores/predictions",
            post(generate_store_product_predictions_handler),
        )
        .route(
            "/api/v1/stores/ingredient-predictions",
            get(get_store_ingredient_predictions_handler),
        )
        .route(
            "/api/v1/stores/ingredient-predictions",
            post(generate_store_ingredient_predictions_handler),
        )
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .with_state(app_state)
}
