-- Rename table: ingredients -> ingredient_catalog
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.tables WHERE table_name = 'ingredients'
  ) THEN
    EXECUTE 'ALTER TABLE ingredients RENAME TO ingredient_catalog';
  END IF;
END$$;

-- Rename column base_uom/base_unit_of_measure -> unit_of_measure_uuid
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'ingredient_catalog' AND column_name = 'base_unit_of_measure'
  ) THEN
    EXECUTE 'ALTER TABLE ingredient_catalog RENAME COLUMN base_unit_of_measure TO unit_of_measure_uuid';
  ELSIF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'ingredient_catalog' AND column_name = 'base_uom'
  ) THEN
    EXECUTE 'ALTER TABLE ingredient_catalog RENAME COLUMN base_uom TO unit_of_measure_uuid';
  END IF;
END$$;

-- Drop columns no longer used
ALTER TABLE ingredient_catalog
  DROP COLUMN IF EXISTS min_stock;

-- Add new columns per new schema
ALTER TABLE ingredient_catalog
  ADD COLUMN IF NOT EXISTS minimal_stock NUMERIC(12,3),
  ADD COLUMN IF NOT EXISTS price    NUMERIC(12,4),
  ADD COLUMN IF NOT EXISTS effective_at BIGINT;

-- Ensure deleted_at keeps default 0
ALTER TABLE ingredient_catalog
  ALTER COLUMN deleted_at SET DEFAULT 0;
