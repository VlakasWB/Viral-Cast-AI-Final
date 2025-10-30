-- Revert profiles changes: re-add store fields and remove birth_place

ALTER TABLE profiles
  ADD COLUMN IF NOT EXISTS store_name VARCHAR(50),
  ADD COLUMN IF NOT EXISTS store_telp VARCHAR(50);

ALTER TABLE profiles
  DROP COLUMN IF EXISTS birth_place;