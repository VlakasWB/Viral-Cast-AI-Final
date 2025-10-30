DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'ingredient_stocks'
          AND column_name = 'ingredient_stock_moves_uuid'
    ) THEN
        ALTER TABLE ingredient_stocks
            RENAME COLUMN ingredient_stock_moves_uuid TO ingredient_uuid;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'ingredient_stocks_ingredient_stock_moves_uuid_idx'
    ) THEN
        ALTER INDEX ingredient_stocks_ingredient_stock_moves_uuid_idx
            RENAME TO ingredient_stocks_ingredient_uuid_idx;
    END IF;
END
$$;
