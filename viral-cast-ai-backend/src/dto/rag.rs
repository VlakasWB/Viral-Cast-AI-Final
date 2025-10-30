use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Document upload request
#[derive(Debug, Deserialize)]
pub struct DocumentUploadRequest {
    pub title: String,
    pub description: Option<String>,
    pub category: String, // POS, Trends, Weather, Sales, etc.
    pub tags: Vec<String>,
}

// Document upload response
#[derive(Debug, Serialize)]
pub struct DocumentUploadResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub file_path: String,
    pub file_size: i64,
    pub file_type: String,
    pub status: String, // processing, ready, error
    pub uploaded_at: DateTime<Utc>,
}

// Ingest text document request (no file upload)
#[derive(Debug, Deserialize)]
pub struct DocumentTextIngestRequest {
    pub title: String,
    pub category: String,
    pub content: String,
    pub tags: Option<Vec<String>>, // optional tags
    pub description: Option<String>,
}

// Ingest text document response
#[derive(Debug, Serialize)]
pub struct DocumentTextIngestResponse {
    pub document_id: Uuid,
    pub chunk_count: i32,
    pub milvus_collection: String,
    pub status: String,
}

// RAG query request
#[derive(Debug, Deserialize)]
pub struct RagQueryRequest {
    pub query: String,
    pub document_ids: Option<Vec<Uuid>>, // Specific documents to search in
    pub category_filter: Option<String>, // Filter by category
    pub max_results: Option<i32>,
    pub similarity_threshold: Option<f32>,
}

// RAG query response
#[derive(Debug, Serialize)]
pub struct RagQueryResponse {
    pub answer: String,
    pub sources: Vec<DocumentSource>,
    pub confidence_score: f32,
    pub processing_time_ms: i64,
}

// Document source information
#[derive(Debug, Serialize, Clone)]
pub struct DocumentSource {
    pub document_id: Uuid,
    pub document_title: String,
    pub chunk_text: String,
    pub similarity_score: f32,
    pub page_number: Option<i32>,
    pub chunk_index: i32,
}

// Document list request
#[derive(Debug, Deserialize)]
pub struct DocumentListRequest {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub category: Option<String>,
    pub search: Option<String>,
    pub status: Option<String>,
}

// Document list response
#[derive(Debug, Serialize)]
pub struct DocumentListResponse {
    pub documents: Vec<DocumentSummary>,
    pub total_count: i64,
    pub page: i32,
    pub limit: i32,
    pub total_pages: i32,
}

// Document summary for list view
#[derive(Debug, Serialize)]
pub struct DocumentSummary {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub file_type: String,
    pub file_size: i64,
    pub status: String,
    pub chunk_count: i32,
    pub uploaded_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
}

// Document processing status
#[derive(Debug, Serialize)]
pub struct DocumentProcessingStatus {
    pub document_id: Uuid,
    pub status: String, // processing, ready, error
    pub progress_percentage: i32,
    pub current_step: String,
    pub error_message: Option<String>,
    pub chunks_processed: i32,
    pub total_chunks: i32,
    pub estimated_completion: Option<DateTime<Utc>>,
}

// Document chunk for internal processing
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: Uuid,
    pub document_id: Uuid,
    pub chunk_index: i32,
    pub content: String,
    pub embedding: Vec<f32>,
    pub page_number: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// RAG configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct RagConfig {
    pub id: Uuid,
    pub chunk_size: i32,
    pub chunk_overlap: i32,
    pub embedding_model: String,
    pub similarity_threshold: f32,
    pub max_results: i32,
    pub enable_reranking: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Update RAG configuration request
#[derive(Debug, Deserialize)]
pub struct UpdateRagConfigRequest {
    pub chunk_size: Option<i32>,
    pub chunk_overlap: Option<i32>,
    pub embedding_model: Option<String>,
    pub similarity_threshold: Option<f32>,
    pub max_results: Option<i32>,
    pub enable_reranking: Option<bool>,
}

// Document analytics
#[derive(Debug, Serialize)]
pub struct DocumentAnalytics {
    pub total_documents: i64,
    pub documents_by_category: Vec<CategoryCount>,
    pub documents_by_status: Vec<StatusCount>,
    pub total_chunks: i64,
    pub total_storage_mb: f64,
    pub most_accessed_documents: Vec<DocumentSummary>,
    pub recent_uploads: Vec<DocumentSummary>,
}

#[derive(Debug, Serialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

// Bulk document operation request
#[derive(Debug, Deserialize)]
pub struct BulkDocumentRequest {
    pub document_ids: Vec<Uuid>,
    pub action: String, // delete, reprocess, update_category
    pub new_category: Option<String>,
}

// Bulk document operation response
#[derive(Debug, Serialize)]
pub struct BulkDocumentResponse {
    pub success_count: i32,
    pub failed_count: i32,
    pub failed_documents: Vec<BulkOperationError>,
}

#[derive(Debug, Serialize)]
pub struct BulkOperationError {
    pub document_id: Uuid,
    pub error_message: String,
}

// RAG + LLM answer request
#[derive(Debug, Deserialize)]
pub struct RagAnswerRequest {
    pub query: String,
    pub document_ids: Option<Vec<Uuid>>, // Specific documents to search in
    pub category_filter: Option<String>, // Filter by category
    pub max_results: Option<i32>,
    pub similarity_threshold: Option<f32>,
    pub prompt_instructions: Option<String>, // Optional extra instructions for the LLM
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

// RAG + LLM answer response
#[derive(Debug, Serialize)]
pub struct RagAnswerResponse {
    pub answer: String,
    pub sources: Vec<DocumentSource>,
    pub confidence_score: f32,
    pub processing_time_ms: i64,
    pub llm_model: Option<String>,
    pub tokens_used: Option<u32>,
}

// Simple RAG answer request (minimal payload: only the user query)
// ID: Endpoint sederhana agar cukup kirim pertanyaan tanpa parameter lain
// EN: Simple endpoint so the user only sends the question without extra params
#[derive(Debug, Deserialize)]
pub struct SimpleRagRequest {
    pub query: String,
}
