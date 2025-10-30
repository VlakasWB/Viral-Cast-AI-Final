-- Add postal_code to stores

ALTER TABLE stores
  ADD COLUMN IF NOT EXISTS postal_code VARCHAR(10);

-- Optional index to help lookups/filtering
CREATE INDEX IF NOT EXISTS stores_postal_code_idx ON stores(postal_code);