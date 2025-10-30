-- Revert new columns
ALTER TABLE ingredient_catalog
  DROP COLUMN IF EXISTS minimal_stock,
  DROP COLUMN IF EXISTS price,
  DROP COLUMN IF EXISTS effective_at;

-- Rename unit_of_measure_uuid back to base_unit_of_measure
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'ingredient_catalog' AND column_name = 'unit_of_measure_uuid'
  ) THEN
    EXECUTE 'ALTER TABLE ingredient_catalog RENAME COLUMN unit_of_measure_uuid TO base_unit_of_measure';
  END IF;
END$$;

-- Rename table ingredient_catalog -> ingredients
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.tables WHERE table_name = 'ingredient_catalog'
  ) THEN
    EXECUTE 'ALTER TABLE ingredient_catalog RENAME TO ingredients';
  END IF;
END$$;

-- Restore minimal_stock column if missing
ALTER TABLE ingredients
  ADD COLUMN IF NOT EXISTS minimal_stock NUMERIC(12,3);
