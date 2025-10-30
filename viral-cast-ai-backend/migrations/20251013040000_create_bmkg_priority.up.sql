-- BMKG area priority table to manage fetch order and scheduling
CREATE TABLE IF NOT EXISTS bmkg_area_priority (
  region_code text PRIMARY KEY REFERENCES bmkg_area(region_code),
  priority integer NOT NULL DEFAULT 100,
  active boolean NOT NULL DEFAULT true,
  last_hit_ms BIGINT,
  next_due_ms BIGINT,
  created_at BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW())*1000)::bigint,
  updated_at BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW())*1000)::bigint,
  deleted_at BIGINT DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_bmkg_area_priority_active
  ON bmkg_area_priority (active)
  WHERE deleted_at = 0;

CREATE INDEX IF NOT EXISTS idx_bmkg_area_priority_priority
  ON bmkg_area_priority (priority)
  WHERE deleted_at = 0;