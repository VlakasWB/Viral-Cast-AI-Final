-- Add middle closing time column to stores
ALTER TABLE stores
    ADD COLUMN IF NOT EXISTS middle_closing_time TIME NULL;