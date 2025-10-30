use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use chrono::{Days, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use reqwest::{Client, StatusCode as ReqwestStatusCode};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};
use tracing::{info, warn};
use uuid::Uuid;

// ID: Gunakan komponen batching tanpa mengimpor tipe privat
// EN: Use batching components without importing private types
use crate::services::batch_processor::{BatchConfig, BatchProcessor};
use crate::services::rate_limiter::GroqRateLimiter;
// ID: Nonaktifkan import layanan agar kompilasi stabil saat modul dinonaktifkan.
// EN: Disable service import to stabilize compilation when module is disabled.

use crate::dto::ai::{GroqApiRequest, GroqApiResponse, GroqMessage};
use crate::dto::api::ApiResponse;
use crate::dto::store_ingredient_predictions::{
    GenerateIngredientPredictionParams, IngredientPredictionDto, IngredientWeatherBriefDto,
    StoreIngredientPredictionResponseDto,
};
use crate::middleware::jwt::JWTAuthMiddleware;
use crate::models::store_ingredient_predictions::StoreIngredientPredictionWithIngredient;
use crate::models::store_product_predictions::{ProductSnapshot, WeatherSlotSnapshot};
use crate::models::weather_bmkg::{BMKGWeatherResponse, WeatherPrediction};
use crate::repository::store_ingredient_predictions as ingredient_predictions_repository;
use crate::repository::store_ingredient_predictions::{
    IngredientSnapshot, NewStoreIngredientPrediction,
};
use crate::repository::store_product_predictions as store_predictions_repository;
use crate::repository::store_product_predictions::RegionContext;
use crate::repository::stores as stores_repository;
use crate::repository::weather_bmkg as weather_repository;
use crate::AppState;

#[derive(Debug, Deserialize)]
struct LlmResponsePayload {
    summary: Option<String>,
    weather_brief: Option<LlmWeatherBrief>,
    ingredients: Vec<LlmIngredientPrediction>,
}

