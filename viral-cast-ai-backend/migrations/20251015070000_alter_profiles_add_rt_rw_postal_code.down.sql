-- Remove RT, RW, and postal_code columns from profiles

ALTER TABLE profiles
  DROP COLUMN IF EXISTS rt,
  DROP COLUMN IF EXISTS rw,
  DROP COLUMN IF EXISTS postal_code;

DROP INDEX IF EXISTS profiles_rt_idx;
DROP INDEX IF EXISTS profiles_rw_idx;
DROP INDEX IF EXISTS profiles_postal_code_idx;