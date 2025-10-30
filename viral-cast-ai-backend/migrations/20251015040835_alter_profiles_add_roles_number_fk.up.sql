-- Add FK constraint on profiles.roles_number referencing roles.number
ALTER TABLE profiles
  ADD CONSTRAINT profiles_roles_number_fk
    FOREIGN KEY (roles_number)
    REFERENCES roles(number)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT;
-- Add up migration script here