-- Drop unit_of_measure_conversions table and legacy uom_conversions if present
-- This removes unused UOM conversion functionality from the schema

-- Drop current name (post-rename)
DROP TABLE IF EXISTS unit_of_measure_conversions;

-- Drop legacy name (pre-rename)
DROP TABLE IF EXISTS uom_conversions;