#[derive(Debug, Deserialize)]
struct LlmWeatherBrief {
    summary: Option<String>,
    key_weather_impacts: Option<Vec<String>>,
    temp_min_c: Option<f32>,
    temp_max_c: Option<f32>,
    humidity_avg: Option<f32>,
    precipitation_total_mm: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct LlmIngredientPrediction {
    ingredient_catalog_uuid: Uuid,
    restock_label: String,
    restock_probability: Option<f32>,
    recommended_restock_qty: Option<f64>,
    current_stock_qty: Option<f64>,
    reasoning: Option<String>,
    forecast_error_margin_pct: Option<f32>,
}

struct ParsedLlmPayload {
    payload: LlmResponsePayload,
    value: Value,
    was_repaired: bool,
}

const DEFAULT_MAX_PRODUCTS: i64 = 30;
const DEFAULT_MAX_INGREDIENTS: usize = 40;
const MAX_INGREDIENTS_PER_BATCH: usize = 3;
const MAX_PRODUCTS_IN_CONTEXT: usize = 8;
const MAX_LINKED_PRODUCTS_PER_INGREDIENT: usize = 3;
const GROQ_MAX_ATTEMPTS: usize = 4;
const DEFAULT_MAX_TOKENS: u32 = 1024;
const MAX_TOKENS_MIN: u32 = 64;
const MAX_TOKENS_MAX: u32 = 2048;
const DEFAULT_PREDICTION_MODEL: &str = "llama-3.1-8b-instant";

pub async fn generate_store_ingredient_predictions_handler(
    State(state): State<Arc<AppState>>,
    Extension(jwt_auth): Extension<JWTAuthMiddleware>,
    Query(params): Query<GenerateIngredientPredictionParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let user_uuid = jwt_auth.user.uuid;
    let store = stores_repository::get_store_by_user_uuid(&state.db, user_uuid)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json_error(
                    404,
                    "Store not found".to_string(),
                    "Akun ini belum memiliki store. Silakan buat store terlebih dahulu."
                        .to_string(),
                )),
            )
        })?;

    let region_context = store_predictions_repository::resolve_region_context(
        &state.db,
        store.village_code.as_deref(),
        store.district_code.as_deref(),
        store.regency_code.as_deref(),
    )
    .await
    .map_err(internal_error)?;

    let region_context = region_context.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json_error(
                400,
                "Region code unavailable for store".to_string(),
                "Store belum memiliki kode wilayah BMKG yang valid. Lengkapi data wilayah terlebih dahulu."
                    .to_string(),
            )),
        )
    })?;

    let timezone = region_context
        .timezone
        .as_deref()
        .and_then(|name| Tz::from_str(name).ok())
        .unwrap_or(chrono_tz::UTC);
    let now_local = timezone.from_utc_datetime(&Utc::now().naive_utc());
    let target_date: NaiveDate = now_local
        .date_naive()
        .checked_add_days(Days::new(1))
        .unwrap_or(now_local.date_naive());

    let products: Vec<ProductSnapshot> =
        store_predictions_repository::fetch_active_products(&state.db, Some(DEFAULT_MAX_PRODUCTS))
            .await
            .map_err(internal_error)?;

    let ingredients: Vec<IngredientSnapshot> =
        ingredient_predictions_repository::fetch_ingredient_snapshots(
            &state.db,
            Some(DEFAULT_MAX_INGREDIENTS),
        )
        .await
        .map_err(internal_error)?;

    if ingredients.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json_error(
                400,
                "No ingredient data".to_string(),
                "Tidak ditemukan bahan baku aktif yang terkait dengan resep produk.".to_string(),
            )),
        ));
    }

    let weather_slots =
        load_weather_slots(&state, &region_context.region_code, target_date).await?;
    let weather_brief = summarize_weather(&weather_slots);

    let temperature = params.temperature.unwrap_or(0.2_f32).clamp(0.0, 1.0);
    let max_tokens = match params.max_tokens {
        Some(requested) => {
            let clamped = requested.clamp(MAX_TOKENS_MIN, MAX_TOKENS_MAX);
            if requested != clamped {
                warn!(
                    "Requested max_tokens {} adjusted to safe range {}-{}",
                    requested, MAX_TOKENS_MIN, MAX_TOKENS_MAX
                );
            }
            clamped
        }
        None => DEFAULT_MAX_TOKENS,
    };

    let predictions_model = resolve_predictions_model();
    let base_model =
        std::env::var("GROQ_MODEL").unwrap_or_else(|_| DEFAULT_PREDICTION_MODEL.to_string());

    let ingredient_chunks: Vec<Vec<IngredientSnapshot>> = ingredients
        .chunks(MAX_INGREDIENTS_PER_BATCH)
        .map(|chunk| chunk.to_vec())
        .collect();
    let total_batches = ingredient_chunks.len().max(1);
    let total_ingredients = ingredients.len();

    info!(
        base_model = %base_model,
        predictions_model = %predictions_model,
        max_tokens = ?max_tokens,
        temperature = ?temperature,
        total_ingredients,
        total_batches,
        "Preparing ingredient LLM batches"
    );

    let ingredient_lookup: HashMap<Uuid, &IngredientSnapshot> = ingredients
        .iter()
        .map(|i| (i.ingredient_catalog_uuid, i))
        .collect();

    let simplified_products = simplify_products_context(&products);
    let base_context = json!({
        "store": {
            "uuid": store.uuid,
            "name": store.name,
            "region_code": region_context.region_code,
            "timezone": region_context.timezone,
        },
        "target_date": target_date,
        "weather_brief": weather_brief,
        "products": simplified_products,
    });

    let log_dir =
        std::env::var("LLM_PROMPT_LOG_DIR").unwrap_or_else(|_| "logs/llm_prompts".to_string());
    if let Err(e) = tokio::fs::create_dir_all(&log_dir).await {
        warn!("Failed to create LLM prompt log dir '{}': {}", log_dir, e);
    }
    let base_ts_ms = Utc::now().timestamp_millis();

    // ID: Inisialisasi rate limiter untuk Groq API
    // EN: Initialize rate limiter for Groq API
    let rate_limiter = GroqRateLimiter::new();

    // ID: Cek apakah handler harus menggunakan service prediksi baru
    // EN: Check whether handler should use the new prediction service
    // ID: Jalur layanan dinonaktifkan untuk stabilisasi build; gunakan handler inline.
    // EN: Service path disabled for build stabilization; use inline handler instead.

    let mut new_prediction_rows: Vec<NewStoreIngredientPrediction> = Vec::new();

    for (batch_index, chunk) in ingredient_chunks.iter().enumerate() {
        // ID: Tambahkan delay antar batch untuk menghindari rate limit (kecuali batch pertama)
        // EN: Add delay between batches to avoid rate limit (except first batch)
        if batch_index > 0 {
            let batch_delay_secs = std::env::var("GROQ_BATCH_DELAY_SECS")
                .unwrap_or_else(|_| "15".to_string())
                .parse::<u64>()
                .unwrap_or(15);

            info!(
                batch = batch_index + 1,
                delay_secs = batch_delay_secs,
                "Waiting between batches to respect rate limits"
            );

            sleep(Duration::from_secs(batch_delay_secs)).await;
        }
        let mut context_json = base_context.clone();
        if let Value::Object(ref mut map) = context_json {
            map.insert(
                "ingredients".to_string(),
                build_ingredient_context_chunk(chunk),
            );
            map.insert(
                "batch".to_string(),
                json!({
                    "index": batch_index + 1,
                    "of": total_batches,
                    "items": chunk.len(),
                    "total": total_ingredients,
                }),
            );
        }

        let (system_prompt, user_prompt) =
            build_prompts(target_date, &context_json, batch_index + 1, total_batches);

        info!(
            batch = batch_index + 1,
            total_batches = total_batches,
            ingredients_in_batch = chunk.len(),
            "Calling ingredient LLM batch"
        );

        let file_name = format!(
            "store_ingredient_predictions_{}_{}_{}_batch{}of{}.json",
            store.uuid,
            region_context.region_code,
            base_ts_ms,
            batch_index + 1,
            total_batches
        );
        let mut path = PathBuf::from(&log_dir);
        path.push(file_name);
        let prompt_log_json = json!({
            "timestamp_ms": base_ts_ms,
            "endpoint": "/api/v1/stores/ingredient-predictions",
            "store_uuid": store.uuid,
            "region_code": region_context.region_code,
            "batch_index": batch_index + 1,
            "total_batches": total_batches,
            "model": predictions_model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "system_prompt": system_prompt.clone(),
            "user_prompt": user_prompt.clone(),
            "context": context_json,
        });
        match serde_json::to_string_pretty(&prompt_log_json) {
            Ok(content) => {
                if let Err(e) = tokio::fs::write(&path, content).await {
                    warn!(
                        "Failed to write LLM prompt log file '{}': {}",
                        path.display(),
                        e
                    );
                } else {
                    info!(
                        batch = batch_index + 1,
                        total_batches = total_batches,
                        path = %path.display(),
                        "LLM prompt log written"
                    );
                }
            }
            Err(e) => warn!("Failed to serialize LLM prompt log JSON: {}", e),
        }

        let llm_response = call_groq_with_model(
            vec![
                GroqMessage {
                    role: "system".to_string(),
                    content: system_prompt.clone(),
                },
                GroqMessage {
                    role: "user".to_string(),
                    content: user_prompt.clone(),
                },
            ],
            predictions_model.clone(),
            Some(max_tokens),
            Some(temperature),
            &rate_limiter,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json_error(
                    502,
                    "LLM request failed".to_string(),
                    format!("Gagal meminta rekomendasi bahan baku dari model LLM: {e}"),
                )),
            )
        })?;

        let llm_text = llm_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        let ParsedLlmPayload {
            payload,
            value: parsed_llm_value,
            was_repaired,
        } = parse_llm_payload(&llm_text).map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json_error(
                    502,
                    "Failed to parse LLM response".to_string(),
                    format!(
                        "Respon model LLM tidak sesuai format JSON yang diminta: {e}. Raw response: {}",
                        llm_text
                    ),
                )),
            )
        })?;
        if was_repaired {
            warn!(
                batch = batch_index + 1,
                total_batches = total_batches,
                "LLM response required repair before parsing into JSON"
            );
        }

        let llm_prompt_record = json!({
            "system_prompt": system_prompt,
            "user_prompt": user_prompt,
            "context": context_json,
        });

        for ingredient_prediction in payload.ingredients.iter() {
            let snapshot =
                match ingredient_lookup.get(&ingredient_prediction.ingredient_catalog_uuid) {
                    Some(snapshot) => snapshot,
                    None => continue,
                };

            let restock_label = ingredient_prediction.restock_label.trim().to_uppercase();
            let restock_probability = ingredient_prediction
                .restock_probability
                .map(|v| v.clamp(0.0, 1.0));
            let recommended_qty = ingredient_prediction
                .recommended_restock_qty
                .unwrap_or(0.0)
                .max(0.0);
            let recommended_restock_qty =
                Decimal::from_f64(recommended_qty).unwrap_or_else(|| Decimal::from(0));
            let forecast_error_pct = ingredient_prediction
                .forecast_error_margin_pct
                .or_else(|| fallback_error_margin(restock_probability));

            let base_reasoning = ingredient_prediction
                .reasoning
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let llm_reasoning = if let Some(error_pct) = forecast_error_pct {
                if base_reasoning.is_empty() {
                    Some(format!(
                        "Margin kesalahan prediksi diperkirakan ±{error_pct:.1}%.",
                        error_pct = error_pct
                    ))
                } else {
                    Some(format!(
                        "{}. Margin kesalahan prediksi diperkirakan ±{error_pct:.1}%.",
                        base_reasoning.trim_end_matches(|c: char| c == '.' || c.is_whitespace()),
                        error_pct = error_pct
                    ))
                }
            } else if base_reasoning.is_empty() {
                None
            } else {
                Some(base_reasoning)
            };

            new_prediction_rows.push(NewStoreIngredientPrediction {
                store_uuid: store.uuid,
                ingredient_catalog_uuid: ingredient_prediction.ingredient_catalog_uuid,
                region_code: Some(region_context.region_code.clone()),
                restock_label,
                restock_probability,
                recommended_restock_qty,
                current_stock_qty: snapshot.current_stock_qty,
                minimum_stock_qty: snapshot.minimum_stock_qty,
                unit_of_measure_code: snapshot.unit_of_measure_code.clone(),
                unit_of_measure_name: snapshot.unit_of_measure_name.clone(),
                weather_summary: payload
                    .weather_brief
                    .as_ref()
                    .and_then(|brief| brief.summary.clone()),
                weather_temp_min_c: payload
                    .weather_brief
                    .as_ref()
                    .and_then(|brief| brief.temp_min_c),
                weather_temp_max_c: payload
                    .weather_brief
                    .as_ref()
                    .and_then(|brief| brief.temp_max_c),
                weather_precip_mm: payload
                    .weather_brief
                    .as_ref()
                    .and_then(|brief| brief.precipitation_total_mm),
                weather_humidity: payload
                    .weather_brief
                    .as_ref()
                    .and_then(|brief| brief.humidity_avg),
                llm_reasoning,
                llm_model: Some(llm_response.model.clone()),
                llm_prompt: Some(llm_prompt_record.clone()),
                llm_response: Some(parsed_llm_value.clone()),
            });
        }
    }

    if new_prediction_rows.is_empty() {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json_error(
                502,
                "LLM produced no predictions".to_string(),
                "Model LLM tidak menghasilkan rekomendasi bahan baku apapun.".to_string(),
            )),
        ));
    }

    ingredient_predictions_repository::upsert_predictions(&state.db, &new_prediction_rows)
        .await
        .map_err(internal_error)?;

    let persisted =
        ingredient_predictions_repository::get_predictions_for_store(&state.db, store.uuid)
            .await
            .map_err(internal_error)?;

    let response_body = build_prediction_response(store.uuid, &region_context, &persisted);

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Rekomendasi bahan baku berhasil dibuat".to_string(),
        data: response_body,
        errors: serde_json::json!({}),
    }))
}

