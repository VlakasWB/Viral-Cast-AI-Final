-- Alter stores opening/closing/middle times to use Unix timestamp (milliseconds)
-- This ensures easy timezone conversion across regions

ALTER TABLE stores
    DROP COLUMN IF EXISTS opening_time,
    DROP COLUMN IF EXISTS middle_closing_time,
    DROP COLUMN IF EXISTS closing_time;

ALTER TABLE stores
    ADD COLUMN IF NOT EXISTS opening_time BIGINT NULL,
    ADD COLUMN IF NOT EXISTS middle_closing_time BIGINT NULL,
    ADD COLUMN IF NOT EXISTS closing_time BIGINT NULL;