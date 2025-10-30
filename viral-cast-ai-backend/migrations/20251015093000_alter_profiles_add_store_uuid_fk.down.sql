-- Remove store_uuid foreign key and column from profiles

ALTER TABLE profiles
  DROP CONSTRAINT IF EXISTS profiles_store_uuid_fk;

-- Index will be dropped automatically when column is removed, but safe either way
DROP INDEX IF EXISTS profiles_store_uuid_idx;

ALTER TABLE profiles
  DROP COLUMN IF EXISTS store_uuid;