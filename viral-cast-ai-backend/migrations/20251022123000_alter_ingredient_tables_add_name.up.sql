-- Add name column to ingredient-related tables
ALTER TABLE ingredient_market_prices ADD COLUMN name VARCHAR(100);
ALTER TABLE ingredient_stock_moves ADD COLUMN name VARCHAR(100);
ALTER TABLE ingredient_stocks ADD COLUMN name VARCHAR(100);