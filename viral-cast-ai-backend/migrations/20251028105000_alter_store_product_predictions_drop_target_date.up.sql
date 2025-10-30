ALTER TABLE store_product_predictions
  DROP CONSTRAINT IF EXISTS store_product_predictions_unique;

DROP INDEX IF EXISTS idx_store_product_predictions_product_date;
DROP INDEX IF EXISTS idx_store_product_predictions_store_date;

ALTER TABLE store_product_predictions
  DROP COLUMN IF EXISTS target_date;

ALTER TABLE store_product_predictions
  ADD CONSTRAINT store_product_predictions_unique_per_product
    UNIQUE (store_uuid, product_uuid);

CREATE INDEX IF NOT EXISTS idx_store_product_predictions_store
  ON store_product_predictions (store_uuid)
  WHERE deleted_at = 0;

CREATE INDEX IF NOT EXISTS idx_store_product_predictions_product
  ON store_product_predictions (product_uuid)
  WHERE deleted_at = 0;
