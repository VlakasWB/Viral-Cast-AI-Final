-- Remove soft delete column (deleted_at) from regions, AI, and RAG schemas

-- Regions
ALTER TABLE IF EXISTS province DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS regency  DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS district DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS village  DROP COLUMN IF EXISTS deleted_at;

-- AI
ALTER TABLE IF EXISTS ai_config       DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS token_usage     DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS ai_request_logs DROP COLUMN IF EXISTS deleted_at;

-- RAG
ALTER TABLE IF EXISTS document_categories       DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS document_tags             DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS rag_configurations        DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS documents                 DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS document_chunks           DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS document_processing_jobs  DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS document_access_logs      DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS rag_query_history         DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE IF EXISTS document_similarity_cache DROP COLUMN IF EXISTS deleted_at;