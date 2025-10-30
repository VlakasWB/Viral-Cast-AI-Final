-- Drop middle closing time column from stores
ALTER TABLE stores
    DROP COLUMN IF EXISTS middle_closing_time;