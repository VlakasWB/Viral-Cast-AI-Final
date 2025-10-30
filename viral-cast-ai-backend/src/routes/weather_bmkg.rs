use axum::{
    extract::{Query, State},
    routing::get,
    Router,
};

use crate::handlers::weather_bmkg::{
    get_prioritized_regions, get_region_by_id, get_regions_list, get_weather_prediction,
    update_region_priority,
};
use crate::AppState;

pub fn weather_bmkg_routes() -> Router<AppState> {
    Router::new()
        // BMKG Weather prediction endpoints
        .route(
            "/api/v1/weather_bmkg/prediction",
            get(get_weather_prediction),
        )
        .route(
            "/api/v1/weather_bmkg/prediction",
            axum::routing::post(
                |Query(params): Query<crate::dto::weather_bmkg::WeatherPredictionRequest>,
                 State(state): State<AppState>| async move {
                    crate::handlers::weather_bmkg::get_weather_prediction(
                        Query(params),
                        State(state),
                    )
                    .await
                },
            ),
        )
        // BMKG Region master data endpoints
        .route("/api/v1/weather_bmkg/regions", get(get_regions_list))
        .route("/api/v1/weather_bmkg/regions/:id", get(get_region_by_id))
        .route(
            "/api/v1/weather_bmkg/priorities",
            get(get_prioritized_regions),
        )
        .route(
            "/api/v1/weather_bmkg/priorities",
            axum::routing::post(update_region_priority),
        )
}
