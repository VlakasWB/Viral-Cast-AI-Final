ALTER TABLE store_product_predictions
  DROP CONSTRAINT IF EXISTS store_product_predictions_unique_per_product;

DROP INDEX IF EXISTS idx_store_product_predictions_product;
DROP INDEX IF EXISTS idx_store_product_predictions_store;

ALTER TABLE store_product_predictions
  ADD COLUMN IF NOT EXISTS target_date DATE;

UPDATE store_product_predictions
SET target_date = (
    to_timestamp(created_at / 1000)::date + INTERVAL '1 day'
  )::date
WHERE target_date IS NULL;

ALTER TABLE store_product_predictions
  ALTER COLUMN target_date SET NOT NULL;

ALTER TABLE store_product_predictions
  ADD CONSTRAINT store_product_predictions_unique
    UNIQUE (store_uuid, product_uuid, target_date);

CREATE INDEX IF NOT EXISTS idx_store_product_predictions_store_date
  ON store_product_predictions (store_uuid, target_date)
  WHERE deleted_at = 0;

CREATE INDEX IF NOT EXISTS idx_store_product_predictions_product_date
  ON store_product_predictions (product_uuid, target_date)
  WHERE deleted_at = 0;
