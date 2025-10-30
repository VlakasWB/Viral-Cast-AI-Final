use std::collections::HashMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde_json::Value;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

use crate::models::store_product_predictions::{
    ProductSnapshot, StoreProductPredictionWithProduct, WeatherSlotSnapshot,
};

#[derive(Debug, Clone)]
pub struct RegionContext {
    pub region_code: String,
    pub timezone: Option<String>,
    pub provinsi: Option<String>,
    pub kotkab: Option<String>,
    pub kecamatan: Option<String>,
    pub desa: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewStoreProductPrediction {
    pub store_uuid: Uuid,
    pub product_uuid: Uuid,
    pub region_code: Option<String>,
    pub demand_label: String,
    pub demand_probability: Option<f32>,
    pub recommended_stock_qty: Decimal,
    pub weather_summary: Option<String>,
    pub weather_temp_min_c: Option<f32>,
    pub weather_temp_max_c: Option<f32>,
    pub weather_precip_mm: Option<f32>,
    pub weather_humidity: Option<f32>,
    pub llm_reasoning: Option<String>,
    pub llm_model: Option<String>,
    pub llm_prompt: Option<Value>,
    pub llm_response: Option<Value>,
}

pub async fn resolve_region_context(
    db: &Pool<Postgres>,
    village_code: Option<&str>,
    district_code: Option<&str>,
    regency_code: Option<&str>,
) -> Result<Option<RegionContext>, sqlx::Error> {
    if let Some(code) = village_code {
        let normalized = code.replace('.', "");
        let query = r#"
            SELECT region_code, timezone, provinsi, kotkab, kecamatan, desa
            FROM bmkg_area
            WHERE deleted_at = 0
              AND (
                region_code = $1
                OR REPLACE(region_code, '.', '') = $2
              )
            ORDER BY region_code
            LIMIT 1
        "#;
        if let Some(row) = sqlx::query(query)
            .bind(code)
            .bind(&normalized)
            .fetch_optional(db)
            .await?
        {
            return Ok(Some(RegionContext {
                region_code: row.try_get::<String, _>("region_code")?,
                timezone: row.try_get::<Option<String>, _>("timezone")?,
                provinsi: row.try_get::<Option<String>, _>("provinsi")?,
                kotkab: row.try_get::<Option<String>, _>("kotkab")?,
                kecamatan: row.try_get::<Option<String>, _>("kecamatan")?,
                desa: row.try_get::<Option<String>, _>("desa")?,
            }));
        }
    }

    if let (Some(district), Some(regency)) = (district_code, regency_code) {
        let query = r#"
            SELECT region_code, timezone, provinsi, kotkab, kecamatan, desa
            FROM bmkg_area
            WHERE deleted_at = 0
              AND adm3 = $1
              AND adm2 = $2
            ORDER BY region_code
            LIMIT 1
        "#;
        if let Some(row) = sqlx::query(query)
            .bind(district)
            .bind(regency)
            .fetch_optional(db)
            .await?
        {
            return Ok(Some(RegionContext {
                region_code: row.try_get::<String, _>("region_code")?,
                timezone: row.try_get::<Option<String>, _>("timezone")?,
                provinsi: row.try_get::<Option<String>, _>("provinsi")?,
                kotkab: row.try_get::<Option<String>, _>("kotkab")?,
                kecamatan: row.try_get::<Option<String>, _>("kecamatan")?,
                desa: row.try_get::<Option<String>, _>("desa")?,
            }));
        }
    }

    Ok(None)
}

pub async fn fetch_active_products(
    db: &Pool<Postgres>,
    limit: Option<i64>,
) -> Result<Vec<ProductSnapshot>, sqlx::Error> {
    let limit = limit.unwrap_or(50).max(1);
    let rows = sqlx::query(
        r#"
        SELECT uuid, name, sku, price
        FROM products
        WHERE deleted_at = 0
          AND status = 'ACTIVE'
        ORDER BY updated_at DESC, name ASC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let price = row.try_get::<Decimal, _>("price").ok()?;
            Some(ProductSnapshot {
                uuid: row.try_get("uuid").ok()?,
                name: row.try_get::<String, _>("name").ok()?,
                sku: row.try_get("sku").ok(),
                price,
            })
        })
        .collect())
}

