-- Revert rename of UOM to Units of Measure

-- 1) units_of_measure -> uoms
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.tables WHERE table_name = 'units_of_measure'
  ) THEN
    EXECUTE 'ALTER TABLE units_of_measure RENAME TO uoms';
  END IF;
END$$;

-- 2) ingredients.base_unit_of_measure -> ingredients.base_uom
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'ingredients' AND column_name = 'base_unit_of_measure'
  ) THEN
    EXECUTE 'ALTER TABLE ingredients RENAME COLUMN base_unit_of_measure TO base_uom';
  END IF;
END$$;

-- 3) unit_of_measure_conversions -> uom_conversions
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.tables WHERE table_name = 'unit_of_measure_conversions'
  ) THEN
    EXECUTE 'ALTER TABLE unit_of_measure_conversions RENAME TO uom_conversions';
  END IF;
END$$;

-- 3a) Columns: from_unit_of_measure -> from_uom; to_unit_of_measure -> to_uom
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'uom_conversions' AND column_name = 'from_unit_of_measure'
  ) THEN
    EXECUTE 'ALTER TABLE uom_conversions RENAME COLUMN from_unit_of_measure TO from_uom';
  END IF;
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'uom_conversions' AND column_name = 'to_unit_of_measure'
  ) THEN
    EXECUTE 'ALTER TABLE uom_conversions RENAME COLUMN to_unit_of_measure TO to_uom';
  END IF;
END$$;

-- 3b) Unique index name
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_indexes WHERE indexname = 'unit_of_measure_conversions_uniq'
  ) THEN
    EXECUTE 'ALTER INDEX unit_of_measure_conversions_uniq RENAME TO uom_conversions_uniq';
  END IF;
END$$;