-- Remove store opening and closing time columns
ALTER TABLE stores
    DROP COLUMN IF EXISTS opening_time,
    DROP COLUMN IF EXISTS closing_time;