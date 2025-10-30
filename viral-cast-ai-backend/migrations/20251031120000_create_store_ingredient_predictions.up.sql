CREATE TABLE IF NOT EXISTS store_ingredient_predictions (
  uuid UUID DEFAULT gen_uuid_v7() PRIMARY KEY,
  store_uuid UUID NOT NULL REFERENCES stores(uuid),
  ingredient_catalog_uuid UUID NOT NULL REFERENCES ingredient_catalog(uuid),
  region_code TEXT,
  restock_label VARCHAR(32) NOT NULL,
  restock_probability REAL,
  recommended_restock_qty NUMERIC(14,4) NOT NULL DEFAULT 0,
  current_stock_qty NUMERIC(14,4),
  minimum_stock_qty NUMERIC(14,4),
  unit_of_measure_code VARCHAR(32),
  unit_of_measure_name VARCHAR(64),
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
  CONSTRAINT store_ingredient_predictions_unique UNIQUE (store_uuid, ingredient_catalog_uuid),
  CONSTRAINT store_ingredient_predictions_qty_nonneg CHECK (recommended_restock_qty >= 0),
  CONSTRAINT store_ingredient_predictions_prob_range CHECK (
    restock_probability IS NULL OR (restock_probability >= 0 AND restock_probability <= 1)
  )
);

CREATE INDEX IF NOT EXISTS idx_store_ingredient_predictions_store
  ON store_ingredient_predictions (store_uuid)
  WHERE deleted_at = 0;

CREATE INDEX IF NOT EXISTS idx_store_ingredient_predictions_ingredient
  ON store_ingredient_predictions (ingredient_catalog_uuid)
  WHERE deleted_at = 0;
