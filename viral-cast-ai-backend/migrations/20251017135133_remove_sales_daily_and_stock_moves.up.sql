-- Drop stock_moves and sales_daily tables and related indexes

-- stock_moves
DROP INDEX IF EXISTS stock_moves_ingredient_created_idx;
DROP TABLE IF EXISTS stock_moves;

-- sales_daily
DROP INDEX IF EXISTS sales_daily_unique_active;
DROP TABLE IF EXISTS sales_daily;