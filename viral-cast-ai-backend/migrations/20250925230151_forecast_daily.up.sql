CREATE TABLE IF NOT EXISTS forecast_daily (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  product_uuid UUID NOT NULL REFERENCES products(uuid),
  date_ts BIGINT NOT NULL,
  method VARCHAR(32) NOT NULL,
  window_size INT,                                 -- <-- was: window INT
  params JSON,
  forecast_qty NUMERIC(12,4) NOT NULL,
  conf_low NUMERIC(12,4),
  conf_high NUMERIC(12,4),
  mae NUMERIC(12,4),
  mape NUMERIC(12,4),
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT forecast_daily_method_valid CHECK (
    method IN ('SMA','WMA','EMA','LINEAR_REGRESSION','MULTIVARIATE_REGRESSION')
  ),
  CONSTRAINT forecast_daily_window_pos CHECK (window_size IS NULL OR window_size > 0)
);
