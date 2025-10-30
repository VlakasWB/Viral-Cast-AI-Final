-- Rename columns in recipe_sets table
ALTER TABLE recipe_sets 
ADD yield_quantity DECIMAL(10,2);

UPDATE recipe_sets 
SET yield_quantity = yield_qty;

ALTER TABLE recipe_sets 
DROP COLUMN yield_qty;

-- Rename columns in recipe_items table
ALTER TABLE recipe_items 
ADD COLUMN recipe_sets_uuid UUID,
    ADD COLUMN ingredient_stocks_uuid UUID,
    ADD COLUMN quantity DECIMAL(10,2),
    ADD COLUMN waste_percent DECIMAL(5,4);

UPDATE recipe_items 
SET recipe_sets_uuid = recipe_uuid,
    ingredient_stocks_uuid = ingredient_uuid,
    quantity = qty,
    waste_percent = waste_pct;

ALTER TABLE recipe_items 
DROP COLUMN recipe_uuid,
     DROP COLUMN ingredient_uuid,
     DROP COLUMN qty,
     DROP COLUMN waste_pct;

-- Update foreign key reference in recipe_items
ALTER TABLE recipe_items DROP CONSTRAINT IF EXISTS recipe_items_ingredient_uuid_fkey;
ALTER TABLE recipe_items ADD CONSTRAINT recipe_items_ingredient_stocks_uuid_fkey 
  FOREIGN KEY (ingredient_stocks_uuid) REFERENCES ingredient_stocks(uuid);

ALTER TABLE recipe_items DROP CONSTRAINT IF EXISTS recipe_items_recipe_uuid_fkey;
ALTER TABLE recipe_items ADD CONSTRAINT recipe_items_recipe_sets_uuid_fkey 
  FOREIGN KEY (recipe_sets_uuid) REFERENCES recipe_sets(uuid);

-- Update constraint names to match new column names
ALTER TABLE recipe_sets DROP CONSTRAINT IF EXISTS recipe_sets_yield_pos;
ALTER TABLE recipe_sets ADD CONSTRAINT recipe_sets_yield_quantity_pos CHECK (yield_quantity > 0);

ALTER TABLE recipe_items DROP CONSTRAINT IF EXISTS recipe_items_qty_pos;
ALTER TABLE recipe_items ADD CONSTRAINT recipe_items_quantity_pos CHECK (quantity > 0);

ALTER TABLE recipe_items DROP CONSTRAINT IF EXISTS recipe_items_waste_range;
ALTER TABLE recipe_items ADD CONSTRAINT recipe_items_waste_percent_range CHECK (waste_percent >= 0 AND waste_percent <= 1);

-- Update indexes to use new column names
DROP INDEX IF EXISTS recipe_items_unique_active;
CREATE UNIQUE INDEX recipe_items_unique_active ON recipe_items (recipe_sets_uuid, ingredient_stocks_uuid) WHERE deleted_at = 0;

DROP INDEX IF EXISTS recipe_items_ingredient_idx;
CREATE INDEX recipe_items_ingredient_stocks_idx ON recipe_items (ingredient_stocks_uuid);
