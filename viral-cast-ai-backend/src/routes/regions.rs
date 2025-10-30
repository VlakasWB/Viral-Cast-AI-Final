use axum::{routing::get, Router};

use crate::handlers::regions::{
    get_district_by_code, get_district_detail, get_districts, get_province_by_code,
    get_province_detail, get_provinces, get_regencies, get_regency_by_code, get_regency_detail,
    get_village_by_code, get_village_detail, get_villages,
};
use crate::AppState;

pub fn create_regions_routes() -> Router<AppState> {
    Router::new()
        // Province routes
        .route("/api/v1/regions/provinces", get(get_provinces))
        .route("/api/v1/regions/provinces/detail", get(get_province_detail))
        .route("/api/v1/regions/provinces/:code", get(get_province_by_code))
        // Regency routes
        .route("/api/v1/regions/regencies", get(get_regencies))
        .route("/api/v1/regions/regencies/detail", get(get_regency_detail))
        .route("/api/v1/regions/regencies/:code", get(get_regency_by_code))
        // District routes
        .route("/api/v1/regions/districts", get(get_districts))
        .route("/api/v1/regions/districts/detail", get(get_district_detail))
        .route("/api/v1/regions/districts/:code", get(get_district_by_code))
        // Village routes
        .route("/api/v1/regions/villages", get(get_villages))
        .route("/api/v1/regions/villages/detail", get(get_village_detail))
        .route("/api/v1/regions/villages/:code", get(get_village_by_code))
}
