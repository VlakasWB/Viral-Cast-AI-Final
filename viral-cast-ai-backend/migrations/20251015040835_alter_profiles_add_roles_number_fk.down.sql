-- Drop FK constraint on profiles.roles_number
ALTER TABLE profiles
  DROP CONSTRAINT IF EXISTS profiles_roles_number_fk;
-- Add down migration script here