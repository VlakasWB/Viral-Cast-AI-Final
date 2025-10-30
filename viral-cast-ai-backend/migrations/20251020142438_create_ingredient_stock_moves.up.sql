CREATE TABLE IF NOT EXISTS ingredient_stock_moves (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ingredient_catalog_uuid UUID NOT NULL,
    quantity NUMERIC(12,4) NOT NULL,
    price NUMERIC(12,4),
    price_updated_at BIGINT,
    effective_at BIGINT NOT NULL,
    expiry_at BIGINT,
    ref_type VARCHAR(30),
    ref_uuid UUID,
    created_at BIGINT,
    updated_at BIGINT,
    deleted_at BIGINT DEFAULT 0,
    CONSTRAINT fk_ingredient_stock_moves_ingredient_catalog
        FOREIGN KEY (ingredient_catalog_uuid)
        REFERENCES ingredient_catalog (uuid)
        ON DELETE CASCADE
);

CREATE INDEX idx_ingredient_stock_moves_ingredient_catalog_uuid ON ingredient_stock_moves (ingredient_catalog_uuid);
CREATE INDEX idx_ingredient_stock_moves_effective_at ON ingredient_stock_moves (effective_at);
CREATE INDEX idx_ingredient_stock_moves_expiry_at ON ingredient_stock_moves (expiry_at);
CREATE INDEX idx_ingredient_stock_moves_ref_type ON ingredient_stock_moves (ref_type);
CREATE INDEX idx_ingredient_stock_moves_deleted_at ON ingredient_stock_moves (deleted_at);