pub async fn get_store_ingredient_predictions_handler(
    State(state): State<Arc<AppState>>,
    Extension(jwt_auth): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let user_uuid = jwt_auth.user.uuid;
    let store = stores_repository::get_store_by_user_uuid(&state.db, user_uuid)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json_error(
                    404,
                    "Store not found".to_string(),
                    "Akun ini belum memiliki store. Silakan buat store terlebih dahulu."
                        .to_string(),
                )),
            )
        })?;

    let region_context = store_predictions_repository::resolve_region_context(
        &state.db,
        store.village_code.as_deref(),
        store.district_code.as_deref(),
        store.regency_code.as_deref(),
    )
    .await
    .map_err(internal_error)?;

    let region_context = region_context.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json_error(
                400,
                "Region code unavailable for store".to_string(),
                "Store belum memiliki kode wilayah BMKG yang valid. Lengkapi data wilayah terlebih dahulu."
                    .to_string(),
            )),
        )
    })?;

    let predictions =
        ingredient_predictions_repository::get_predictions_for_store(&state.db, store.uuid)
            .await
            .map_err(internal_error)?;

    if predictions.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json_error(
                404,
                "No predictions found".to_string(),
                "Belum ada rekomendasi bahan baku yang tersimpan untuk store ini.".to_string(),
            )),
        ));
    }

    let response_body = build_prediction_response(store.uuid, &region_context, &predictions);

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Rekomendasi bahan baku berhasil diambil".to_string(),
        data: response_body,
        errors: serde_json::json!({}),
    }))
}

