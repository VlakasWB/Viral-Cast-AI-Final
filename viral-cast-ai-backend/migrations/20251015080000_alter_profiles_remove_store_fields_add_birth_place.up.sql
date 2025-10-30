-- Alter profiles: remove store fields and add birth_place

ALTER TABLE profiles
  DROP COLUMN IF EXISTS store_name,
  DROP COLUMN IF EXISTS store_telp,
  ADD COLUMN IF NOT EXISTS birth_place VARCHAR(255);