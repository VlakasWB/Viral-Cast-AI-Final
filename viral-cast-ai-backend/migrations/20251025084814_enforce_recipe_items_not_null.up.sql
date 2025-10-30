-- Ensure recipe item foreign keys and quantity are always present
DELETE FROM recipe_items
WHERE recipe_sets_uuid IS NULL
   OR ingredient_stocks_uuid IS NULL
   OR quantity IS NULL;

ALTER TABLE recipe_items
    ALTER COLUMN recipe_sets_uuid SET NOT NULL,
    ALTER COLUMN ingredient_stocks_uuid SET NOT NULL,
    ALTER COLUMN quantity SET NOT NULL;
