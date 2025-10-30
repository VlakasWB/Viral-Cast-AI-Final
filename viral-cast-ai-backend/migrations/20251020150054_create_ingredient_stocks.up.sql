-- Create ingredient_stocks table
CREATE TABLE IF NOT EXISTS ingredient_stocks (
    uuid UUID PRIMARY KEY,
    ingredient_stock_moves_uuid UUID NOT NULL,
    total_quantity NUMERIC(12,4) NOT NULL DEFAULT 0,
    total_value NUMERIC(14,4) NOT NULL DEFAULT 0,
    current_cost NUMERIC(12,4),
    avg_cost NUMERIC(12,4),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    deleted_at BIGINT NOT NULL DEFAULT 0,
    FOREIGN KEY (ingredient_stock_moves_uuid) REFERENCES ingredient_stock_moves(uuid)
);

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS ingredient_stocks_ingredient_stock_moves_uuid_idx ON ingredient_stocks(ingredient_stock_moves_uuid);
