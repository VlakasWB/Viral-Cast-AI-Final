-- Create stores table (brand/outlet store entity)

CREATE TABLE IF NOT EXISTS stores (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  brand_url VARCHAR(255),

  -- Region codes (nullable for gradual adoption)
  province_code VARCHAR(20),
  regency_code VARCHAR(20),
  district_code VARCHAR(20),
  village_code VARCHAR(20),

  -- Administrative neighborhood codes
  rt VARCHAR(3),
  rw VARCHAR(3),

  -- Contact fields
  telp VARCHAR(25),
  whatsapp VARCHAR(50),
  instagram VARCHAR(255),

  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0
);

-- Indexes to speed up lookups by region codes
CREATE INDEX IF NOT EXISTS stores_province_code_idx ON stores(province_code);
CREATE INDEX IF NOT EXISTS stores_regency_code_idx ON stores(regency_code);
CREATE INDEX IF NOT EXISTS stores_district_code_idx ON stores(district_code);
CREATE INDEX IF NOT EXISTS stores_village_code_idx ON stores(village_code);

-- Foreign key constraints to region tables by code
ALTER TABLE stores
  ADD CONSTRAINT stores_province_code_fk
    FOREIGN KEY (province_code)
    REFERENCES province(code)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT;

ALTER TABLE stores
  ADD CONSTRAINT stores_regency_code_fk
    FOREIGN KEY (regency_code)
    REFERENCES regency(code)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT;

ALTER TABLE stores
  ADD CONSTRAINT stores_district_code_fk
    FOREIGN KEY (district_code)
    REFERENCES district(code)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT;

ALTER TABLE stores
  ADD CONSTRAINT stores_village_code_fk
    FOREIGN KEY (village_code)
    REFERENCES village(code)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT;