use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{handlers::payments::*, AppState};

pub fn create_payments_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/payments", post(create_payment))
        .route("/api/v1/payments", get(get_payments))
        .route("/api/v1/payments/:id", get(get_payment_by_id))
        .route("/api/v1/payments/:id", put(update_payment))
        .route("/api/v1/payments/:id", delete(delete_payment))
        .route("/api/v1/payments/stats", get(get_payment_stats))
        .route(
            "/api/v1/payments/order/:order_uuid",
            get(get_payments_by_order),
        )
        // QRIS (Xendit) routes
        .route(
            "/api/v1/payments/qris/sandbox",
            post(create_qris_payment_sandbox),
        )
        .route("/api/v1/payments/qris/live", post(create_qris_payment_live))
        .route(
            "/api/v1/payments/qris/sandbox/:external_ref/status",
            get(get_qris_status_sandbox),
        )
        .route(
            "/api/v1/payments/qris/live/:external_ref/status",
            get(get_qris_status_live),
        )
        .route(
            "/api/v1/payments/xendit/webhook",
            post(xendit_webhook_handler),
        )
        .with_state(app_state)
}
