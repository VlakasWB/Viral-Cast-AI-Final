DROP INDEX IF EXISTS idx_document_chunks_vector_id;

ALTER TABLE IF EXISTS document_chunks
    DROP COLUMN IF EXISTS milvus_vector_id;