fn build_prediction_response(
    store_uuid: Uuid,
    region_context: &RegionContext,
    persisted: &[StoreIngredientPredictionWithIngredient],
) -> StoreIngredientPredictionResponseDto {
    let weather_numbers = ingredient_predictions_repository::aggregate_prediction_fields(persisted);
    let weather = if persisted.is_empty() {
        None
    } else {
        Some(IngredientWeatherBriefDto {
            summary: persisted
                .iter()
                .find_map(|p| p.weather_summary.clone())
                .unwrap_or_else(|| "Tidak ada ringkasan cuaca".to_string()),
            temp_min_c: weather_numbers.get("temp_min_c").and_then(|v| *v),
            temp_max_c: weather_numbers.get("temp_max_c").and_then(|v| *v),
            humidity_avg: weather_numbers.get("humidity_avg").and_then(|v| *v),
            precipitation_total_mm: weather_numbers.get("precip_total_mm").and_then(|v| *v),
        })
    };

    let llm_summary = persisted
        .iter()
        .find_map(|p| p.llm_response.clone())
        .and_then(|value| {
            value
                .get("summary")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
        });

    let ingredients = persisted
        .iter()
        .map(|p| IngredientPredictionDto {
            ingredient_catalog_uuid: p.ingredient_catalog_uuid,
            ingredient_name: p.ingredient_name.clone(),
            unit_of_measure_code: p.unit_of_measure_code.clone(),
            unit_of_measure_name: p.unit_of_measure_name.clone(),
            restock_label: p.restock_label.clone(),
            restock_probability: p.restock_probability,
            recommended_restock_qty: p.recommended_restock_qty,
            current_stock_qty: p.current_stock_qty,
            minimum_stock_qty: p.minimum_stock_qty,
            llm_reasoning: p.llm_reasoning.clone(),
            forecast_error_margin_pct: extract_forecast_error_margin(p)
                .or_else(|| fallback_error_margin(p.restock_probability)),
        })
        .collect();

    StoreIngredientPredictionResponseDto {
        store_uuid,
        region_code: Some(region_context.region_code.clone()),
        timezone: region_context.timezone.clone(),
        weather,
        ingredients,
        llm_model: persisted.iter().find_map(|p| p.llm_model.clone()),
        llm_summary,
        generated_at_ms: persisted
            .iter()
            .find_map(|p| p.updated_at.or(p.created_at))
            .unwrap_or_else(|| Utc::now().timestamp_millis()),
    }
}

