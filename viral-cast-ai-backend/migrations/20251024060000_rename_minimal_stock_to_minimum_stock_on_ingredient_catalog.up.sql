DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'ingredient_catalog'
          AND column_name = 'minimal_stock'
    ) THEN
        EXECUTE 'ALTER TABLE ingredient_catalog RENAME COLUMN minimal_stock TO minimum_stock';
    END IF;
END
$$;
