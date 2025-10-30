-- Step 1: Restore original column names in recipe_items table
ALTER TABLE recipe_items 
ADD COLUMN recipe_uuid UUID,
    ADD COLUMN ingredient_uuid UUID,
    ADD COLUMN qty DECIMAL(10,2),
    ADD COLUMN waste_pct DECIMAL(5,4);

UPDATE recipe_items 
SET recipe_uuid = recipe_sets_uuid,
    qty = quantity,
    waste_pct = waste_percent;

-- Map ingredient_stocks_uuid back to ingredient_catalog_uuid for restored ingredient_uuid
UPDATE recipe_items ri
SET ingredient_uuid = ism.ingredient_uuid
FROM ingredient_stocks ism
WHERE ri.ingredient_stocks_uuid = ism.uuid;

ALTER TABLE recipe_items 
DROP COLUMN recipe_sets_uuid,
     DROP COLUMN ingredient_stocks_uuid,
     DROP COLUMN quantity,
     DROP COLUMN waste_percent;

-- Step 2: Restore original column name in recipe_sets table
ALTER TABLE recipe_sets ADD yield_qty DECIMAL(10,2);
UPDATE recipe_sets SET yield_qty = yield_quantity;
ALTER TABLE recipe_sets DROP COLUMN yield_quantity;

-- Step 3: Restore foreign key references in recipe_items (now that columns exist)
ALTER TABLE recipe_items DROP CONSTRAINT IF EXISTS recipe_items_ingredient_stocks_uuid_fkey;
ALTER TABLE recipe_items ADD CONSTRAINT recipe_items_ingredient_uuid_fkey 
  FOREIGN KEY (ingredient_uuid) REFERENCES ingredient_catalog(uuid) NOT VALID;

ALTER TABLE recipe_items DROP CONSTRAINT IF EXISTS recipe_items_recipe_sets_uuid_fkey;
ALTER TABLE recipe_items ADD CONSTRAINT recipe_items_recipe_uuid_fkey 
  FOREIGN KEY (recipe_uuid) REFERENCES recipe_sets(uuid) NOT VALID;

-- Step 4: Restore constraint names to original
ALTER TABLE recipe_items DROP CONSTRAINT IF EXISTS recipe_items_waste_percent_range;
ALTER TABLE recipe_items ADD CONSTRAINT recipe_items_waste_range CHECK (waste_pct >= 0 AND waste_pct <= 1);

ALTER TABLE recipe_items DROP CONSTRAINT IF EXISTS recipe_items_quantity_pos;
ALTER TABLE recipe_items ADD CONSTRAINT recipe_items_qty_pos CHECK (qty > 0);

ALTER TABLE recipe_sets DROP CONSTRAINT IF EXISTS recipe_sets_yield_quantity_pos;
ALTER TABLE recipe_sets ADD CONSTRAINT recipe_sets_yield_pos CHECK (yield_qty > 0);

-- Step 5: Restore indexes to original names (after columns restored)
DROP INDEX IF EXISTS recipe_items_ingredient_stocks_idx;
CREATE INDEX recipe_items_ingredient_idx ON recipe_items (ingredient_uuid);

DROP INDEX IF EXISTS recipe_items_unique_active;
CREATE UNIQUE INDEX recipe_items_unique_active ON recipe_items (recipe_uuid, ingredient_uuid) WHERE deleted_at = 0;