async fn load_weather_slots(
    state: &AppState,
    region_code: &str,
    target_date: NaiveDate,
) -> Result<Vec<WeatherSlotSnapshot>, (StatusCode, Json<Value>)> {
    let current_slots =
        store_predictions_repository::weather_slots_for_date(&state.db, region_code, target_date)
            .await
            .map_err(internal_error)?;

    if !current_slots.is_empty() {
        return Ok(current_slots);
    }

    match weather_repository::get_latest_raw_forecast_run_json(&state.db, region_code).await {
        Ok(Some(raw_json)) => {
            match serde_json::from_value::<BMKGWeatherResponse>(raw_json.clone()) {
                Ok(weather_response) => {
                    if let Err(e) = upsert_forecasts_from_response(
                        state,
                        region_code,
                        &weather_response,
                        false,
                        None,
                    )
                    .await
                    {
                        warn!(
                            region = region_code,
                            error = %e,
                            "[ingredient_predictions] failed to upsert forecasts from cached run"
                        );
                    } else {
                        let cached_slots = store_predictions_repository::weather_slots_for_date(
                            &state.db,
                            region_code,
                            target_date,
                        )
                        .await
                        .map_err(internal_error)?;

                        if !cached_slots.is_empty() {
                            return Ok(cached_slots);
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        region = region_code,
                        error = %e,
                        "[ingredient_predictions] failed to decode cached bmkg_forecast_run raw JSON"
                    );
                }
            }
        }
        Ok(None) => {}
        Err(e) => {
            warn!(
                region = region_code,
                error = %e,
                "[ingredient_predictions] failed to read cached bmkg_forecast_run"
            );
        }
    }

    match fetch_and_persist_weather(state, region_code).await {
        Ok(_weather_response) => store_predictions_repository::weather_slots_for_date(
            &state.db,
            region_code,
            target_date,
        )
        .await
        .map_err(internal_error),
        Err(err_msg) => Err((
            StatusCode::BAD_GATEWAY,
            Json(json_error(
                502,
                "Failed to fetch weather forecast".to_string(),
                format!(
                    "Gagal mengambil prediksi cuaca dari BMKG untuk region {region_code}: {err_msg}"
                ),
            )),
        )),
    }
}

async fn upsert_forecasts_from_response(
    state: &AppState,
    region_code: &str,
    weather_response: &BMKGWeatherResponse,
    persist_run: bool,
    analysis_ms_override: Option<i64>,
) -> Result<(), String> {
    let analysis_ms = analysis_ms_override
        .or_else(|| {
            weather_response
                .data
                .first()
                .and_then(|pred| pred.analysis_date.as_deref())
                .and_then(|iso| parse_iso_to_ms(iso))
        })
        .unwrap_or_else(|| Utc::now().timestamp_millis());

    if persist_run {
        let raw_json = serde_json::to_value(weather_response).map_err(|e| e.to_string())?;
        weather_repository::insert_forecast_run(&state.db, region_code, analysis_ms, raw_json)
            .await
            .map_err(|e| e.to_string())?;
    }

    for prediction in weather_response.data.iter() {
        weather_repository::insert_forecast(&state.db, region_code, analysis_ms, prediction)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

async fn fetch_and_persist_weather(
    state: &AppState,
    region_code: &str,
) -> Result<BMKGWeatherResponse, String> {
    let weather = fetch_bmkg_weather_data(region_code).await?;
    let analysis_ms = weather
        .data
        .first()
        .and_then(|pred| pred.analysis_date.as_deref())
        .and_then(|iso| parse_iso_to_ms(iso))
        .unwrap_or_else(|| Utc::now().timestamp_millis());

    if state.bmkg_save_json {
        if let Err(e) = dump_weather_json(region_code, &weather) {
            warn!(
                region = region_code,
                error = %e,
                "[ingredient_predictions] failed to write weather JSON dump"
            );
        }
    }

    upsert_forecasts_from_response(state, region_code, &weather, true, Some(analysis_ms))
        .await
        .map_err(|e| e.to_string())?;

    Ok(weather)
}

async fn fetch_bmkg_weather_data(region_code: &str) -> Result<BMKGWeatherResponse, String> {
    const BMKG_API_BASE_URL: &str = "https://api.bmkg.go.id/publik/prakiraan-cuaca";
    let url = format!("{}?adm4={}", BMKG_API_BASE_URL, region_code);

    let response = Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("BMKG API returned status {}", response.status()));
    }

    let bmkg_api_response: crate::models::weather_bmkg::BMKGApiResponse =
        response.json().await.map_err(|e| e.to_string())?;

    if let Some(location_data) = bmkg_api_response.data.first() {
        let mut all_predictions: Vec<WeatherPrediction> = Vec::new();
        for chunk in location_data.cuaca.iter() {
            all_predictions.extend_from_slice(chunk);
        }

        Ok(BMKGWeatherResponse {
            data: all_predictions,
            lokasi: bmkg_api_response.lokasi,
        })
    } else {
        Err("No weather data found for the specified location".into())
    }
}

fn dump_weather_json(region_code: &str, weather: &BMKGWeatherResponse) -> Result<(), String> {
    let dir = std::path::Path::new("data/json");
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let file_name = format!(
        "bmkg_prediction_{}_{}.json",
        region_code.replace('.', "-"),
        Utc::now().format("%Y%m%d%H%M%S")
    );
    let path = dir.join(file_name);
    let json_string = serde_json::to_string_pretty(weather).map_err(|e| e.to_string())?;
    std::fs::write(path, json_string).map_err(|e| e.to_string())
}

fn summarize_weather(slots: &[WeatherSlotSnapshot]) -> Value {
    if slots.is_empty() {
        return json!({
            "summary": "Tidak ada data prakiraan cuaca yang tersedia",
            "temp_min_c": null,
            "temp_max_c": null,
            "humidity_avg": null,
            "precipitation_total_mm": null
        });
    }

    let mut temp_min: Option<f32> = None;
    let mut temp_max: Option<f32> = None;
    let mut humidity_sum = 0.0f32;
    let mut humidity_count = 0.0f32;
    let mut precipitation_sum = 0.0f32;
    let mut weather_counts: HashMap<String, usize> = HashMap::new();

    for slot in slots {
        if let Some(temp) = slot.temperature_c {
            temp_min = Some(temp_min.map(|current| current.min(temp)).unwrap_or(temp));
            temp_max = Some(temp_max.map(|current| current.max(temp)).unwrap_or(temp));
        }
        if let Some(humidity) = slot.humidity_pct {
            humidity_sum += humidity;
            humidity_count += 1.0;
        }
        if let Some(precip) = slot.precipitation_mm {
            precipitation_sum += precip;
        }
        if let Some(desc) = slot
            .weather_desc_en
            .clone()
            .or(slot.weather_desc_id.clone())
        {
            *weather_counts.entry(desc).or_insert(0) += 1;
        }
    }

    let dominant_weather = weather_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(desc, _)| desc)
        .unwrap_or_else(|| "Cuaca beragam".to_string());

    let humidity_avg = if humidity_count > 0.0 {
        Some((humidity_sum / humidity_count * 10.0).round() / 10.0)
    } else {
        None
    };

    let precipitation_total = if precipitation_sum > 0.0 {
        Some((precipitation_sum * 10.0).round() / 10.0)
    } else {
        None
    };

    json!({
        "summary": dominant_weather,
        "temp_min_c": temp_min,
        "temp_max_c": temp_max,
        "humidity_avg": humidity_avg,
        "precipitation_total_mm": precipitation_total,
    })
}

fn parse_iso_to_ms(iso: &str) -> Option<i64> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        Some(dt.timestamp_millis())
    } else {
        chrono::DateTime::parse_from_str(iso, "%Y-%m-%d %H:%M:%S%z")
            .ok()
            .map(|dt| dt.timestamp_millis())
    }
}

