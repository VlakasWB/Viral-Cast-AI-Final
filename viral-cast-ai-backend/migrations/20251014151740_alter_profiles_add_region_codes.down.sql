-- Revert region code changes from profiles

-- Drop FK constraints first
ALTER TABLE profiles DROP CONSTRAINT IF EXISTS profiles_province_code_fk;
ALTER TABLE profiles DROP CONSTRAINT IF EXISTS profiles_regency_code_fk;
ALTER TABLE profiles DROP CONSTRAINT IF EXISTS profiles_district_code_fk;
ALTER TABLE profiles DROP CONSTRAINT IF EXISTS profiles_village_code_fk;

-- Drop indexes
DROP INDEX IF EXISTS profiles_province_code_idx;
DROP INDEX IF EXISTS profiles_regency_code_idx;
DROP INDEX IF EXISTS profiles_district_code_idx;
DROP INDEX IF EXISTS profiles_village_code_idx;

-- Drop columns
ALTER TABLE profiles
  DROP COLUMN IF EXISTS province_code,
  DROP COLUMN IF EXISTS regency_code,
  DROP COLUMN IF EXISTS district_code,
  DROP COLUMN IF EXISTS village_code;