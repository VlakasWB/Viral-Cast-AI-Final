-- Drop indexes
DROP INDEX IF EXISTS idx_user_rate_limits_ip_window;
DROP INDEX IF EXISTS idx_user_rate_limits_window;

-- Drop tables
DROP TABLE IF EXISTS user_rate_limits;
DROP TABLE IF EXISTS user_input_controls;