fn build_prompts(
    target_date: NaiveDate,
    context_json: &Value,
    batch_index: usize,
    total_batches: usize,
) -> (String, String) {
    let mut system_prompt = "Anda analis operasional dapur F&B. Prediksi kebutuhan restock besok memakai data resep aktif, stok bahan saat ini, dan ringkasan cuaca BMKG. Beri rekomendasi realistis, sebut kisaran suhu & kondisi cuaca, jelaskan dampak jika cuaca membaik atau memburuk, dan cantumkan margin kesalahan (%).".to_string();

    if total_batches > 1 {
        system_prompt.push_str(&format!(
            " Proseskan hanya bahan yang ada pada batch {} dari {} dan abaikan bahan di batch lain.",
            batch_index, total_batches
        ));
    }

    let batch_instruction = if total_batches > 1 {
        format!(
            "Ini adalah batch {} dari {}. Hanya kembalikan data untuk daftar bahan pada batch ini dan pastikan struktur JSON tetap konsisten.",
            batch_index, total_batches
        )
    } else {
        "Fokuskan hasil hanya pada bahan yang ada di daftar berikut.".to_string()
    };

    let user_prompt = format!(
        "Gunakan konteks JSON berikut untuk menghitung kebutuhan restock bahan pada {date}. {batch_instruction} \
        Balas dengan JSON persis:
        {{
          \"summary\": string,
          \"weather_brief\": {{
            \"summary\": string,
            \"temp_min_c\": number|null,
            \"temp_max_c\": number|null,
            \"humidity_avg\": number|null,
            \"precipitation_total_mm\": number|null
          }},
          \"ingredients\": [
            {{
              \"ingredient_catalog_uuid\": \"UUID\",
              \"restock_label\": \"CRITICAL\"|\"HIGH\"|\"MEDIUM\"|\"LOW\",
              \"restock_probability\": number|null,
              \"recommended_restock_qty\": number,
              \"forecast_error_margin_pct\": number|null,
              \"reasoning\": string
            }}
          ]
        }}
        Pedoman: (1) bahasa Indonesia singkat; (2) setiap reasoning sebut suhu (misal \"24-30°C\") dan kondisi cuaca ringkas; \
        (3) jelaskan dampak jika cuaca lebih panas/dingin atau kering/basah; (4) pertimbangkan stok & minimum stok; \
        (5) forecast_error_margin_pct berupa angka 0-100. \
        Konteks:\n{context}",
        batch_instruction = batch_instruction,
        date = target_date,
        context = serde_json::to_string_pretty(context_json).unwrap_or_default()
    );

    (system_prompt, user_prompt)
}

fn simplify_products_context(products: &[ProductSnapshot]) -> Value {
    let simplified: Vec<Value> = products
        .iter()
        .take(MAX_PRODUCTS_IN_CONTEXT)
        .map(|product| {
            json!({
                "product_uuid": product.uuid,
                "product_name": product.name,
                "product_sku": product.sku,
                "price": decimal_to_f64(Some(product.price)),
            })
        })
        .collect();
    Value::from(simplified)
}

fn build_ingredient_context_chunk(chunk: &[IngredientSnapshot]) -> Value {
    let simplified: Vec<Value> = chunk
        .iter()
        .map(|ingredient| {
            let linked_products: Vec<Value> = ingredient
                .linked_products
                .iter()
                .take(MAX_LINKED_PRODUCTS_PER_INGREDIENT)
                .map(|usage| {
                    json!({
                        "product_uuid": usage.product_uuid,
                        "product_name": usage.product_name,
                        "product_sku": usage.product_sku,
                        "recipe_sets_uuid": usage.recipe_sets_uuid,
                        "recipe_yield_qty": decimal_to_f64(usage.recipe_yield_qty),
                        "ingredient_quantity": decimal_to_f64(usage.ingredient_quantity),
                        "waste_percent": decimal_to_f64(usage.waste_percent),
                    })
                })
                .collect();

            json!({
                "ingredient_catalog_uuid": ingredient.ingredient_catalog_uuid,
                "ingredient_name": ingredient.ingredient_name,
                "ingredient_stock_uuid": ingredient.ingredient_stock_uuid,
                "unit_of_measure_code": ingredient.unit_of_measure_code,
                "unit_of_measure_name": ingredient.unit_of_measure_name,
                "current_stock_qty": decimal_to_f64(ingredient.current_stock_qty),
                "minimum_stock_qty": decimal_to_f64(ingredient.minimum_stock_qty),
                "linked_products": linked_products,
            })
        })
        .collect();

    Value::from(simplified)
}

fn decimal_to_f64(value: Option<Decimal>) -> Option<f64> {
    value.and_then(|decimal| decimal.to_f64())
}

fn parse_retry_after_seconds(body: &str) -> Option<f64> {
    if let Some(idx) = body.to_lowercase().find("try again in ") {
        let rest = &body[idx + "try again in ".len()..];
        let mut number = String::new();
        for ch in rest.chars() {
            if ch.is_ascii_digit() || ch == '.' {
                number.push(ch);
            } else {
                break;
            }
        }
        if let Ok(value) = number.parse::<f64>() {
            return Some(value);
        }
    }
    None
}

