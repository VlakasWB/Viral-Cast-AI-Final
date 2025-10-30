-- Add store opening and closing time columns
ALTER TABLE stores
    ADD COLUMN IF NOT EXISTS opening_time TIME NULL,
    ADD COLUMN IF NOT EXISTS closing_time TIME NULL;