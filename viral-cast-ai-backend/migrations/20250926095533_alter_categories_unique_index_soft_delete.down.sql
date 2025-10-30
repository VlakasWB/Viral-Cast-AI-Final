-- Drop the unique index for non-deleted records
DROP INDEX IF EXISTS uniq_categories_name_not_deleted;

-- Restore original UNIQUE constraint on name column
-- Note: This will only work if there are no duplicate names in non-deleted records
ALTER TABLE categories ADD CONSTRAINT categories_name_key UNIQUE (name);