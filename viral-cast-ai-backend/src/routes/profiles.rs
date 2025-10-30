use std::sync::Arc;

use axum::{middleware, routing::get, Router};

use crate::{
    handlers::profiles::{
        create_profiles_handler, get_my_profiles_handler, get_user_profiles_handler,
        update_profiles_handler_patch, update_profiles_handler_put,
    },
    middleware::jwt::auth,
    AppState,
};

pub fn create_profiles_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/v1/profiles",
            get(get_my_profiles_handler)
                .post(create_profiles_handler)
                .put(update_profiles_handler_put)
                .patch(update_profiles_handler_patch)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/profiles/:id",
            get(get_user_profiles_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}
