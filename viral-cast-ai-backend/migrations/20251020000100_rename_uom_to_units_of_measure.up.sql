-- Rename tables and columns related to UOM to Units of Measure

-- 1) uoms -> units_of_measure
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.tables WHERE table_name = 'uoms'
  ) THEN
    EXECUTE 'ALTER TABLE uoms RENAME TO units_of_measure';
  END IF;
END$$;

-- 2) ingredients.base_uom -> ingredients.base_unit_of_measure
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'ingredients' AND column_name = 'base_uom'
  ) THEN
    EXECUTE 'ALTER TABLE ingredients RENAME COLUMN base_uom TO base_unit_of_measure';
  END IF;
END$$;

-- 3) uom_conversions -> unit_of_measure_conversions
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.tables WHERE table_name = 'uom_conversions'
  ) THEN
    EXECUTE 'ALTER TABLE uom_conversions RENAME TO unit_of_measure_conversions';
  END IF;
END$$;

-- 3a) Columns: from_uom -> from_unit_of_measure; to_uom -> to_unit_of_measure
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'unit_of_measure_conversions' AND column_name = 'from_uom'
  ) THEN
    EXECUTE 'ALTER TABLE unit_of_measure_conversions RENAME COLUMN from_uom TO from_unit_of_measure';
  END IF;
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'unit_of_measure_conversions' AND column_name = 'to_uom'
  ) THEN
    EXECUTE 'ALTER TABLE unit_of_measure_conversions RENAME COLUMN to_uom TO to_unit_of_measure';
  END IF;
END$$;

-- 3b) Unique index name
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_indexes WHERE indexname = 'uom_conversions_uniq'
  ) THEN
    EXECUTE 'ALTER INDEX uom_conversions_uniq RENAME TO unit_of_measure_conversions_uniq';
  END IF;
END$$;

-- 4) Update foreign keys on ingredients/base_unit_of_measure to reference units_of_measure
-- Postgres keeps FKs across table rename; but ensure constraint names are updated for clarity
-- Optional: rename constraint names if needed (skip if unknown)