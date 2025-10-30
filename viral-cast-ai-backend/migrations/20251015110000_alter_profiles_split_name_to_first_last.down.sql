-- Revert profiles name split: restore single name column and remove first_name/last_name

-- Add back name column
ALTER TABLE profiles
  ADD COLUMN IF NOT EXISTS name VARCHAR(255);

-- Backfill name from first_name and last_name
UPDATE profiles
  SET name = COALESCE(name, NULLIF(CONCAT_WS(' ', first_name, last_name), ''))
WHERE name IS NULL;

-- Drop new columns
ALTER TABLE profiles
  DROP COLUMN IF EXISTS first_name,
  DROP COLUMN IF EXISTS last_name;