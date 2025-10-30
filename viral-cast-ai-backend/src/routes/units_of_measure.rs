use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    handlers::units_of_measure::{
        create_uom_handler, delete_uom_handler, get_uom_handler, get_uoms_handler,
        update_uom_handler,
    },
    middleware::jwt::auth,
    AppState,
};

pub fn create_uoms_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/v1/uoms",
            post(create_uom_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/uoms",
            get(get_uoms_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/uoms/:id",
            get(get_uom_handler)
                .patch(update_uom_handler)
                .delete(delete_uom_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}

// Provide an alias to match main.rs usage
pub fn create_units_of_measure_router(app_state: Arc<AppState>) -> Router {
    create_uoms_router(app_state)
}
