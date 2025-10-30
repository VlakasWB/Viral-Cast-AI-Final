DROP INDEX IF EXISTS idx_bmkg_forecast_extras_gin;
DROP INDEX IF EXISTS idx_bmkg_forecast_valid_ts;
DROP INDEX IF EXISTS idx_bmkg_forecast_rc_valid_ms_latest;
DROP INDEX IF EXISTS idx_bmkg_forecast_rc_valid_ms;
DROP INDEX IF EXISTS idx_bmkg_forecast_run_analysis_ms;
DROP INDEX IF EXISTS idx_bmkg_area_region_code_alive;

DROP TABLE IF EXISTS bmkg_forecast;
DROP TABLE IF EXISTS bmkg_forecast_run;
DROP TABLE IF EXISTS bmkg_area;