async fn call_groq_with_model(
    messages: Vec<GroqMessage>,
    model: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    rate_limiter: &GroqRateLimiter,
) -> Result<GroqApiResponse, String> {
    let api_key =
        std::env::var("GROQ_API_KEY").map_err(|_| "GROQ_API_KEY must be set".to_string())?;
    let api_url = std::env::var("GROQ_API_URL")
        .unwrap_or_else(|_| "https://api.groq.com/openai/v1/chat/completions".to_string());

    let req_body = GroqApiRequest {
        messages: messages.clone(),
        model: model.clone(),
        max_tokens,
        temperature,
    };

    // ID: Buat cache key untuk request ini
    // EN: Create cache key for this request
    let mut hasher = DefaultHasher::new();
    hasher.write(
        serde_json::to_string(&req_body)
            .unwrap_or_default()
            .as_bytes(),
    );
    let cache_key = format!("{:x}", hasher.finish());

    // ID: Cek cache terlebih dahulu (cache menyimpan string JSON)
    // EN: Check cache first (cache stores JSON string)
    if let Some(cached_json) = rate_limiter.get_cached_result(&cache_key).await {
        info!("Using cached Groq response for request");
        if let Ok(parsed) = serde_json::from_str::<GroqApiResponse>(&cached_json) {
            return Ok(parsed);
        } else {
            warn!("Cached Groq response failed to parse, ignoring cache entry");
        }
    }

    // ID: Estimasi token untuk request ini (rough estimation)
    // EN: Estimate tokens for this request (rough estimation)
    let estimated_tokens_usize = messages
        .iter()
        .map(|msg| msg.content.len() / 4) // Rough token estimation
        .sum::<usize>()
        + max_tokens.unwrap_or(DEFAULT_MAX_TOKENS) as usize;
    let estimated_tokens = estimated_tokens_usize.min(u32::MAX as usize) as u32;

    // ID: Cek rate limit sebelum melakukan request
    // EN: Check rate limit before making request
    if !rate_limiter.can_make_request(estimated_tokens).await {
        let wait_dur = rate_limiter.get_wait_time().await;
        let wait_time = wait_dur.as_secs();
        warn!(
            estimated_tokens,
            wait_time_secs = wait_time,
            "Rate limit would be exceeded, waiting before request"
        );

        if wait_time > 0 {
            sleep(Duration::from_secs(wait_time)).await;
        }

        // ID: Cek lagi setelah menunggu
        // EN: Check again after waiting
        if !rate_limiter.can_make_request(estimated_tokens).await {
            return Err("Rate limit exceeded, please try again later".to_string());
        }
    }

    if let Ok(payload_json) = serde_json::to_string(&req_body) {
        info!("Sending Groq request payload: {}", payload_json);
    }

    let client = Client::new();
    let mut backoff_secs = 5.0;

    for attempt in 1..=GROQ_MAX_ATTEMPTS {
        let response = client
            .post(&api_url)
            .bearer_auth(&api_key)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        let retry_after_header = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok());
        let body = response.text().await.map_err(|e| e.to_string())?;

        if status == ReqwestStatusCode::TOO_MANY_REQUESTS {
            let wait_secs = retry_after_header
                .or_else(|| parse_retry_after_seconds(&body))
                .unwrap_or(backoff_secs)
                .clamp(1.0, 60.0);

            warn!(
                attempt,
                wait_secs, "Groq rate limited, backing off before retry"
            );

            if attempt == GROQ_MAX_ATTEMPTS {
                return Err(format!(
                    "Groq rate limit reached after {attempt} attempts. Last body: {}",
                    body
                ));
            }

            sleep(Duration::from_secs_f64(wait_secs)).await;
            backoff_secs = (backoff_secs * 1.5).min(60.0);
            continue;
        }

        if !status.is_success() {
            let snippet: String = body.chars().take(512).collect();
            warn!(
                %status,
                body_snippet = %snippet,
                "Groq API responded with non-success status"
            );
            return Err(format!(
                "Groq API responded with status {} and body: {}",
                status, snippet
            ));
        }

        let groq_response = serde_json::from_str::<GroqApiResponse>(&body).map_err(|e| {
            let snippet: String = body.chars().take(512).collect();
            format!("Failed to parse Groq response: {e}. Body: {snippet}")
        })?;

        // ID: Catat penggunaan token dan cache hasil
        // EN: Record token usage and cache result
        rate_limiter.record_usage(estimated_tokens).await;
        let ttl_hours = std::env::var("CACHE_TTL_HOURS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(24);
        let ttl = Duration::from_secs(ttl_hours * 3600);
        // ID: Simpan JSON asli ke cache
        // EN: Save raw JSON to cache
        rate_limiter
            .cache_result(cache_key, body.clone(), ttl)
            .await;

        return Ok(groq_response);
    }

    Err("Groq request exhausted retry attempts".to_string())
}

fn resolve_predictions_model() -> String {
    std::env::var("GROQ_PREDICTIONS_MODEL")
        .ok()
        .and_then(|value| {
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        })
        .or_else(|| {
            std::env::var("GROQ_MODEL")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| DEFAULT_PREDICTION_MODEL.to_string())
}

fn parse_llm_payload(raw: &str) -> Result<ParsedLlmPayload, String> {
    let fragment = extract_json_payload(raw).unwrap_or_else(|| raw.trim().to_string());
    if fragment.is_empty() {
        return Err("LLM response was empty".to_string());
    }

    let mut candidates: Vec<(String, bool)> = Vec::new();
    candidates.push((fragment.clone(), false));

    if let Some(repaired) = repair_json_fragment(&fragment) {
        if !repaired.is_empty()
            && !candidates
                .iter()
                .any(|(candidate, _)| candidate.as_str() == repaired.as_str())
        {
            let repaired_flag = repaired != fragment;
            candidates.push((repaired, repaired_flag));
        }
    }

    let mut errors: Vec<String> = Vec::new();

    for (candidate, repaired_flag) in candidates {
        match serde_json::from_str::<Value>(&candidate) {
            Ok(value) => {
                if let Ok(payload) = serde_json::from_value::<LlmResponsePayload>(value.clone()) {
                    return Ok(ParsedLlmPayload {
                        payload,
                        value,
                        was_repaired: repaired_flag,
                    });
                }
                if let Some(payload) = coerce_payload_from_value(&value) {
                    return Ok(ParsedLlmPayload {
                        payload,
                        value,
                        was_repaired: repaired_flag,
                    });
                }
                errors.push("Parsed JSON but structure mismatch".to_string());
            }
            Err(e) => errors.push(e.to_string()),
        }
    }

    if errors.is_empty() {
        errors.push("LLM response could not be parsed as JSON".to_string());
    }

    Err(errors.join(" | "))
}

fn extract_json_payload(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}');
    match end {
        Some(end_idx) if end_idx >= start => Some(trimmed[start..=end_idx].to_string()),
        _ => Some(trimmed[start..].to_string()),
    }
}

