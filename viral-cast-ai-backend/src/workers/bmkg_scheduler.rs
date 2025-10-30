use crate::AppState;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use redis::{aio::MultiplexedConnection, cmd, AsyncCommands};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use std::time::Duration;

// Scheduler configuration
// Jalankan tiap 72 detik: 50 hits/jam
const HITS_PER_HOUR: u32 = 50;
const INTERVAL_SECS: u64 = 3600 / HITS_PER_HOUR as u64; // 72 seconds
const DEDUP_WINDOW_MS: i64 = 2 * 24 * 60 * 60 * 1000; // 2 days
const BMKG_QUEUE_KEY: &str = "queue:bmkg:fetch";
const BMKG_INFLIGHT_PREFIX: &str = "queue:bmkg:fetch:inflight:";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BmkgFetchJob {
    region_code: String,
    origin: String, // scheduler|fallback
    enqueued_ms: i64,
}

pub async fn start_bmkg_scheduler(state: std::sync::Arc<AppState>) {
    let interval = tokio::time::interval(Duration::from_secs(INTERVAL_SECS));
    tokio::pin!(interval);

    // Run immediately once
    // Seed prioritas berdasarkan jam toko (opening/middle/closing) sebelum loop berjalan
    if let Err(e) = seed_store_region_priorities(&state).await {
        tracing::warn!(error = %e, "[bmkg_scheduler] seed_store_region_priorities error, continue without strict schedule");
    }
    if let Err(e) = run_once(&state).await {
        tracing::warn!(error = %e, "[bmkg_scheduler] initial run_once error, skipped");
    }

    loop {
        interval.as_mut().tick().await;

        if let Err(e) = run_once(&state).await {
            tracing::warn!(error = %e, "[bmkg_scheduler] run_once error, skipped");
        }
    }
}

/// Start BMKG queue workers that consume Redis queue and process jobs concurrently.
pub async fn start_bmkg_queue_workers(state: Arc<AppState>, concurrency: usize) {
    for i in 0..concurrency {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                // Try to get a redis connection; if it fails, backoff briefly and retry
                let conn_res = st.redis_client.get_multiplexed_async_connection().await;
                let mut conn: MultiplexedConnection = match conn_res {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!(worker = i, error = %e, "[bmkg_queue_worker] Redis connection failed; retrying");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                // BLPOP blocks until a job is available
                let popped: redis::RedisResult<Option<(String, String)>> =
                    conn.blpop(BMKG_QUEUE_KEY, 0.0).await;
                let opt = match popped {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!(worker = i, error = %e, "[bmkg_queue_worker] BLPOP failed; reconnecting");
                        continue;
                    }
                };

                if let Some((_key, payload)) = opt {
                    let job: Result<BmkgFetchJob, _> = serde_json::from_str(&payload);
                    match job {
                        Ok(job) => {
                            let rc = job.region_code.clone();
                            let now_ms = Utc::now().timestamp_millis();
                            match fetch_and_persist(&st, &rc).await {
                                Ok(_) => {
                                    // Compute next_due and update priority metadata, mirroring scheduler logic
                                    if let Ok(Some(times)) =
                                        fetch_store_times_for_region(&st, &rc).await
                                    {
                                        let next_due_ms = compute_next_due_ms(&times, now_ms)
                                            .unwrap_or(now_ms + (INTERVAL_SECS as i64) * 1000);
                                        if let Err(e) = sqlx::query("UPDATE bmkg_area_priority SET last_hit_ms = $1, next_due_ms = $2, updated_at = $3 WHERE region_code = $4")
                                            .bind(now_ms)
                                            .bind(next_due_ms)
                                            .bind(now_ms)
                                            .bind(&rc)
                                            .execute(&st.db)
                                            .await
                                        {
                                            tracing::warn!(region = %rc, error = %e, "[bmkg_queue_worker] Failed to update priority on success");
                                        }
                                    } else {
                                        let active_count: i64 = sqlx::query_scalar(
                                            "SELECT COUNT(*)::bigint FROM bmkg_area_priority WHERE active = true AND deleted_at = 0"
                                        )
                                        .fetch_one(&st.db)
                                        .await
                                        .unwrap_or(1);
                                        let cycle_ms =
                                            (INTERVAL_SECS as i64) * 1000 * active_count.max(1);
                                        let next_due = now_ms + cycle_ms;
                                        if let Err(e) = sqlx::query("UPDATE bmkg_area_priority SET last_hit_ms = $1, next_due_ms = $2, updated_at = $3 WHERE region_code = $4")
                                            .bind(now_ms)
                                            .bind(next_due)
                                            .bind(now_ms)
                                            .bind(&rc)
                                            .execute(&st.db)
                                            .await
                                        {
                                            tracing::warn!(region = %rc, error = %e, "[bmkg_queue_worker] Failed to update cycle next_due on success");
                                        }
                                    }

                                    // Clear inflight key
                                    let inflight_key = format!("{}{}", BMKG_INFLIGHT_PREFIX, rc);
                                    let _ = conn.del::<_, ()>(&inflight_key).await;
                                }
                                Err(e) => {
                                    // On failure, delay retry by 24 hours
                                    let delay_hours: i64 = 24;
                                    let delay_ms =
                                        ChronoDuration::hours(delay_hours).num_milliseconds();
                                    let next_due = now_ms + delay_ms;
                                    tracing::warn!(
                                        worker = i,
                                        region = %rc,
                                        error = %e,
                                        delay_hours = delay_hours,
                                        next_due_ms = next_due,
                                        "[bmkg_queue_worker] fetch_and_persist failed, delayed"
                                    );
                                    drop(e);
                                    if let Err(e2) = sqlx::query("UPDATE bmkg_area_priority SET next_due_ms = $1, updated_at = $2 WHERE region_code = $3")
                                        .bind(next_due)
                                        .bind(now_ms)
                                        .bind(&rc)
                                        .execute(&st.db)
                                        .await
                                    {
                                        tracing::warn!(region = %rc, error = %e2, "[bmkg_queue_worker] Failed to set next_due on error");
                                    }

                                    // Clear inflight to allow rescheduling in the future window
                                    let inflight_key = format!("{}{}", BMKG_INFLIGHT_PREFIX, rc);
                                    let _ = conn.del::<_, ()>(&inflight_key).await;
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(worker = i, error = %e, payload = %payload, "[bmkg_queue_worker] Invalid job payload");
                        }
                    }
                }
            }
        });
    }
}

