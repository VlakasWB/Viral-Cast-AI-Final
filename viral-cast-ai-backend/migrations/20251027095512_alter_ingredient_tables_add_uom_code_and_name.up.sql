-- Add up migration script here
-- Add unit_of_measure_code and unit_of_measure_name to ingredient_market_prices
ALTER TABLE IF EXISTS ingredient_market_prices
    ADD COLUMN IF NOT EXISTS unit_of_measure_code VARCHAR(50),
    ADD COLUMN IF NOT EXISTS unit_of_measure_name VARCHAR(100);

-- Add unit_of_measure_code and unit_of_measure_name to ingredient_stock_moves
ALTER TABLE IF EXISTS ingredient_stock_moves
    ADD COLUMN IF NOT EXISTS unit_of_measure_code VARCHAR(50),
    ADD COLUMN IF NOT EXISTS unit_of_measure_name VARCHAR(100);

-- Add unit_of_measure_code and unit_of_measure_name to ingredient_stocks
ALTER TABLE IF EXISTS ingredient_stocks
    ADD COLUMN IF NOT EXISTS unit_of_measure_code VARCHAR(50),
    ADD COLUMN IF NOT EXISTS unit_of_measure_name VARCHAR(100);

-- Backfill existing records with current unit of measure snapshots
UPDATE ingredient_market_prices imp
SET unit_of_measure_code = COALESCE(imp.unit_of_measure_code, uom.code),
    unit_of_measure_name = COALESCE(imp.unit_of_measure_name, uom.name)
FROM ingredient_catalog ic
LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
WHERE imp.ingredient_catalog_uuid = ic.uuid
  AND (imp.unit_of_measure_code IS NULL OR imp.unit_of_measure_name IS NULL);

UPDATE ingredient_stock_moves m
SET unit_of_measure_code = COALESCE(
        m.unit_of_measure_code,
        COALESCE(uom.code, imp.unit_of_measure_code)
    ),
    unit_of_measure_name = COALESCE(
        m.unit_of_measure_name,
        COALESCE(uom.name, imp.unit_of_measure_name)
    )
FROM ingredient_catalog ic
LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
LEFT JOIN ingredient_market_prices imp
    ON imp.ingredient_catalog_uuid = ic.uuid
   AND imp.deleted_at = 0
WHERE m.ingredient_catalog_uuid = ic.uuid
  AND (m.unit_of_measure_code IS NULL OR m.unit_of_measure_name IS NULL);

UPDATE ingredient_stocks s
SET unit_of_measure_code = COALESCE(
        s.unit_of_measure_code,
        COALESCE(m.unit_of_measure_code, uom.code)
    ),
    unit_of_measure_name = COALESCE(
        s.unit_of_measure_name,
        COALESCE(m.unit_of_measure_name, uom.name)
    )
FROM ingredient_stock_moves m
LEFT JOIN ingredient_catalog ic ON m.ingredient_catalog_uuid = ic.uuid
LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
WHERE s.ingredient_stock_moves_uuid = m.uuid
  AND (s.unit_of_measure_code IS NULL OR s.unit_of_measure_name IS NULL);
