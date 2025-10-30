use std::sync::Arc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    handlers::products::{
        create_product_handler, delete_product_handler, get_product_handler, get_products_handler,
        update_product_handler,
    },
    AppState,
};

pub fn create_products_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/products", post(create_product_handler))
        .route("/api/v1/products", get(get_products_handler))
        .route("/api/v1/products/:id", get(get_product_handler))
        .route("/api/v1/products/:id", put(update_product_handler))
        .route("/api/v1/products/:id", delete(delete_product_handler))
        .with_state(app_state)
}
