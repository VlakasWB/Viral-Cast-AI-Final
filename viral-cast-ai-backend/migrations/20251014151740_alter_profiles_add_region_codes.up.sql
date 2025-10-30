-- Add region code columns into profiles and link to region tables by code

-- Add columns (nullable to allow gradual adoption)
ALTER TABLE profiles
  ADD COLUMN IF NOT EXISTS province_code VARCHAR(20),
  ADD COLUMN IF NOT EXISTS regency_code VARCHAR(20),
  ADD COLUMN IF NOT EXISTS district_code VARCHAR(20),
  ADD COLUMN IF NOT EXISTS village_code VARCHAR(20);

-- Create indexes on the new columns for faster lookups
CREATE INDEX IF NOT EXISTS profiles_province_code_idx ON profiles(province_code);
CREATE INDEX IF NOT EXISTS profiles_regency_code_idx ON profiles(regency_code);
CREATE INDEX IF NOT EXISTS profiles_district_code_idx ON profiles(district_code);
CREATE INDEX IF NOT EXISTS profiles_village_code_idx ON profiles(village_code);

-- Add FK constraints referencing the unique code columns in region tables
ALTER TABLE profiles
  ADD CONSTRAINT profiles_province_code_fk
    FOREIGN KEY (province_code)
    REFERENCES province(code)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT;

ALTER TABLE profiles
  ADD CONSTRAINT profiles_regency_code_fk
    FOREIGN KEY (regency_code)
    REFERENCES regency(code)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT;

ALTER TABLE profiles
  ADD CONSTRAINT profiles_district_code_fk
    FOREIGN KEY (district_code)
    REFERENCES district(code)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT;

ALTER TABLE profiles
  ADD CONSTRAINT profiles_village_code_fk
    FOREIGN KEY (village_code)
    REFERENCES village(code)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT;

-- Optional: ensure provided codes are consistent with hierarchy is out of scope
-- You can enforce consistency at application layer or with triggers.