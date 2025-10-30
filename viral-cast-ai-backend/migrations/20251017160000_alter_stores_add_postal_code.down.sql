-- Remove postal_code from stores (rollback)

-- Drop index first to avoid errors
DROP INDEX IF EXISTS stores_postal_code_idx;

ALTER TABLE stores
  DROP COLUMN IF EXISTS postal_code;