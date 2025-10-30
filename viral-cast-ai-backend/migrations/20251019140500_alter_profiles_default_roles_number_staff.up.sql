-- Set default roles_number to 6 (Staff)
ALTER TABLE profiles
  ALTER COLUMN roles_number SET DEFAULT 6;

-- Ensure inserts with NULL still get default via trigger
CREATE OR REPLACE FUNCTION profiles_set_default_roles_number()
RETURNS TRIGGER AS $$
BEGIN
  IF NEW.roles_number IS NULL THEN
    NEW.roles_number := 6; -- Staff
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER profiles_default_roles_number_before_insert
BEFORE INSERT ON profiles
FOR EACH ROW
EXECUTE FUNCTION profiles_set_default_roles_number();

-- Optional: backfill existing profiles without a role
UPDATE profiles SET roles_number = 6 WHERE roles_number IS NULL;