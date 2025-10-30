ALTER TABLE rag_configurations
    ALTER COLUMN supported_file_types
    SET DEFAULT ARRAY['pdf', 'txt', 'docx', 'md', 'csv'];

UPDATE rag_configurations
SET supported_file_types = (
    SELECT ARRAY(
        SELECT val
        FROM unnest(supported_file_types) AS expanded(val)
        WHERE lower(val) NOT IN ('xlsx', 'xls', 'xlsm')
        ORDER BY 1
    )
)
WHERE deleted_at = 0;
