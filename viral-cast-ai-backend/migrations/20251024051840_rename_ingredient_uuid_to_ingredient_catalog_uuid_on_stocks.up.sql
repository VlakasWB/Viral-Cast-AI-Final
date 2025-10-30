DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'ingredient_stocks'
          AND column_name = 'ingredient_uuid'
    ) THEN
        ALTER TABLE ingredient_stocks
            RENAME COLUMN ingredient_uuid TO ingredient_stock_moves_uuid;
    ELSIF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'ingredient_stocks'
          AND column_name = 'ingredient_catalog_uuid'
    ) THEN
        ALTER TABLE ingredient_stocks
            RENAME COLUMN ingredient_catalog_uuid TO ingredient_stock_moves_uuid;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'ingredient_stocks_ingredient_uuid_idx'
    ) THEN
        ALTER INDEX ingredient_stocks_ingredient_uuid_idx
            RENAME TO ingredient_stocks_ingredient_stock_moves_uuid_idx;
    ELSIF EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'ingredient_stocks_ingredient_catalog_uuid_idx'
    ) THEN
        ALTER INDEX ingredient_stocks_ingredient_catalog_uuid_idx
            RENAME TO ingredient_stocks_ingredient_stock_moves_uuid_idx;
    END IF;
END
$$;
