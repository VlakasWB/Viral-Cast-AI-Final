ALTER TABLE products
  ADD CONSTRAINT products_current_recipe_fk
  FOREIGN KEY (current_recipe_uuid)
  REFERENCES recipe_sets(uuid);
