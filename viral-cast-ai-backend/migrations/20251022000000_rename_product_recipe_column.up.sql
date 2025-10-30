-- Drop existing foreign key constraint
ALTER TABLE products
  DROP CONSTRAINT IF EXISTS products_current_recipe_fk;

-- Rename column
ALTER TABLE products
  RENAME COLUMN current_recipe_uuid TO recipe_sets_uuid;

-- Add new foreign key constraint
ALTER TABLE products
  ADD CONSTRAINT products_recipe_sets_fk
  FOREIGN KEY (recipe_sets_uuid)
  REFERENCES recipe_sets(uuid);