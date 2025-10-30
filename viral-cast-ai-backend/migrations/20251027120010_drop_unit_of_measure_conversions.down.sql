-- Recreate unit_of_measure_conversions table for rollback (if needed)
-- Uses current naming: unit_of_measure_conversions with columns from_unit_of_measure/to_unit_of_measure

CREATE TABLE IF NOT EXISTS unit_of_measure_conversions (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  from_unit_of_measure UUID NOT NULL REFERENCES units_of_measure(uuid),
  to_unit_of_measure   UUID NOT NULL REFERENCES units_of_measure(uuid),
  multiplier NUMERIC(18,6) NOT NULL,
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS unit_of_measure_conversions_uniq
  ON unit_of_measure_conversions (from_unit_of_measure, to_unit_of_measure);