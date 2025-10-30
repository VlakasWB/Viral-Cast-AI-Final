ALTER TABLE IF EXISTS document_chunks
    ADD COLUMN IF NOT EXISTS milvus_vector_id BIGINT;

CREATE INDEX IF NOT EXISTS idx_document_chunks_vector_id
    ON document_chunks (milvus_vector_id);
