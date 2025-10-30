use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Document model for database
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_type: String,
    pub mime_type: String,
    pub status: String, // processing, ready, error, deleted
    pub error_message: Option<String>,
    pub chunk_count: i32,
    pub processing_progress: i32, // 0-100
    pub current_processing_step: Option<String>,
    pub uploaded_by: Option<Uuid>, // User ID who uploaded
    pub access_count: i64,
    pub last_accessed: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Document chunk model for vector storage
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocumentChunk {
    pub id: Uuid,
    pub document_id: Uuid,
    pub chunk_index: i32,
    pub content: String,
    pub content_hash: String, // For deduplication
    pub embedding: Vec<f32>,
    pub page_number: Option<i32>,
    pub start_char: Option<i32>,
    pub end_char: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// RAG configuration model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RagConfiguration {
    pub id: Uuid,
    pub chunk_size: i32,
    pub chunk_overlap: i32,
    pub embedding_model: String,
    pub embedding_dimensions: i32,
    pub similarity_threshold: f32,
    pub max_results: i32,
    pub enable_reranking: bool,
    pub reranking_model: Option<String>,
    pub supported_file_types: Vec<String>,
    pub max_file_size_mb: i32,
    pub max_documents_per_user: Option<i32>,
    pub retention_days: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Document access log for analytics
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocumentAccessLog {
    pub id: Uuid,
    pub document_id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_ip: Option<String>,
    pub query: String,
    pub similarity_score: f32,
    pub response_time_ms: i64,
    pub chunks_retrieved: i32,
    pub accessed_at: DateTime<Utc>,
}

// Document processing job for background tasks
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocumentProcessingJob {
    pub id: Uuid,
    pub document_id: Uuid,
    pub job_type: String, // extract_text, generate_embeddings, reprocess
    pub status: String,   // pending, processing, completed, failed
    pub priority: i32,    // 1-10, higher is more priority
    pub progress_percentage: i32,
    pub current_step: String,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Document category configuration
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocumentCategory {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: String, // Hex color for UI
    pub icon: Option<String>,
    pub allowed_file_types: Vec<String>,
    pub max_file_size_mb: Option<i32>,
    pub auto_tags: Vec<String>, // Tags automatically applied to documents in this category
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Document tag model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocumentTag {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub usage_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Document similarity cache for performance
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocumentSimilarityCache {
    pub id: Uuid,
    pub query_hash: String,
    pub document_ids: Vec<Uuid>,
    pub similarity_scores: Vec<f32>,
    pub chunk_ids: Vec<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// RAG query history for analytics and improvement
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RagQueryHistory {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_ip: Option<String>,
    pub query: String,
    pub query_hash: String,
    pub category_filter: Option<String>,
    pub document_ids_filter: Option<Vec<Uuid>>,
    pub max_results: i32,
    pub similarity_threshold: f32,
    pub results_count: i32,
    pub top_similarity_score: f32,
    pub response_time_ms: i64,
    pub user_feedback: Option<String>, // good, bad, neutral
    pub feedback_comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Document {
    pub fn is_ready(&self) -> bool {
        self.status == "ready"
    }

    pub fn is_processing(&self) -> bool {
        self.status == "processing"
    }

    pub fn has_error(&self) -> bool {
        self.status == "error"
    }

    pub fn increment_access_count(&mut self) {
        self.access_count += 1;
        self.last_accessed = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

impl DocumentChunk {
    pub fn calculate_similarity(&self, query_embedding: &[f32]) -> f32 {
        if self.embedding.len() != query_embedding.len() {
            return 0.0;
        }

        // Cosine similarity calculation
        let dot_product: f32 = self
            .embedding
            .iter()
            .zip(query_embedding.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = query_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

impl RagConfiguration {
    pub fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            chunk_size: 1000,
            chunk_overlap: 200,
            embedding_model: "text-embedding-ada-002".to_string(),
            embedding_dimensions: 1536,
            similarity_threshold: 0.7,
            max_results: 10,
            enable_reranking: false,
            reranking_model: None,
            supported_file_types: vec![
                "pdf".to_string(),
                "txt".to_string(),
                "docx".to_string(),
                "md".to_string(),
                "csv".to_string(),
                "xlsx".to_string(),
                "xls".to_string(),
                "xlsm".to_string(),
            ],
            max_file_size_mb: 50,
            max_documents_per_user: Some(100),
            retention_days: Some(365),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_file_type_supported(&self, file_type: &str) -> bool {
        self.supported_file_types
            .contains(&file_type.to_lowercase())
    }

    pub fn is_file_size_allowed(&self, size_bytes: i64) -> bool {
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
        size_mb <= self.max_file_size_mb as f64
    }
}

impl DocumentProcessingJob {
    pub fn new(document_id: Uuid, job_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            document_id,
            job_type,
            status: "pending".to_string(),
            priority: 5,
            progress_percentage: 0,
            current_step: "Initializing".to_string(),
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn start_processing(&mut self) {
        self.status = "processing".to_string();
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn complete(&mut self) {
        self.status = "completed".to_string();
        self.progress_percentage = 100;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn fail(&mut self, error: String) {
        self.status = "failed".to_string();
        self.error_message = Some(error);
        self.retry_count += 1;
        self.updated_at = Utc::now();
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries && self.status == "failed"
    }
}
