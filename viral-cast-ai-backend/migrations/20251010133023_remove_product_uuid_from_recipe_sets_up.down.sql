-- Restore product_uuid column to recipe_sets table

-- Make name column nullable again
ALTER TABLE recipe_sets ALTER COLUMN name DROP NOT NULL;

-- Add back the product_uuid column
ALTER TABLE recipe_sets ADD COLUMN product_uuid UUID;

-- Add back the foreign key constraint
ALTER TABLE recipe_sets 
  ADD CONSTRAINT recipe_sets_product_uuid_fkey 
  FOREIGN KEY (product_uuid) REFERENCES products(uuid);