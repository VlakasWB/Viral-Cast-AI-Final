use std::sync::Arc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    handlers::recipe_items::{
        create_recipe_item_handler, delete_recipe_item_handler, get_recipe_item_handler,
        get_recipe_items_handler, update_recipe_item_handler,
    },
    AppState,
};

pub fn create_recipe_items_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/recipe-items", post(create_recipe_item_handler))
        .route("/api/recipe-items", get(get_recipe_items_handler))
        .route("/api/recipe-items/:id", get(get_recipe_item_handler))
        .route("/api/recipe-items/:id", put(update_recipe_item_handler))
        .route("/api/recipe-items/:id", delete(delete_recipe_item_handler))
        .with_state(app_state)
}
