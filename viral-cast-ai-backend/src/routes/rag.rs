use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{
    handlers::rag::{
        answer_simple_rag, answer_with_rag_and_llm, delete_document, get_document_status,
        get_rag_configuration, ingest_text_document, list_documents, query_rag,
        update_rag_configuration, upload_document,
    },
    AppState,
};

pub fn create_rag_router() -> Router<Arc<AppState>> {
    Router::new()
        // Document management endpoints
        .route("/documents/upload-document", post(ingest_text_document))
        .route("/documents/upload", post(upload_document))
        .route("/documents", get(list_documents))
        .route("/documents/:id/status", get(get_document_status))
        .route("/documents/:id", delete(delete_document))
        // RAG query endpoint
        .route("/query", post(query_rag))
        .route("/answer", post(answer_with_rag_and_llm))
        .route("/answer/simple", post(answer_simple_rag))
        // Configuration endpoints
        .route("/config", get(get_rag_configuration))
        .route("/config", put(update_rag_configuration))
}
