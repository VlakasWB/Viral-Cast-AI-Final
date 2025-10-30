-- Add down migration script here
-- Revert minimal_stock column back to quantity
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'ingredient_catalog' AND column_name = 'minimal_stock'
    ) THEN
        ALTER TABLE ingredient_catalog
            RENAME COLUMN minimal_stock TO quantity;
    END IF;
END$$;
