-- Alter profiles: split name into first_name and last_name

-- Add new columns (nullable to allow gradual adoption)
ALTER TABLE profiles
  ADD COLUMN IF NOT EXISTS first_name VARCHAR(255),
  ADD COLUMN IF NOT EXISTS last_name VARCHAR(255);

-- Backfill first_name from existing name for existing rows
UPDATE profiles
  SET first_name = COALESCE(first_name, name)
WHERE name IS NOT NULL;

-- Drop old single name column
ALTER TABLE profiles
  DROP COLUMN IF EXISTS name;