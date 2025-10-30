-- Recreate stock_moves and sales_daily tables (rollback)

-- stock_moves
CREATE TABLE IF NOT EXISTS stock_moves (
  uuid UUID DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
  ingredient_uuid UUID NOT NULL REFERENCES ingredients(uuid),
  move VARCHAR(20) NOT NULL,
  qty NUMERIC(12,3) NOT NULL,
  unit_cost NUMERIC(12,4),
  note VARCHAR(255),
  ref_order_uuid UUID,
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0,

  CONSTRAINT stock_moves_move_valid CHECK (move IN ('IN','OUT','ADJUST_IN','ADJUST_OUT')),
  CONSTRAINT stock_moves_qty_pos CHECK (qty > 0),
  CONSTRAINT stock_moves_unit_cost_when_in CHECK (
    CASE WHEN move IN ('IN','ADJUST_IN') THEN unit_cost IS NOT NULL ELSE TRUE END
  )
);

CREATE INDEX IF NOT EXISTS stock_moves_ingredient_created_idx
  ON stock_moves (ingredient_uuid, created_at);

-- sales_daily
CREATE TABLE IF NOT EXISTS sales_daily (
  uuid UUID DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
  product_uuid UUID NOT NULL REFERENCES products(uuid),
  date_ts BIGINT NOT NULL,
  qty NUMERIC(12,4) NOT NULL DEFAULT 0,
  revenue NUMERIC(12,2) NOT NULL DEFAULT 0,
  cost NUMERIC(12,2) NOT NULL DEFAULT 0,
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS sales_daily_unique_active
  ON sales_daily (product_uuid, date_ts)
  WHERE deleted_at = 0;