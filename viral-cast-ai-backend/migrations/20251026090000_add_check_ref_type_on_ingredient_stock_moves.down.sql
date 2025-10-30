-- Drop CHECK constraint for ref_type
ALTER TABLE ingredient_stock_moves
DROP CONSTRAINT IF EXISTS ingredient_stock_moves_ref_type_allowed;