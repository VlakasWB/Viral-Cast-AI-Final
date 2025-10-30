-- =============== SALES_DAILY (AGREGAT) ===============
CREATE TABLE IF NOT EXISTS sales_daily (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  product_uuid UUID NOT NULL REFERENCES products(uuid),
  date_ts BIGINT NOT NULL,
  qty NUMERIC(12,4) NOT NULL DEFAULT 0,
  revenue NUMERIC(12,2) NOT NULL DEFAULT 0,
  cost NUMERIC(12,2) NOT NULL DEFAULT 0,
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0
);

-- unik per (produk, hari) hanya untuk baris aktif
CREATE UNIQUE INDEX IF NOT EXISTS sales_daily_unique_active
  ON sales_daily (product_uuid, date_ts)
  WHERE deleted_at = 0;