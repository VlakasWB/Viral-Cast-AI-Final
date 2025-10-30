-- Membuat tabel ingredient_market_prices untuk menyimpan harga pasar bahan
CREATE TABLE IF NOT EXISTS ingredient_market_prices (
   uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
   ingredient_catalog_uuid UUID NOT NULL,
   price NUMERIC(12,4),
   effective_at BIGINT NOT NULL,
   created_at BIGINT,
   updated_at BIGINT,
   deleted_at BIGINT DEFAULT 0
);

-- Menambahkan foreign key constraint ke tabel ingredient_catalog
ALTER TABLE ingredient_market_prices
   ADD CONSTRAINT fk_ingredient_market_prices_ingredient_catalog
   FOREIGN KEY (ingredient_catalog_uuid)
   REFERENCES ingredient_catalog(uuid);

-- Menambahkan indeks untuk pencarian yang lebih cepat
CREATE INDEX idx_ingredient_market_prices_ingredient_catalog_uuid ON ingredient_market_prices(ingredient_catalog_uuid);
CREATE INDEX idx_ingredient_market_prices_effective_at ON ingredient_market_prices(effective_at);