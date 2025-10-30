-- Drop existing foreign key constraint
ALTER TABLE products
  DROP CONSTRAINT IF EXISTS products_recipe_sets_fk;

-- Rename column back to original name
ALTER TABLE products
  RENAME COLUMN recipe_sets_uuid TO current_recipe_uuid;

-- Add back original foreign key constraint
ALTER TABLE products
  ADD CONSTRAINT products_current_recipe_fk
  FOREIGN KEY (current_recipe_uuid)
  REFERENCES recipe_sets(uuid);