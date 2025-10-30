-- Add RT, RW, and postal_code columns to profiles

ALTER TABLE profiles
  ADD COLUMN IF NOT EXISTS rt VARCHAR(3),
  ADD COLUMN IF NOT EXISTS rw VARCHAR(3),
  ADD COLUMN IF NOT EXISTS postal_code VARCHAR(10);

-- Indexes to help lookups/filtering
CREATE INDEX IF NOT EXISTS profiles_rt_idx ON profiles(rt);
CREATE INDEX IF NOT EXISTS profiles_rw_idx ON profiles(rw);
CREATE INDEX IF NOT EXISTS profiles_postal_code_idx ON profiles(postal_code);

-- No foreign keys for these fields; they are free-form administrative codes.