async fn run_once(state: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Pilih region terdekat untuk dieksekusi:
    // - Utamakan priority paling kecil
    // - Dalam priority yang sama, pilih yang paling lama belum di-hit (last_hit_ms tertua)
    // - Tetap hormati next_due_ms agar yang belum jatuh tempo tidak dipilih
    let now_ms: i64 = (Utc::now().timestamp_millis()) as i64;

    let query = r#"
        SELECT region_code
        FROM bmkg_area_priority
        WHERE active = true AND deleted_at = 0
          AND (
            (next_due_ms IS NULL OR next_due_ms <= $1)
            AND
            (last_hit_ms IS NULL OR last_hit_ms <= ($1 - $2))
          )
        ORDER BY priority ASC, COALESCE(last_hit_ms, 0) ASC
        LIMIT 1
    "#;

    let region_code: Option<String> = sqlx::query_scalar::<_, String>(query)
        .bind(now_ms)
        .bind(DEDUP_WINDOW_MS)
        .fetch_optional(&state.db)
        .await?;

    if let Some(region_code) = region_code {
        // Enqueue job to Redis with inflight dedupe; fallback to direct execute when Redis fails
        let enq_res = enqueue_bmkg_job(state, &region_code, "scheduler").await;
        if let Err(e) = enq_res {
            tracing::warn!(region = %region_code, error = %e, "[bmkg_scheduler] enqueue failed; falling back to direct fetch");
            // Fallback: do the work inline to avoid missed runs
            match fetch_and_persist(state, &region_code).await {
                Ok(_) => {
                    // Mirror the priority update logic used by worker
                    if let Some(times) = fetch_store_times_for_region(state, &region_code).await? {
                        let next_due_ms = compute_next_due_ms(&times, now_ms)
                            .unwrap_or(now_ms + (INTERVAL_SECS as i64) * 1000);
                        sqlx::query("UPDATE bmkg_area_priority SET last_hit_ms = $1, next_due_ms = $2, updated_at = $3 WHERE region_code = $4")
                            .bind(now_ms)
                            .bind(next_due_ms)
                            .bind(now_ms)
                            .bind(&region_code)
                            .execute(&state.db)
                            .await?;
                    } else {
                        let active_count: i64 = sqlx::query_scalar(
                            "SELECT COUNT(*)::bigint FROM bmkg_area_priority WHERE active = true AND deleted_at = 0"
                        )
                        .fetch_one(&state.db)
                        .await
                        .unwrap_or(1);
                        let cycle_ms = (INTERVAL_SECS as i64) * 1000 * active_count.max(1);
                        let next_due = now_ms + cycle_ms;
                        sqlx::query("UPDATE bmkg_area_priority SET last_hit_ms = $1, next_due_ms = $2, updated_at = $3 WHERE region_code = $4")
                            .bind(now_ms)
                            .bind(next_due)
                            .bind(now_ms)
                            .bind(&region_code)
                            .execute(&state.db)
                            .await?;
                    }
                }
                Err(e) => {
                    let delay_hours: i64 = 24;
                    let delay_ms = ChronoDuration::hours(delay_hours).num_milliseconds();
                    let next_due = now_ms + delay_ms;
                    tracing::warn!(
                        region = %region_code,
                        error = %e,
                        delay_hours = delay_hours,
                        next_due_ms = next_due,
                        "[bmkg_scheduler] direct fetch_and_persist error; delayed"
                    );
                    drop(e);
                    sqlx::query("UPDATE bmkg_area_priority SET next_due_ms = $1, updated_at = $2 WHERE region_code = $3")
                        .bind(next_due)
                        .bind(now_ms)
                        .bind(&region_code)
                        .execute(&state.db)
                        .await?;
                }
            }
        }
    } else {
        // Fallback: choose any area not fetched within 2 days
        let fallback_query = r#"
            SELECT a.region_code
            FROM bmkg_area a
            LEFT JOIN bmkg_forecast_run fr
              ON fr.region_code = a.region_code
              AND fr.analysis_ms > ($1 - $2)
            WHERE a.deleted_at = 0
            GROUP BY a.region_code
            HAVING COUNT(fr.region_code) = 0
            ORDER BY a.region_code ASC
            LIMIT 1
        "#;

        if let Some(rc) = sqlx::query_scalar::<_, String>(fallback_query)
            .bind(now_ms)
            .bind(DEDUP_WINDOW_MS)
            .fetch_optional(&state.db)
            .await?
        {
            // Enqueue fallback job instead of executing inline
            if let Err(e) = enqueue_bmkg_job(state, &rc, "fallback").await {
                tracing::warn!(region = %rc, error = %e, "[bmkg_scheduler] enqueue fallback failed; trying inline");
                // Inline fallback when Redis not available
                match fetch_and_persist(state, &rc).await {
                    Ok(_) => {
                        let next_due = now_ms + (INTERVAL_SECS as i64) * 1000;
                        sqlx::query("INSERT INTO bmkg_area_priority(region_code, priority, active, last_hit_ms, next_due_ms, updated_at) VALUES ($1, 100, true, $2, $3, $2) ON CONFLICT (region_code) DO UPDATE SET last_hit_ms = EXCLUDED.last_hit_ms, next_due_ms = EXCLUDED.next_due_ms, updated_at = EXCLUDED.updated_at")
                            .bind(&rc)
                            .bind(now_ms)
                            .bind(next_due)
                            .execute(&state.db)
                            .await?;
                    }
                    Err(e) => {
                        let delay_hours: i64 = 24;
                        let delay_ms = ChronoDuration::hours(delay_hours).num_milliseconds();
                        let next_due = now_ms + delay_ms;
                        tracing::warn!(
                            region = %rc,
                            error = %e,
                            delay_hours = delay_hours,
                            next_due_ms = next_due,
                            "[bmkg_scheduler] inline fetch_and_persist error (fallback), delayed"
                        );
                        drop(e);
                        sqlx::query("INSERT INTO bmkg_area_priority(region_code, priority, active, next_due_ms, updated_at) VALUES ($1, 100, true, $2, $2) ON CONFLICT (region_code) DO UPDATE SET next_due_ms = EXCLUDED.next_due_ms, updated_at = EXCLUDED.updated_at")
                            .bind(&rc)
                            .bind(next_due)
                            .execute(&state.db)
                            .await?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Enqueue a BMKG fetch job with inflight dedupe. Returns error when Redis fails.
async fn enqueue_bmkg_job(
    state: &AppState,
    region_code: &str,
    origin: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await?;

    // Inflight key to prevent double-enqueue for the same region
    let inflight_key = format!("{}{}", BMKG_INFLIGHT_PREFIX, region_code);
    // SET inflight NX EX 600 seconds
    let set_res: Option<String> = cmd("SET")
        .arg(&inflight_key)
        .arg(1)
        .arg("NX")
        .arg("EX")
        .arg(600)
        .query_async(&mut conn)
        .await?;

    if set_res.is_none() {
        // Key already exists; skip enqueue to avoid duplicate processing
        tracing::debug!(region = %region_code, "[bmkg_scheduler] inflight dedupe hit; skipping enqueue");
        return Ok(());
    }

    let job = BmkgFetchJob {
        region_code: region_code.to_string(),
        origin: origin.to_string(),
        enqueued_ms: Utc::now().timestamp_millis(),
    };
    let payload = serde_json::to_string(&job)?;
    conn.rpush::<_, _, ()>(BMKG_QUEUE_KEY, payload).await?;
    Ok(())
}

async fn fetch_and_persist(
    state: &AppState,
    region_code: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Call existing BMKG handler function to fetch data
    // Reuse fetch_bmkg_weather_data via a small wrapper to avoid duplicating logic
    let weather = super_fetch_bmkg_weather_data(region_code).await?;

    // Determine analysis_ms from BMKG data production time (analysis_date fields) or use current
    // Based on WeatherPrediction.analysis_date
    let analysis_iso_opt = weather.data.first().and_then(|w| w.analysis_date.clone());
    let analysis_ms = analysis_iso_opt
        .as_deref()
        .and_then(|iso| parse_iso_to_ms(iso))
        .unwrap_or(Utc::now().timestamp_millis());

    // Insert into bmkg_forecast_run (raw JSON)
    let raw_json = serde_json::to_value(&weather)?;
    sqlx::query("INSERT INTO bmkg_forecast_run(region_code, analysis_ms, raw_json, source) VALUES ($1, $2, $3, 'bmkg') ON CONFLICT DO NOTHING")
        .bind(region_code)
        .bind(analysis_ms as i64)
        .bind(raw_json)
        .execute(&state.db)
        .await?;

    // Optionally save the raw JSON to disk for testing/inspection
    if state.bmkg_save_json {
        let dir = std::path::Path::new("data/json");
        if let Err(e) = std::fs::create_dir_all(dir) {
            tracing::warn!(error = %e, "[bmkg_scheduler] Failed to create data/json directory");
        }
        let fname = format!(
            "bmkg_run_{}_{}.json",
            region_code.replace('.', "-"),
            chrono::Utc::now().format("%Y%m%d%H%M%S")
        );
        let path = dir.join(fname);
        if let Ok(json_str) = serde_json::to_string_pretty(&weather) {
            if let Err(e) = std::fs::write(&path, json_str) {
                tracing::warn!(file = %path.display(), error = %e, "[bmkg_scheduler] Failed to write JSON dump");
            }
        }
    }

    // Insert flattened predictions into bmkg_forecast
    for pred in weather.data.iter() {
        let valid_ms = parse_iso_to_ms(&pred.utc_datetime).unwrap_or_else(|| {
            parse_iso_to_ms(&pred.datetime).unwrap_or(Utc::now().timestamp_millis())
        });
        let extras = serde_json::json!({
            "wd_to": pred.wd_to,
            "vs_text": pred.vs_text,
            "time_index": pred.time_index,
            "image": pred.image,
        });

        sqlx::query(
            r#"
            INSERT INTO bmkg_forecast(
                region_code, analysis_ms, valid_ms,
                t, hu, weather_code, weather_desc_id, weather_desc_en,
                ws, wd, wd_deg, tcc, vs_m, tp_mm, time_index, image_url, extras
            ) VALUES (
                $1, $2, $3,
                $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14, $15, $16, $17
            ) ON CONFLICT DO NOTHING
        "#,
        )
        .bind(region_code)
        .bind(analysis_ms as i64)
        .bind(valid_ms as i64)
        .bind(pred.t as f32)
        .bind(pred.hu as f32)
        .bind(pred.weather as i16)
        .bind(pred.weather_desc.as_deref())
        .bind(pred.weather_desc_en.as_deref())
        .bind(pred.ws as f32)
        .bind(pred.wd.as_deref())
        .bind(pred.wd_deg as f32)
        .bind(pred.tcc as f32)
        .bind(pred.vs as f32)
        .bind(pred.tp as f32)
        .bind(pred.time_index.as_deref())
        .bind(pred.image.as_deref())
        .bind(extras)
        .execute(&state.db)
        .await?;
    }

    Ok(())
}

fn parse_iso_to_ms(iso: &str) -> Option<i64> {
    // Try RFC3339 then generic chrono parsing
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        return Some(dt.timestamp_millis());
    }
    chrono::DateTime::parse_from_str(iso, "%Y-%m-%d %H:%M:%S%z")
        .ok()
        .map(|dt| dt.timestamp_millis())
}

// Bring the BMKG fetch function from handlers to worker without changing visibility of handlers
async fn super_fetch_bmkg_weather_data(
    kode_wilayah: &str,
) -> Result<
    crate::models::weather_bmkg::BMKGWeatherResponse,
    Box<dyn std::error::Error + Send + Sync>,
> {
    use crate::models::weather_bmkg::{BMKGApiResponse, BMKGWeatherResponse};
    use reqwest;

    const BMKG_API_BASE_URL: &str = "https://api.bmkg.go.id/publik/prakiraan-cuaca";
    let client = reqwest::Client::new();
    let url = format!("{}?adm4={}", BMKG_API_BASE_URL, kode_wilayah);
    let response = client
        .get(&url)
        .header("User-Agent", "ViralCastAI/1.0")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("BMKG API returned status: {}", response.status()).into());
    }

    let bmkg_api_response: BMKGApiResponse = response.json().await?;
    if let Some(location_data) = bmkg_api_response.data.first() {
        let mut all_predictions = Vec::new();
        for day_predictions in &location_data.cuaca {
            for prediction in day_predictions {
                all_predictions.push(prediction.clone());
            }
        }
        Ok(BMKGWeatherResponse {
            data: all_predictions,
            lokasi: bmkg_api_response.lokasi,
        })
    } else {
        Err("No weather data found for the specified location".into())
    }
}

// --- New: helpers for store-time scheduling ---

/// Hitung next due dalam epoch ms berdasarkan daftar jam lokal dan timezone.
fn compute_next_due_ms(times_ms: &[i64], now_ms: i64) -> Option<i64> {
    if times_ms.is_empty() {
        return None;
    }
    times_ms.iter().copied().filter(|&t| t >= now_ms).min()
}

/// Ambil daftar jam toko (opening/middle_closing/closing) yang terpetakan ke satu region BMKG.
/// Mengembalikan (timezone, times) bila ada; None bila tidak ada toko yang memetakan ke region.
async fn fetch_store_times_for_region(
    state: &AppState,
    region_code: &str,
) -> Result<Option<Vec<i64>>, sqlx::Error> {
    // Pertama coba mapping via village_code = region_code
    let rows = sqlx::query(
        r#"
        SELECT s.opening_time, s.middle_closing_time, s.closing_time
        FROM stores s
        JOIN bmkg_area ba ON ba.region_code = s.village_code
        WHERE s.deleted_at = 0 AND ba.deleted_at = 0 AND ba.region_code = $1
          AND (s.opening_time IS NOT NULL OR s.middle_closing_time IS NOT NULL OR s.closing_time IS NOT NULL)
        "#,
    )
    .bind(region_code)
    .fetch_all(&state.db)
    .await?;

    let mut times: Vec<i64> = Vec::new();
    for row in rows.iter() {
        if let Ok(Some(t)) = row.try_get::<Option<i64>, _>("opening_time") {
            times.push(t);
        }
        if let Ok(Some(t)) = row.try_get::<Option<i64>, _>("middle_closing_time") {
            times.push(t);
        }
        if let Ok(Some(t)) = row.try_get::<Option<i64>, _>("closing_time") {
            times.push(t);
        }
    }

    if times.is_empty() {
        // Kedua, mapping via adm3+adm2 (district+regency) → pilih satu area pertama
        let rows2 = sqlx::query(
            r#"
            SELECT s.opening_time, s.middle_closing_time, s.closing_time
            FROM stores s
            JOIN LATERAL (
              SELECT region_code, timezone
              FROM bmkg_area
              WHERE deleted_at = 0 AND (adm3 = s.district_code) AND (adm2 = s.regency_code)
              ORDER BY region_code
              LIMIT 1
            ) ba ON TRUE
            WHERE s.deleted_at = 0 AND ba.region_code = $1
              AND (s.opening_time IS NOT NULL OR s.middle_closing_time IS NOT NULL OR s.closing_time IS NOT NULL)
            "#,
        )
        .bind(region_code)
        .fetch_all(&state.db)
        .await?;

        for row in rows2.iter() {
            if let Ok(Some(t)) = row.try_get::<Option<i64>, _>("opening_time") {
                times.push(t);
            }
            if let Ok(Some(t)) = row.try_get::<Option<i64>, _>("middle_closing_time") {
                times.push(t);
            }
            if let Ok(Some(t)) = row.try_get::<Option<i64>, _>("closing_time") {
                times.push(t);
            }
        }
    }

    if times.is_empty() {
        Ok(None)
    } else {
        // Hilangkan duplikat jam
        times.sort();
        times.dedup();
        Ok(Some(times))
    }
}

/// Seed/refresh bmkg_area_priority berdasarkan jam toko: region yang terdaftar di toko
/// diberi prioritas lebih tinggi dan next_due disetel ke waktu terdekat.
async fn seed_store_region_priorities(state: &AppState) -> Result<(), sqlx::Error> {
    // Ambil mapping region dari toko via union: village_code langsung, atau adm3+adm2 (pilih satu area)
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ba.region_code, s.opening_time, s.middle_closing_time, s.closing_time
        FROM stores s
        JOIN bmkg_area ba ON ba.region_code = s.village_code
        WHERE s.deleted_at = 0 AND ba.deleted_at = 0
          AND (s.opening_time IS NOT NULL OR s.middle_closing_time IS NOT NULL OR s.closing_time IS NOT NULL)
        UNION ALL
        SELECT DISTINCT ba2.region_code, s2.opening_time, s2.middle_closing_time, s2.closing_time
        FROM stores s2
        JOIN LATERAL (
          SELECT region_code, timezone
          FROM bmkg_area
          WHERE deleted_at = 0 AND (adm3 = s2.district_code) AND (adm2 = s2.regency_code)
          ORDER BY region_code
          LIMIT 1
        ) ba2 ON TRUE
        WHERE s2.deleted_at = 0
          AND (s2.opening_time IS NOT NULL OR s2.middle_closing_time IS NOT NULL OR s2.closing_time IS NOT NULL)
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    use std::collections::HashMap;
    let mut map: HashMap<String, Vec<i64>> = HashMap::new();
    for row in rows.into_iter() {
        let rc: String = row.try_get::<String, _>("region_code").unwrap_or_default();
        let entry = map.entry(rc).or_insert(Vec::new());
        if let Ok(Some(t)) = row.try_get::<Option<i64>, _>("opening_time") {
            entry.push(t);
        }
        if let Ok(Some(t)) = row.try_get::<Option<i64>, _>("middle_closing_time") {
            entry.push(t);
        }
        if let Ok(Some(t)) = row.try_get::<Option<i64>, _>("closing_time") {
            entry.push(t);
        }
    }

    let now_ms = Utc::now().timestamp_millis();
    for (rc, mut times) in map.into_iter() {
        times.sort();
        times.dedup();
        if let Some(next_due) = compute_next_due_ms(&times, now_ms) {
            // Upsert dengan priority tinggi (misal 1) dan aktifkan
            sqlx::query(
                "INSERT INTO bmkg_area_priority(region_code, priority, active, next_due_ms, updated_at) VALUES ($1, $2, true, $3, $4) ON CONFLICT (region_code) DO UPDATE SET priority = EXCLUDED.priority, active = true, next_due_ms = EXCLUDED.next_due_ms, updated_at = EXCLUDED.updated_at"
            )
            .bind(&rc)
            .bind(1_i32)
            .bind(next_due)
            .bind(now_ms)
            .execute(&state.db)
            .await?;
        } else {
            // Jika gagal hitung, tetap aktifkan dengan prioritas tinggi tanpa next_due khusus
            sqlx::query(
                "INSERT INTO bmkg_area_priority(region_code, priority, active, updated_at) VALUES ($1, $2, true, $3) ON CONFLICT (region_code) DO UPDATE SET priority = EXCLUDED.priority, active = true, updated_at = EXCLUDED.updated_at"
            )
            .bind(&rc)
            .bind(1_i32)
            .bind(now_ms)
            .execute(&state.db)
            .await?;
        }
    }

    Ok(())
}
