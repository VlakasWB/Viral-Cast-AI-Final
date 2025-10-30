-- Remove product_uuid column from recipe_sets table
-- Recipes are now independent entities that can be assigned to products

-- First, remove the foreign key constraint if it exists
ALTER TABLE recipe_sets DROP CONSTRAINT IF EXISTS recipe_sets_product_uuid_fkey;

-- Remove the product_uuid column
ALTER TABLE recipe_sets DROP COLUMN IF EXISTS product_uuid;

-- Make name column NOT NULL since recipes should have names
ALTER TABLE recipe_sets ALTER COLUMN name SET NOT NULL;