pub async fn weather_slots_for_date(
    db: &Pool<Postgres>,
    region_code: &str,
    target_date: NaiveDate,
) -> Result<Vec<WeatherSlotSnapshot>, sqlx::Error> {
    let query = r#"
        WITH latest AS (
            SELECT MAX(analysis_ms) AS analysis_ms
            FROM bmkg_forecast
            WHERE region_code = $1
              AND deleted_at = 0
              AND valid_ts::date = $2
        )
        SELECT
            bf.valid_ms,
            bf.time_index,
            bf.t,
            bf.hu,
            bf.tp_mm,
            bf.ws,
            bf.weather_code,
            bf.weather_desc_id,
            bf.weather_desc_en
        FROM bmkg_forecast bf
        CROSS JOIN latest
        WHERE bf.region_code = $1
          AND bf.deleted_at = 0
          AND bf.valid_ts::date = $2
          AND (latest.analysis_ms IS NULL OR bf.analysis_ms = latest.analysis_ms)
        ORDER BY bf.valid_ms ASC
    "#;

    let rows = sqlx::query(query)
        .bind(region_code)
        .bind(target_date)
        .fetch_all(db)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| WeatherSlotSnapshot {
            valid_ms: row.try_get::<i64, _>("valid_ms").unwrap_or_default(),
            time_index: row
                .try_get::<Option<String>, _>("time_index")
                .ok()
                .flatten(),
            temperature_c: row.try_get::<Option<f32>, _>("t").ok().flatten(),
            humidity_pct: row.try_get::<Option<f32>, _>("hu").ok().flatten(),
            precipitation_mm: row.try_get::<Option<f32>, _>("tp_mm").ok().flatten(),
            wind_speed_kmh: row.try_get::<Option<f32>, _>("ws").ok().flatten(),
            weather_code: row.try_get::<Option<i16>, _>("weather_code").ok().flatten(),
            weather_desc_id: row
                .try_get::<Option<String>, _>("weather_desc_id")
                .ok()
                .flatten(),
            weather_desc_en: row
                .try_get::<Option<String>, _>("weather_desc_en")
                .ok()
                .flatten(),
        })
        .collect())
}

