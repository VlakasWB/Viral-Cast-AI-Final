-- Remove existing UNIQUE constraint on name column
ALTER TABLE categories DROP CONSTRAINT IF EXISTS categories_name_key;

-- Create unique index that only applies to non-deleted records
-- This allows reusing names after soft delete (deleted_at != 0)
CREATE UNIQUE INDEX IF NOT EXISTS uniq_categories_name_not_deleted 
ON categories (name) 
WHERE deleted_at = 0;