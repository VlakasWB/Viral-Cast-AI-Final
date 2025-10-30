-- Add CHECK constraint to enforce allowed ref_type values
-- Allows NULL, otherwise only PURCHASE | PRODUCTION | ADJUSTMENT | WASTE | RETURN (case-insensitive)
ALTER TABLE ingredient_stock_moves
ADD CONSTRAINT ingredient_stock_moves_ref_type_allowed
CHECK (
  ref_type IS NULL
  OR UPPER(ref_type) IN ('PURCHASE','PRODUCTION','ADJUSTMENT','WASTE','RETURN')
);