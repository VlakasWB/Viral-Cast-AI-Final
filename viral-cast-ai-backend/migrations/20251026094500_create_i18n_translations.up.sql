-- i18n translations tables based on dbml/schema_i18n.dbml
-- Requires pgcrypto extension for gen_random_uuid()

-- ingredient_catalog_translations
CREATE TABLE IF NOT EXISTS ingredient_catalog_translations (
  uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  ingredient_catalog_uuid UUID NOT NULL REFERENCES ingredient_catalog(uuid),
  locale VARCHAR(10) NOT NULL,
  name VARCHAR(100) NOT NULL,
  translation_status VARCHAR(20) DEFAULT 'human',
  created_at BIGINT,
  updated_at BIGINT,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT ingredient_catalog_translations_locale_chk CHECK (locale IN ('id','en')),
  CONSTRAINT ingredient_catalog_translations_status_chk CHECK (translation_status IN ('human','machine','pending'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uniq_ingredient_catalog_translations_locale
  ON ingredient_catalog_translations (ingredient_catalog_uuid, locale)
  WHERE deleted_at = 0;
CREATE INDEX IF NOT EXISTS ingredient_catalog_translations_catalog_idx
  ON ingredient_catalog_translations (ingredient_catalog_uuid);
CREATE INDEX IF NOT EXISTS ingredient_catalog_translations_locale_idx
  ON ingredient_catalog_translations (locale);


-- product_translations
CREATE TABLE IF NOT EXISTS product_translations (
  uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_uuid UUID NOT NULL REFERENCES products(uuid),
  locale VARCHAR(10) NOT NULL,
  name VARCHAR(150) NOT NULL,
  description TEXT,
  translation_status VARCHAR(20) DEFAULT 'human',
  created_at BIGINT,
  updated_at BIGINT,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT product_translations_locale_chk CHECK (locale IN ('id','en')),
  CONSTRAINT product_translations_status_chk CHECK (translation_status IN ('human','machine','pending'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uniq_product_translations_locale
  ON product_translations (product_uuid, locale)
  WHERE deleted_at = 0;
CREATE INDEX IF NOT EXISTS product_translations_product_idx
  ON product_translations (product_uuid);
CREATE INDEX IF NOT EXISTS product_translations_locale_idx
  ON product_translations (locale);


-- category_translations
CREATE TABLE IF NOT EXISTS category_translations (
  uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  category_uuid UUID NOT NULL REFERENCES categories(uuid),
  locale VARCHAR(10) NOT NULL,
  name VARCHAR(120) NOT NULL,
  translation_status VARCHAR(20) DEFAULT 'human',
  created_at BIGINT,
  updated_at BIGINT,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT category_translations_locale_chk CHECK (locale IN ('id','en')),
  CONSTRAINT category_translations_status_chk CHECK (translation_status IN ('human','machine','pending'))
);

CREATE INDEX IF NOT EXISTS category_translations_category_idx
  ON category_translations (category_uuid);
CREATE INDEX IF NOT EXISTS category_translations_locale_idx
  ON category_translations (locale);


-- recipe_set_translations
CREATE TABLE IF NOT EXISTS recipe_set_translations (
  uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  recipe_set_uuid UUID NOT NULL REFERENCES recipe_sets(uuid),
  locale VARCHAR(10) NOT NULL,
  name VARCHAR(150) NOT NULL,
  description TEXT,
  translation_status VARCHAR(20) DEFAULT 'human',
  created_at BIGINT,
  updated_at BIGINT,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT recipe_set_translations_locale_chk CHECK (locale IN ('id','en')),
  CONSTRAINT recipe_set_translations_status_chk CHECK (translation_status IN ('human','machine','pending'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uniq_recipe_set_translations_locale
  ON recipe_set_translations (recipe_set_uuid, locale)
  WHERE deleted_at = 0;
CREATE INDEX IF NOT EXISTS recipe_set_translations_set_idx
  ON recipe_set_translations (recipe_set_uuid);
CREATE INDEX IF NOT EXISTS recipe_set_translations_locale_idx
  ON recipe_set_translations (locale);


-- recipe_item_translations
CREATE TABLE IF NOT EXISTS recipe_item_translations (
  uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  recipe_item_uuid UUID NOT NULL REFERENCES recipe_items(uuid),
  locale VARCHAR(10) NOT NULL,
  name VARCHAR(150),
  instructions TEXT,
  notes TEXT,
  translation_status VARCHAR(20) DEFAULT 'human',
  created_at BIGINT,
  updated_at BIGINT,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT recipe_item_translations_locale_chk CHECK (locale IN ('id','en')),
  CONSTRAINT recipe_item_translations_status_chk CHECK (translation_status IN ('human','machine','pending'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uniq_recipe_item_translations_locale
  ON recipe_item_translations (recipe_item_uuid, locale)
  WHERE deleted_at = 0;
CREATE INDEX IF NOT EXISTS recipe_item_translations_item_idx
  ON recipe_item_translations (recipe_item_uuid);
CREATE INDEX IF NOT EXISTS recipe_item_translations_locale_idx
  ON recipe_item_translations (locale);


-- ingredient_stock_move_translations
CREATE TABLE IF NOT EXISTS ingredient_stock_move_translations (
  uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  ingredient_stock_move_uuid UUID NOT NULL REFERENCES ingredient_stock_moves(uuid),
  locale VARCHAR(10) NOT NULL,
  name VARCHAR(150),
  translation_status VARCHAR(20) DEFAULT 'human',
  created_at BIGINT,
  updated_at BIGINT,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT ingredient_stock_move_translations_locale_chk CHECK (locale IN ('id','en')),
  CONSTRAINT ingredient_stock_move_translations_status_chk CHECK (translation_status IN ('human','machine','pending'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uniq_ingredient_stock_move_translations_locale
  ON ingredient_stock_move_translations (ingredient_stock_move_uuid, locale)
  WHERE deleted_at = 0;
CREATE INDEX IF NOT EXISTS ingredient_stock_move_translations_move_idx
  ON ingredient_stock_move_translations (ingredient_stock_move_uuid);
CREATE INDEX IF NOT EXISTS ingredient_stock_move_translations_locale_idx
  ON ingredient_stock_move_translations (locale);


-- ingredient_stock_translations
CREATE TABLE IF NOT EXISTS ingredient_stock_translations (
  uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  ingredient_stock_uuid UUID NOT NULL REFERENCES ingredient_stocks(uuid),
  locale VARCHAR(10) NOT NULL,
  ingredient_name VARCHAR(150),
  translation_status VARCHAR(20) DEFAULT 'human',
  created_at BIGINT,
  updated_at BIGINT,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT ingredient_stock_translations_locale_chk CHECK (locale IN ('id','en')),
  CONSTRAINT ingredient_stock_translations_status_chk CHECK (translation_status IN ('human','machine','pending'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uniq_ingredient_stock_translations_locale
  ON ingredient_stock_translations (ingredient_stock_uuid, locale)
  WHERE deleted_at = 0;
CREATE INDEX IF NOT EXISTS ingredient_stock_translations_stock_idx
  ON ingredient_stock_translations (ingredient_stock_uuid);
CREATE INDEX IF NOT EXISTS ingredient_stock_translations_locale_idx
  ON ingredient_stock_translations (locale);


-- ingredient_market_price_translations
CREATE TABLE IF NOT EXISTS ingredient_market_price_translations (
  uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  ingredient_market_price_uuid UUID NOT NULL REFERENCES ingredient_market_prices(uuid),
  locale VARCHAR(10) NOT NULL,
  name VARCHAR(100),
  translation_status VARCHAR(20) DEFAULT 'human',
  created_at BIGINT,
  updated_at BIGINT,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT ingredient_market_price_translations_locale_chk CHECK (locale IN ('id','en')),
  CONSTRAINT ingredient_market_price_translations_status_chk CHECK (translation_status IN ('human','machine','pending'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uniq_ingredient_market_price_translations_locale
  ON ingredient_market_price_translations (ingredient_market_price_uuid, locale)
  WHERE deleted_at = 0;
CREATE INDEX IF NOT EXISTS ingredient_market_price_translations_market_price_idx
  ON ingredient_market_price_translations (ingredient_market_price_uuid);
CREATE INDEX IF NOT EXISTS ingredient_market_price_translations_locale_idx
  ON ingredient_market_price_translations (locale);