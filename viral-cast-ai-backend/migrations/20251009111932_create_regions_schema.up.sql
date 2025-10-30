-- Enable UUID generator (Postgres)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Province
CREATE TABLE IF NOT EXISTS province (
  uuid UUID PRIMARY KEY DEFAULT (gen_uuid_v7()),
  code VARCHAR(20) NOT NULL UNIQUE,
  name VARCHAR(100) NOT NULL
);

-- Regency
CREATE TABLE IF NOT EXISTS regency (
  uuid UUID PRIMARY KEY DEFAULT (gen_uuid_v7()),
  code VARCHAR(20) NOT NULL UNIQUE,
  name VARCHAR(100) NOT NULL,
  province_uuid UUID NOT NULL
    REFERENCES province(uuid)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
);

-- District
CREATE TABLE IF NOT EXISTS district (
  uuid UUID PRIMARY KEY DEFAULT (gen_uuid_v7()),
  code VARCHAR(20) NOT NULL UNIQUE,
  name VARCHAR(100) NOT NULL,
  regency_uuid UUID NOT NULL
    REFERENCES regency(uuid)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
);

-- Village
CREATE TABLE IF NOT EXISTS village (
  uuid UUID PRIMARY KEY DEFAULT (gen_uuid_v7()),
  code VARCHAR(20) NOT NULL UNIQUE,
  name VARCHAR(100) NOT NULL,
  district_uuid UUID NOT NULL
    REFERENCES district(uuid)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
);

-- Create indexes for foreign keys
CREATE INDEX IF NOT EXISTS regency_province_idx ON regency(province_uuid);
CREATE INDEX IF NOT EXISTS district_regency_idx ON district(regency_uuid);
CREATE INDEX IF NOT EXISTS village_district_idx ON village(district_uuid);