fn repair_json_fragment(fragment: &str) -> Option<String> {
    let mut repaired = fragment.trim().to_string();
    if repaired.is_empty() {
        return None;
    }

    loop {
        let trimmed = repaired.trim_end();
        if trimmed.is_empty() {
            repaired.clear();
            break;
        }
        if trimmed.chars().last() == Some(',') {
            repaired.truncate(trimmed.len() - 1);
            continue;
        }
        if trimmed.len() != repaired.len() {
            repaired.truncate(trimmed.len());
        }
        break;
    }

    let mut stack: Vec<char> = Vec::new();
    let mut in_string = false;
    let mut escape = false;

    for ch in repaired.chars() {
        if in_string {
            if escape {
                escape = false;
                continue;
            }
            match ch {
                '\\' => escape = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => stack.push('{'),
            '[' => stack.push('['),
            '}' => {
                if matches!(stack.last(), Some('{')) {
                    stack.pop();
                }
            }
            ']' => {
                if matches!(stack.last(), Some('[')) {
                    stack.pop();
                }
            }
            _ => {}
        }
    }

    if in_string {
        repaired.push('"');
    }

    while let Some(open) = stack.pop() {
        match open {
            '{' => repaired.push('}'),
            '[' => repaired.push(']'),
            _ => {}
        }
    }

    Some(repaired)
}

fn coerce_payload_from_value(value: &Value) -> Option<LlmResponsePayload> {
    let obj = value.as_object()?;
    let summary = obj
        .get("summary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let weather_brief = obj.get("weather_brief").and_then(coerce_weather_brief);

    let mut ingredients: Vec<LlmIngredientPrediction> = Vec::new();
    if let Some(arr) = obj.get("ingredients").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(ingredient) = coerce_ingredient_prediction(item) {
                ingredients.push(ingredient);
            }
        }
    }

    Some(LlmResponsePayload {
        summary,
        weather_brief,
        ingredients,
    })
}

fn coerce_weather_brief(value: &Value) -> Option<LlmWeatherBrief> {
    let obj = value.as_object()?;
    let key_impacts = match obj.get("key_weather_impacts") {
        Some(Value::Array(items)) => {
            let collected: Vec<String> = items
                .iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect();
            if collected.is_empty() {
                None
            } else {
                Some(collected)
            }
        }
        Some(Value::String(s)) if !s.trim().is_empty() => Some(vec![s.trim().to_string()]),
        _ => None,
    };

    Some(LlmWeatherBrief {
        summary: obj
            .get("summary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        key_weather_impacts: key_impacts,
        temp_min_c: obj.get("temp_min_c").and_then(|v| value_to_f32(v)),
        temp_max_c: obj.get("temp_max_c").and_then(|v| value_to_f32(v)),
        humidity_avg: obj.get("humidity_avg").and_then(|v| value_to_f32(v)),
        precipitation_total_mm: obj
            .get("precipitation_total_mm")
            .and_then(|v| value_to_f32(v)),
    })
}

fn coerce_ingredient_prediction(value: &Value) -> Option<LlmIngredientPrediction> {
    let obj = value.as_object()?;
    let uuid_value = obj.get("ingredient_catalog_uuid")?;
    let uuid_str = match uuid_value {
        Value::String(s) => s.trim(),
        _ => return None,
    };
    let ingredient_catalog_uuid = Uuid::parse_str(uuid_str).ok()?;

    let restock_label = obj
        .get("restock_label")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "MEDIUM".to_string());

    let restock_probability = obj.get("restock_probability").and_then(|v| value_to_f32(v));

    let recommended_restock_qty = obj
        .get("recommended_restock_qty")
        .and_then(|v| value_to_f64(v));

    let current_stock_qty = obj.get("current_stock_qty").and_then(|v| value_to_f64(v));

    let forecast_error_margin_pct = obj
        .get("forecast_error_margin_pct")
        .or_else(|| obj.get("expected_error_pct"))
        .or_else(|| obj.get("confidence_interval_pct"))
        .and_then(|v| value_to_f32(v));

    let reasoning = obj
        .get("reasoning")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(LlmIngredientPrediction {
        ingredient_catalog_uuid,
        restock_label,
        restock_probability,
        recommended_restock_qty,
        current_stock_qty,
        reasoning,
        forecast_error_margin_pct,
    })
}

fn extract_forecast_error_margin(
    prediction: &StoreIngredientPredictionWithIngredient,
) -> Option<f32> {
    let response = prediction.llm_response.as_ref()?;
    let ingredients = response.get("ingredients")?.as_array()?;
    let target_uuid = prediction.ingredient_catalog_uuid.to_string();

    for entry in ingredients {
        let entry_uuid = entry
            .get("ingredient_catalog_uuid")
            .and_then(|v| v.as_str())
            .map(|s| s.trim());

        if let Some(entry_uuid) = entry_uuid {
            if entry_uuid.eq_ignore_ascii_case(&target_uuid) {
                for key in [
                    "forecast_error_margin_pct",
                    "expected_error_pct",
                    "confidence_interval_pct",
                ] {
                    if let Some(value) = entry.get(key) {
                        if let Some(parsed) = value_to_f32(value) {
                            return Some(parsed);
                        }
                    }
                }
            }
        }
    }

    None
}

fn fallback_error_margin(probability: Option<f32>) -> Option<f32> {
    probability.map(|prob| {
        let prob = prob.clamp(0.0, 1.0);
        let margin = (1.0 - prob) * 100.0;
        margin.clamp(5.0, 60.0)
    })
}

fn value_to_f32(value: &Value) -> Option<f32> {
    value_to_f64(value).map(|v| v as f32)
}

fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(num) => num.as_f64(),
        Value::String(s) => {
            let cleaned = s.trim().trim_end_matches('%').trim();
            if cleaned.is_empty() {
                None
            } else {
                cleaned.replace(',', ".").parse::<f64>().ok()
            }
        }
        Value::Bool(true) => Some(1.0),
        Value::Bool(false) => Some(0.0),
        _ => None,
    }
}

fn json_error(code: i32, message: String, detail: String) -> Value {
    json!({
        "code": code,
        "status": "error",
        "message": message,
        "data": {},
        "errors": { "detail": detail }
    })
}

fn internal_error<E: std::fmt::Display>(error: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json_error(
            500,
            "Internal server error".to_string(),
            format!("{error}"),
        )),
    )
}
