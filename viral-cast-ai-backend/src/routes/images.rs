use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, post},
    Router,
};

use crate::{
    handlers::images::{
        delete_product_image_handler, upload_product_image_handler,
        upload_product_image_handler_v2, upload_store_brand_image_handler,
        upload_user_image_handler,
    },
    middleware::jwt::auth,
    AppState,
};

pub fn create_images_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/v1/images/upload/product",
            post(upload_product_image_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/images/upload/user",
            post(upload_user_image_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        // NEW: separate endpoints for profile photo and background uploads
        .route(
            "/api/v1/images/upload/user/profile-photo",
            post(upload_user_image_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/images/upload/user/background",
            post(upload_user_image_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/images/upload/store",
            post(upload_store_brand_image_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/images/upload/product/v2",
            post(upload_product_image_handler_v2)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/v1/images/delete/product",
            delete(delete_product_image_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}
