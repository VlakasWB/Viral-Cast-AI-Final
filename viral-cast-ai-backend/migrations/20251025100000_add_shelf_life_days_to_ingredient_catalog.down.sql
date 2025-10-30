-- Remove shelf_life_days from ingredient_catalog
ALTER TABLE ingredient_catalog DROP COLUMN IF EXISTS shelf_life_days;