-- Add down migration script here
-- Remove unit_of_measure_code and unit_of_measure_name from ingredient_market_prices
ALTER TABLE IF EXISTS ingredient_market_prices
    DROP COLUMN IF EXISTS unit_of_measure_code,
    DROP COLUMN IF EXISTS unit_of_measure_name;

-- Remove unit_of_measure_code and unit_of_measure_name from ingredient_stock_moves
ALTER TABLE IF EXISTS ingredient_stock_moves
    DROP COLUMN IF EXISTS unit_of_measure_code,
    DROP COLUMN IF EXISTS unit_of_measure_name;

-- Remove unit_of_measure_code and unit_of_measure_name from ingredient_stocks
ALTER TABLE IF EXISTS ingredient_stocks
    DROP COLUMN IF EXISTS unit_of_measure_code,
    DROP COLUMN IF EXISTS unit_of_measure_name;