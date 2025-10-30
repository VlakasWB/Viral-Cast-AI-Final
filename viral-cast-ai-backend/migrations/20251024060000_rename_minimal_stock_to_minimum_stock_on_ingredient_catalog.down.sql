DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'ingredient_catalog'
          AND column_name = 'minimum_stock'
    ) THEN
        EXECUTE 'ALTER TABLE ingredient_catalog RENAME COLUMN minimum_stock TO minimal_stock';
    END IF;
END
$$;
