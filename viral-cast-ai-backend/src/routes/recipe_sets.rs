use std::sync::Arc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    handlers::recipe_sets::{
        create_recipe_set_handler, delete_recipe_set_handler, get_recipe_set_handler,
        get_recipe_sets_handler, update_recipe_set_handler,
    },
    AppState,
};

pub fn create_recipe_sets_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/recipe-sets", post(create_recipe_set_handler))
        .route("/api/recipe-sets", get(get_recipe_sets_handler))
        .route("/api/recipe-sets/:id", get(get_recipe_set_handler))
        .route("/api/recipe-sets/:id", put(update_recipe_set_handler))
        .route("/api/recipe-sets/:id", delete(delete_recipe_set_handler))
        .with_state(app_state)
}
