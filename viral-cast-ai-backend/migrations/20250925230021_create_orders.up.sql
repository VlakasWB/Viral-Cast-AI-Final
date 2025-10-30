-- =============== ORDERS =================
CREATE TABLE IF NOT EXISTS orders (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  order_no VARCHAR(30) NOT NULL UNIQUE,
  cashier_uuid UUID,
  status VARCHAR(20) NOT NULL DEFAULT 'PAID',
  subtotal NUMERIC(12,2) NOT NULL DEFAULT 0,
  discount NUMERIC(12,2) NOT NULL DEFAULT 0,
  tax NUMERIC(12,2) NOT NULL DEFAULT 0,
  total NUMERIC(12,2) NOT NULL DEFAULT 0,
  net_profit NUMERIC(12,2),
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT orders_status_valid CHECK (status IN ('DRAFT','PAID','CANCELLED','REFUNDED'))
);

CREATE INDEX IF NOT EXISTS orders_created_idx ON orders (created_at);
CREATE INDEX IF NOT EXISTS orders_status_idx  ON orders (status);

-- =============== ORDER ITEMS ===============
CREATE TABLE IF NOT EXISTS order_items (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  order_uuid UUID NOT NULL REFERENCES orders(uuid) ON DELETE CASCADE,
  product_uuid UUID NOT NULL REFERENCES products(uuid),
  qty NUMERIC(12,4) NOT NULL,
  unit_price NUMERIC(12,2) NOT NULL,
  unit_cost NUMERIC(12,2),
  line_total NUMERIC(12,2) NOT NULL,
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT order_items_qty_pos CHECK (qty > 0)
);

CREATE INDEX IF NOT EXISTS order_items_prod_created_idx
  ON order_items (product_uuid, created_at);
CREATE INDEX IF NOT EXISTS order_items_order_idx
  ON order_items (order_uuid);