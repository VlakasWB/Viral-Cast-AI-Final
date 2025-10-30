-- Drop social links columns from profiles
ALTER TABLE profiles
  DROP COLUMN IF EXISTS website,
  DROP COLUMN IF EXISTS instagram,
  DROP COLUMN IF EXISTS whatsapp;
-- Add down migration script here