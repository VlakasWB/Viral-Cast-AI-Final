use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;

use crate::{handlers::orders::*, AppState};

pub fn create_orders_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        // CRUD operations
        .route("/api/v1/orders", post(create_order))
        .route("/api/v1/orders", get(get_orders))
        .route("/api/v1/orders/:id", get(get_order_by_id))
        .route("/api/v1/orders/:id", put(update_order))
        .route("/api/v1/orders/:id", delete(delete_order))
        // Status management
        .route("/api/v1/orders/:id/status", patch(update_order_status))
        // Statistics and analytics
        .route("/api/v1/orders/stats", get(get_order_stats))
        .with_state(app_state)
}
