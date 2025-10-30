
DROP INDEX IF EXISTS order_items_prod_created_idx;
DROP INDEX IF EXISTS order_items_order_idx;

DROP INDEX IF EXISTS orders_created_idx;
DROP INDEX IF EXISTS orders_status_idx;

DROP TABLE IF EXISTS order_items;
DROP TABLE IF EXISTS orders;