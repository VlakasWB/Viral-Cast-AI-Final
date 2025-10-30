-- Add store_uuid reference from profiles to stores

ALTER TABLE profiles
  ADD COLUMN IF NOT EXISTS store_uuid UUID;

-- Optional index to accelerate joins/lookups
CREATE INDEX IF NOT EXISTS profiles_store_uuid_idx ON profiles(store_uuid);

-- Foreign key linking to stores
ALTER TABLE profiles
  ADD CONSTRAINT profiles_store_uuid_fk
    FOREIGN KEY (store_uuid)
    REFERENCES stores(uuid)
    ON DELETE SET NULL
    ON UPDATE CASCADE;