pub async fn upsert_predictions(
    db: &Pool<Postgres>,
    items: &[NewStoreProductPrediction],
) -> Result<(), sqlx::Error> {
    if items.is_empty() {
        return Ok(());
    }

    let mut tx = db.begin().await?;
    for item in items {
        sqlx::query(
            r#"
            INSERT INTO store_product_predictions (
                store_uuid,
                product_uuid,
                region_code,
                demand_label,
                demand_probability,
                recommended_stock_qty,
                weather_summary,
                weather_temp_min_c,
                weather_temp_max_c,
                weather_precip_mm,
                weather_humidity,
                llm_reasoning,
                llm_model,
                llm_prompt,
                llm_response,
                created_at,
                updated_at,
                deleted_at
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,
                (EXTRACT(EPOCH FROM NOW()) * 1000)::bigint,
                (EXTRACT(EPOCH FROM NOW()) * 1000)::bigint,
                0
            )
            ON CONFLICT (store_uuid, product_uuid)
            DO UPDATE SET
                region_code = EXCLUDED.region_code,
                demand_label = EXCLUDED.demand_label,
                demand_probability = EXCLUDED.demand_probability,
                recommended_stock_qty = EXCLUDED.recommended_stock_qty,
                weather_summary = EXCLUDED.weather_summary,
                weather_temp_min_c = EXCLUDED.weather_temp_min_c,
                weather_temp_max_c = EXCLUDED.weather_temp_max_c,
                weather_precip_mm = EXCLUDED.weather_precip_mm,
                weather_humidity = EXCLUDED.weather_humidity,
                llm_reasoning = EXCLUDED.llm_reasoning,
                llm_model = EXCLUDED.llm_model,
                llm_prompt = EXCLUDED.llm_prompt,
                llm_response = EXCLUDED.llm_response,
                updated_at = (EXTRACT(EPOCH FROM NOW()) * 1000)::bigint,
                deleted_at = 0
            "#,
        )
        .bind(item.store_uuid)
        .bind(item.product_uuid)
        .bind(item.region_code.clone())
        .bind(&item.demand_label)
        .bind(item.demand_probability)
        .bind(item.recommended_stock_qty)
        .bind(item.weather_summary.clone())
        .bind(item.weather_temp_min_c)
        .bind(item.weather_temp_max_c)
        .bind(item.weather_precip_mm)
        .bind(item.weather_humidity)
        .bind(item.llm_reasoning.clone())
        .bind(item.llm_model.clone())
        .bind(item.llm_prompt.clone())
        .bind(item.llm_response.clone())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn get_predictions_for_store(
    db: &Pool<Postgres>,
    store_uuid: Uuid,
) -> Result<Vec<StoreProductPredictionWithProduct>, sqlx::Error> {
    let query = r#"
        SELECT
            spp.uuid,
            spp.store_uuid,
            spp.product_uuid,
            spp.region_code,
            spp.demand_label,
            spp.demand_probability,
            spp.recommended_stock_qty,
            spp.weather_summary,
            spp.weather_temp_min_c,
            spp.weather_temp_max_c,
            spp.weather_precip_mm,
            spp.weather_humidity,
            spp.llm_reasoning,
            spp.llm_model,
            spp.llm_prompt,
            spp.llm_response,
            spp.created_at,
            spp.updated_at,
            spp.deleted_at,
            p.name AS product_name,
            p.sku AS product_sku,
            p.price AS product_price
        FROM store_product_predictions spp
        JOIN products p ON p.uuid = spp.product_uuid
        WHERE spp.store_uuid = $1
          AND spp.deleted_at = 0
          AND p.deleted_at = 0
        ORDER BY p.name ASC
    "#;

    sqlx::query_as::<_, StoreProductPredictionWithProduct>(query)
        .bind(store_uuid)
        .fetch_all(db)
        .await
}

pub fn aggregate_prediction_fields(
    predictions: &[StoreProductPredictionWithProduct],
) -> HashMap<String, Option<f32>> {
    let mut min_temp: Option<f32> = None;
    let mut max_temp: Option<f32> = None;
    let mut humidity_acc = 0.0f32;
    let mut humidity_count = 0f32;
    let mut precip_acc = 0.0f32;

    for p in predictions {
        if let Some(v) = p.weather_temp_min_c {
            min_temp = Some(min_temp.map(|c| c.min(v)).unwrap_or(v));
        }
        if let Some(v) = p.weather_temp_max_c {
            max_temp = Some(max_temp.map(|c| c.max(v)).unwrap_or(v));
        }
        if let Some(v) = p.weather_humidity {
            humidity_acc += v;
            humidity_count += 1.0;
        }
        if let Some(v) = p.weather_precip_mm {
            precip_acc += v;
        }
    }

    let humidity_avg = if humidity_count > 0.0 {
        Some(humidity_acc / humidity_count)
    } else {
        None
    };

    let mut map = HashMap::new();
    map.insert("temp_min_c".to_string(), min_temp);
    map.insert("temp_max_c".to_string(), max_temp);
    map.insert(
        "humidity_avg".to_string(),
        humidity_avg.map(|v| (v * 10.0).round() / 10.0),
    );
    map.insert(
        "precip_total_mm".to_string(),
        if precip_acc > 0.0 {
            Some((precip_acc * 10.0).round() / 10.0)
        } else {
            None
        },
    );
    map
}
