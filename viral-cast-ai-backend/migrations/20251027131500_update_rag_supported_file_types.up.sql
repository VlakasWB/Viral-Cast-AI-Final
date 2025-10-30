-- Extend supported file types so Excel uploads (xlsx/xls/xlsm) are accepted by RAG validation
ALTER TABLE rag_configurations
    ALTER COLUMN supported_file_types
    SET DEFAULT ARRAY['pdf', 'txt', 'docx', 'md', 'csv', 'xlsx', 'xls', 'xlsm'];

UPDATE rag_configurations
SET supported_file_types = (
    SELECT ARRAY(
        SELECT DISTINCT lower(val)
        FROM unnest(supported_file_types || ARRAY['xlsx', 'xls', 'xlsm']) AS expanded(val)
        ORDER BY 1
    )
)
WHERE deleted_at = 0;
