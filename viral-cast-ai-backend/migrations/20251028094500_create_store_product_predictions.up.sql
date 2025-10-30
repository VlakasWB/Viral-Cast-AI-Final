CREATE TABLE IF NOT EXISTS store_product_predictions (
  uuid UUID DEFAULT gen_uuid_v7() PRIMARY KEY,
  store_uuid UUID NOT NULL REFERENCES stores(uuid),
  product_uuid UUID NOT NULL REFERENCES products(uuid),
  region_code TEXT,
  target_date DATE NOT NULL,
  demand_label VARCHAR(32) NOT NULL,
  demand_probability REAL,
  recommended_stock_qty NUMERIC(12,2) NOT NULL DEFAULT 0,
  weather_summary TEXT,
  weather_temp_min_c REAL,
  weather_temp_max_c REAL,
  weather_precip_mm REAL,
  weather_humidity REAL,
  llm_reasoning TEXT,
  llm_model VARCHAR(128),
  llm_prompt JSONB,
  llm_response JSONB,
  created_at BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW()) * 1000)::bigint,
  updated_at BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW()) * 1000)::bigint,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT store_product_predictions_unique UNIQUE (store_uuid, product_uuid, target_date),
  CONSTRAINT store_product_predictions_stock_nonneg CHECK (recommended_stock_qty >= 0),
  CONSTRAINT store_product_predictions_prob_range CHECK (
    demand_probability IS NULL OR (demand_probability >= 0 AND demand_probability <= 1)
  )
);

CREATE INDEX IF NOT EXISTS idx_store_product_predictions_store_date
  ON store_product_predictions (store_uuid, target_date)
  WHERE deleted_at = 0;

CREATE INDEX IF NOT EXISTS idx_store_product_predictions_product_date
  ON store_product_predictions (product_uuid, target_date)
  WHERE deleted_at = 0;
