-- Add shelf_life_days to ingredient_catalog
ALTER TABLE ingredient_catalog ADD COLUMN IF NOT EXISTS shelf_life_days INT;