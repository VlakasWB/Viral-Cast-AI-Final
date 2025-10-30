-- Drop trigger and function, remove default
DROP TRIGGER IF EXISTS profiles_default_roles_number_before_insert ON profiles;
DROP FUNCTION IF EXISTS profiles_set_default_roles_number();

ALTER TABLE profiles
  ALTER COLUMN roles_number DROP DEFAULT;