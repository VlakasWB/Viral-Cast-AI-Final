-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Helper function to generate time-ordered UUID v7
CREATE OR REPLACE FUNCTION gen_uuid_v7() RETURNS uuid AS $$
DECLARE
    unix_ts_ms BIGINT;
    ts_bytes BYTEA;
    rand_bytes BYTEA;
    uuid_bytes BYTEA;
BEGIN
    unix_ts_ms := (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::BIGINT;
    ts_bytes := decode(lpad(to_hex(unix_ts_ms), 12, '0'), 'hex');
    rand_bytes := gen_random_bytes(10);
    uuid_bytes := ts_bytes || rand_bytes;

    -- Set version to 7
    uuid_bytes := set_byte(uuid_bytes, 6, (get_byte(uuid_bytes, 6) & 0x0F) | 0x70);
    -- Set variant to RFC 4122
    uuid_bytes := set_byte(uuid_bytes, 8, (get_byte(uuid_bytes, 8) & 0x3F) | 0x80);

    RETURN encode(uuid_bytes, 'hex')::uuid;
END;
$$ LANGUAGE plpgsql;

-- Users
CREATE TABLE
    IF NOT EXISTS users (
        uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
        username VARCHAR(100) NOT NULL UNIQUE,
        email VARCHAR(255) UNIQUE,
        password VARCHAR(100) NOT NULL,
        access_token TEXT,
        refresh_token TEXT,
        created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        deleted_at BIGINT DEFAULT 0
    );

-- Roles
CREATE TABLE
    IF NOT EXISTS roles (
        uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
        number SERIAL UNIQUE,
        name VARCHAR(100),
        created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        deleted_at BIGINT DEFAULT 0
    );

--profiles
CREATE TABLE
    IF NOT EXISTS profiles (
        uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
        user_uuid UUID REFERENCES users(uuid),
        name VARCHAR(255),
        photo_profile VARCHAR(255),
        background_profile VARCHAR(255),
        gender VARCHAR(25),
        telp VARCHAR(25),
        birth_date VARCHAR(50),
        roles_number INTEGER CHECK (roles_number >= 0),
        store_name VARCHAR(50),
        store_telp VARCHAR(50),
        created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        deleted_at BIGINT DEFAULT 0
    );

--categories
CREATE TABLE 
    IF NOT EXISTS categories (
        uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
        name VARCHAR(50) NOT NULL UNIQUE,
        created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        deleted_at BIGINT DEFAULT 0
    );

--uoms
CREATE TABLE 
    IF NOT EXISTS uoms (
        uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
        code VARCHAR(10) NOT NULL UNIQUE,
        name VARCHAR(50) NOT NULL,
        created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        deleted_at BIGINT DEFAULT 0
    );

--ingredients
CREATE TABLE 
    IF NOT EXISTS ingredients (
        uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
        name VARCHAR(100) NOT NULL UNIQUE,
        base_uom UUID NOT NULL REFERENCES uoms(uuid),
        minimal_stock NUMERIC(12,3),
        shelf_life_days INT,
        created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        deleted_at BIGINT DEFAULT 0
    );

--ingredient_prices
CREATE TABLE 
    IF NOT EXISTS ingredient_prices (
        uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
        ingredient_uuid UUID NOT NULL REFERENCES ingredients(uuid),
        unit_cost NUMERIC(12,4) NOT NULL,
        effective_at BIGINT NOT NULL,
        created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
        deleted_at BIGINT DEFAULT 0
    );

--products
CREATE TABLE IF NOT EXISTS products (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  category_uuid UUID NOT NULL REFERENCES categories(uuid),
  name VARCHAR(100) NOT NULL,
  sku VARCHAR(50),                                -- unik hanya utk data aktif (via partial unique idx)
  price NUMERIC(12,2) NOT NULL,
  current_recipe_uuid UUID,                       -- FK ditambahkan setelah recipe_sets dibuat
  status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',   -- 'ACTIVE' | 'INACTIVE'
  image_url VARCHAR(255),
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT products_status_valid CHECK (status IN ('ACTIVE','INACTIVE'))
);

--recipe_sets
CREATE TABLE IF NOT EXISTS recipe_sets (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  product_uuid UUID NOT NULL REFERENCES products(uuid),
  name VARCHAR(100),                               -- "Default", "Seasonal A", dll
  yield_qty NUMERIC(12,4) NOT NULL DEFAULT 1,      -- hasil per batch; 1 = per porsi
  effective_from BIGINT,                           -- unix ms (nullable)
  effective_to   BIGINT,                           -- unix ms (nullable)
  is_active BOOLEAN NOT NULL DEFAULT true,         -- batasi 1 aktif per produk (lihat idx di bawah)
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT recipe_sets_yield_pos CHECK (yield_qty > 0),
  CONSTRAINT recipe_sets_effective_range CHECK (
    effective_to IS NULL OR effective_from IS NULL OR effective_to >= effective_from
  )
);

--recipe_items
CREATE TABLE IF NOT EXISTS recipe_items (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  recipe_uuid UUID NOT NULL REFERENCES recipe_sets(uuid) ON DELETE CASCADE,
  ingredient_uuid UUID NOT NULL REFERENCES ingredients(uuid),
  qty NUMERIC(12,4) NOT NULL,                      -- pemakaian per 1 yield
  waste_pct NUMERIC(6,3) DEFAULT 0,                -- 0..1 (mis. 0.05 = 5%)
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0,
  CONSTRAINT recipe_items_qty_pos CHECK (qty > 0),
  CONSTRAINT recipe_items_waste_range CHECK (waste_pct >= 0 AND waste_pct <= 1)
);

--stock_moves
CREATE TABLE IF NOT EXISTS stock_moves (
  uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
  ingredient_uuid UUID NOT NULL REFERENCES ingredients(uuid),
  move VARCHAR(20) NOT NULL,                 -- 'IN' | 'OUT' | 'ADJUST_IN' | 'ADJUST_OUT'
  qty NUMERIC(12,3) NOT NULL,                -- selalu > 0
  unit_cost NUMERIC(12,4),                   -- wajib saat IN/ADJUST_IN
  note VARCHAR(255),
  ref_order_uuid UUID,
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0,

  -- Validasi nilai move
  CONSTRAINT stock_moves_move_valid CHECK (move IN ('IN','OUT','ADJUST_IN','ADJUST_OUT')),
  -- Qty harus positif
  CONSTRAINT stock_moves_qty_pos CHECK (qty > 0),
  -- unit_cost wajib saat stok bertambah
  CONSTRAINT stock_moves_unit_cost_when_in CHECK (
    CASE WHEN move IN ('IN','ADJUST_IN') THEN unit_cost IS NOT NULL ELSE TRUE END
  )
);

--uom_conversions
CREATE TABLE 
  IF NOT EXISTS uom_conversions (
    uuid UUID DEFAULT gen_uuid_v7() NOT NULL PRIMARY KEY,
    from_uom UUID NOT NULL REFERENCES uoms(uuid),
    to_uom UUID NOT NULL REFERENCES uoms(uuid),
    multiplier NUMERIC(18,6) NOT NULL,
    created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
    updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
    deleted_at BIGINT DEFAULT 0
  );

CREATE UNIQUE INDEX users_username_uniq ON users (username);
CREATE UNIQUE INDEX users_email_uniq ON users (email);
CREATE UNIQUE INDEX profiles_user_uuid_uniq ON profiles (user_uuid);
CREATE UNIQUE INDEX IF NOT EXISTS ingredient_prices_uniq
  ON ingredient_prices (ingredient_uuid, effective_at);
CREATE INDEX IF NOT EXISTS stock_moves_ingredient_created_idx
  ON stock_moves (ingredient_uuid, created_at);
CREATE UNIQUE INDEX IF NOT EXISTS uom_conversions_uniq
  ON uom_conversions (from_uom, to_uom);

CREATE UNIQUE INDEX IF NOT EXISTS products_sku_active_uniq
  ON products (lower(sku))
  WHERE deleted_at = 0;
CREATE INDEX IF NOT EXISTS products_category_idx
  ON products (category_uuid);

CREATE INDEX IF NOT EXISTS recipe_sets_prod_efffrom_idx
  ON recipe_sets (product_uuid, effective_from);

CREATE UNIQUE INDEX IF NOT EXISTS recipe_sets_active_uniq
  ON recipe_sets (product_uuid)
  WHERE is_active = true AND deleted_at = 0;

CREATE UNIQUE INDEX IF NOT EXISTS recipe_items_unique_active
  ON recipe_items (recipe_uuid, ingredient_uuid)
  WHERE deleted_at = 0;

CREATE INDEX IF NOT EXISTS recipe_items_ingredient_idx
  ON recipe_items (ingredient_uuid);
