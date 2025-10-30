ALTER TABLE recipe_items
    ALTER COLUMN recipe_sets_uuid DROP NOT NULL,
    ALTER COLUMN ingredient_stocks_uuid DROP NOT NULL,
    ALTER COLUMN quantity DROP NOT NULL;
