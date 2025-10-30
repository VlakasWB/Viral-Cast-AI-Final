-- Add down migration script here

-- Drop indexes first
DROP INDEX IF EXISTS users_username_uniq;
DROP INDEX IF EXISTS users_email_uniq;
DROP INDEX IF EXISTS profiles_user_uuid_uniq;
DROP INDEX IF EXISTS ingredient_prices_uniq;
DROP INDEX IF EXISTS stock_moves_ingredient_created_idx;
DROP INDEX IF EXISTS uom_conversions_uniq;
DROP INDEX IF EXISTS recipe_items_unique_active;
DROP INDEX IF EXISTS recipe_items_ingredient_idx;
DROP INDEX IF EXISTS recipe_sets_active_uniq;
DROP INDEX IF EXISTS recipe_sets_prod_efffrom_idx;
DROP INDEX IF EXISTS products_sku_active_uniq;
DROP INDEX IF EXISTS products_category_idx;

-- Drop tables in reverse order of dependencies
-- Tables with foreign keys must be dropped first
DROP TABLE IF EXISTS recipe_items;
DROP TABLE IF EXISTS recipe_sets;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS uom_conversions;
DROP TABLE IF EXISTS stock_moves;
DROP TABLE IF EXISTS ingredient_prices;
DROP TABLE IF EXISTS ingredients;
DROP TABLE IF EXISTS uoms;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS profiles;
DROP TABLE IF EXISTS roles;
DROP TABLE IF EXISTS users;

-- Drop extension if needed (optional, as it might be used by other databases)
-- DROP EXTENSION IF EXISTS "uuid-ossp";
