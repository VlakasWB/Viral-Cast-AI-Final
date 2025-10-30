-- [ID] Wilayah (pakai epoch ms untuk audit)
-- [EN] Area master (use epoch ms for audit)
CREATE TABLE IF NOT EXISTS bmkg_area (
  region_code text PRIMARY KEY,
  adm1 text, adm2 text, adm3 text, adm4 text,
  provinsi text, kotkab text, kecamatan text, desa text,
  timezone text NOT NULL,
  lat double precision, lon double precision,

  created_at BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW())*1000)::bigint,
  updated_at BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW())*1000)::bigint,
  deleted_at BIGINT DEFAULT 0
);

-- [ID] Satu "run" per analysis_date (menyimpan raw JSON per batch)
-- [EN] One forecast "run" per analysis_date (store raw JSON per batch)
CREATE TABLE IF NOT EXISTS bmkg_forecast_run (
  region_code text NOT NULL REFERENCES bmkg_area(region_code),
  analysis_ms  BIGINT NOT NULL,  -- [ID] epoch ms ; [EN] epoch ms

  -- [ID] kolom turunan untuk kemudahan query
  -- [EN] generated column for convenient querying
  analysis_ts  timestamptz GENERATED ALWAYS AS (to_timestamp(analysis_ms/1000.0)) STORED,

  fetched_ms   BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW())*1000)::bigint,
  raw_json     jsonb NOT NULL,
  source       text DEFAULT 'bmkg',

  created_at   BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW())*1000)::bigint,
  updated_at   BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW())*1000)::bigint,
  deleted_at   BIGINT DEFAULT 0,

  PRIMARY KEY (region_code, analysis_ms)
);

-- [ID] Fakta prakiraan per slot waktu
-- [EN] Forecast facts per time slot
CREATE TABLE IF NOT EXISTS bmkg_forecast (
  region_code text NOT NULL,
  analysis_ms  BIGINT NOT NULL,
  valid_ms     BIGINT NOT NULL,  -- [ID] epoch ms (UTC) ; [EN] epoch ms (UTC)

  -- [ID] kolom turunan untuk filter/index/group-by
  -- [EN] generated columns for filtering/indexing/grouping
  analysis_ts  timestamptz GENERATED ALWAYS AS (to_timestamp(analysis_ms/1000.0)) STORED,
  valid_ts     timestamptz GENERATED ALWAYS AS (to_timestamp(valid_ms/1000.0)) STORED,

  t real, hu real,
  weather_code smallint,

  -- [ID] Deskripsi cuaca dalam Bahasa Indonesia & Inggris
  -- [EN] Weather description in Indonesian & English
  weather_desc_id text,
  weather_desc_en text,

  ws real, wd text, wd_deg real,
  tcc real, vs_m real, tp_mm real,
  time_index text, image_url text,
  extras jsonb NOT NULL DEFAULT '{}'::jsonb,

  created_at BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW())*1000)::bigint,
  updated_at BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW())*1000)::bigint,
  deleted_at BIGINT DEFAULT 0,

  PRIMARY KEY (region_code, analysis_ms, valid_ms),
  FOREIGN KEY (region_code, analysis_ms)
    REFERENCES bmkg_forecast_run(region_code, analysis_ms) ON DELETE CASCADE
);

-- [ID] Index untuk query cepat (hanya data non-deleted)
-- [EN] Indexes for fast queries (filter non-deleted)
CREATE INDEX IF NOT EXISTS idx_bmkg_forecast_rc_valid_ms
  ON bmkg_forecast (region_code, valid_ms)
  WHERE deleted_at = 0;

CREATE INDEX IF NOT EXISTS idx_bmkg_forecast_rc_valid_ms_latest
  ON bmkg_forecast (region_code, valid_ms, analysis_ms DESC)
  WHERE deleted_at = 0;

CREATE INDEX IF NOT EXISTS idx_bmkg_forecast_valid_ts
  ON bmkg_forecast (valid_ts)
  WHERE deleted_at = 0;

CREATE INDEX IF NOT EXISTS idx_bmkg_forecast_extras_gin
  ON bmkg_forecast USING GIN (extras)
  WHERE deleted_at = 0;

-- (Opsional) index untuk run & area yang tidak terhapus
CREATE INDEX IF NOT EXISTS idx_bmkg_forecast_run_analysis_ms
  ON bmkg_forecast_run (analysis_ms DESC)
  WHERE deleted_at = 0;

CREATE INDEX IF NOT EXISTS idx_bmkg_area_region_code_alive
  ON bmkg_area (region_code)
  WHERE deleted_at = 0;
