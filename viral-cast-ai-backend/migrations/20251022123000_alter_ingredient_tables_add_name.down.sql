-- Remove name column from ingredient-related tables
ALTER TABLE ingredient_market_prices DROP COLUMN name;
ALTER TABLE ingredient_stock_moves DROP COLUMN name;
ALTER TABLE ingredient_stocks DROP COLUMN name;