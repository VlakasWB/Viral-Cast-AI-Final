-- =============== PAYMENTS ===============
CREATE TABLE IF NOT EXISTS payments (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  order_uuid UUID NOT NULL REFERENCES orders(uuid) ON DELETE CASCADE,
  method VARCHAR(20) NOT NULL,
  amount NUMERIC(12,2) NOT NULL,
  paid_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  external_ref VARCHAR(100),
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT payments_method_valid CHECK (method IN ('CASH','CARD','QRIS','TRANSFER')),
  CONSTRAINT payments_amount_pos CHECK (amount >= 0)
);

CREATE INDEX IF NOT EXISTS payments_order_idx ON payments (order_uuid);