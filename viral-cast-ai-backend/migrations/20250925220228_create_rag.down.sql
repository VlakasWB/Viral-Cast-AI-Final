-- Drop indexes first
DROP INDEX IF EXISTS idx_similarity_cache_expires_at;
DROP INDEX IF EXISTS idx_similarity_cache_query_hash;
DROP INDEX IF EXISTS idx_query_history_created_at;
DROP INDEX IF EXISTS idx_query_history_query_hash;
DROP INDEX IF EXISTS idx_query_history_user_id;
DROP INDEX IF EXISTS idx_access_logs_accessed_at;
DROP INDEX IF EXISTS idx_access_logs_user_id;
DROP INDEX IF EXISTS idx_access_logs_document_id;
DROP INDEX IF EXISTS idx_processing_jobs_created_at;
DROP INDEX IF EXISTS idx_processing_jobs_priority;
DROP INDEX IF EXISTS idx_processing_jobs_status;
DROP INDEX IF EXISTS idx_processing_jobs_document_id;
-- DROP INDEX IF EXISTS idx_document_chunks_embedding; -- If using pgvector
DROP INDEX IF EXISTS idx_document_chunks_content_hash;
DROP INDEX IF EXISTS idx_document_chunks_document_id;
DROP INDEX IF EXISTS idx_documents_tags;
DROP INDEX IF EXISTS idx_documents_created_at;
DROP INDEX IF EXISTS idx_documents_uploaded_by;
DROP INDEX IF EXISTS idx_documents_status;
DROP INDEX IF EXISTS idx_documents_category;

-- Drop tables in reverse order of creation (respecting foreign key constraints)
DROP TABLE IF EXISTS document_similarity_cache;
DROP TABLE IF EXISTS rag_query_history;
DROP TABLE IF EXISTS document_access_logs;
DROP TABLE IF EXISTS document_processing_jobs;
DROP TABLE IF EXISTS document_chunks;
DROP TABLE IF EXISTS documents;
DROP TABLE IF EXISTS rag_configurations;
DROP TABLE IF EXISTS document_tags;
DROP TABLE IF EXISTS document_categories;

-- Drop vector extension if it was created (uncomment if needed)
-- DROP EXTENSION IF EXISTS vector;