-- Add up migration script here
-- Rename quantity column to minimal_stock and adjust related constraints/indexes if any
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'ingredient_catalog' AND column_name = 'quantity'
    ) THEN
        ALTER TABLE ingredient_catalog
            RENAME COLUMN quantity TO minimal_stock;
    END IF;
END$$;
