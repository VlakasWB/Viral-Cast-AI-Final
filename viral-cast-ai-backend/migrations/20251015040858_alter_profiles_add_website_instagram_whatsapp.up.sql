-- Add social links columns to profiles
ALTER TABLE profiles
  ADD COLUMN IF NOT EXISTS website VARCHAR(255),
  ADD COLUMN IF NOT EXISTS instagram VARCHAR(255),
  ADD COLUMN IF NOT EXISTS whatsapp VARCHAR(50);
-- Add up migration script here