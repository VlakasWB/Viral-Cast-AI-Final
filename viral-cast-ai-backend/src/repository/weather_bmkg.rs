use chrono::Utc;
use serde_json::Value;
use sqlx::{Pool, Postgres};

use crate::dto::weather_bmkg::RegionListRequest;
use crate::models::weather_bmkg::{BMKGWeatherResponse, RegionMaster, WeatherPrediction};

fn parse_iso_to_ms(iso: &str) -> Option<i64> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        return Some(dt.timestamp_millis());
    }
    chrono::DateTime::parse_from_str(iso, "%Y-%m-%d %H:%M:%S%z")
        .ok()
        .map(|dt| dt.timestamp_millis())
}

pub async fn get_prioritized_regions(db: &Pool<Postgres>) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT region_code FROM bmkg_area_priority
        WHERE active = true AND deleted_at = 0
        ORDER BY priority ASC, COALESCE(next_due_ms, 0) ASC
        "#,
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn get_latest_raw_forecast_run_json(
    db: &Pool<Postgres>,
    region_code: &str,
) -> Result<Option<Value>, sqlx::Error> {
    let latest_query = r#"
        SELECT 
          fr.raw_json
        FROM bmkg_forecast_run fr
        WHERE fr.region_code = $1 AND fr.deleted_at = 0
        ORDER BY fr.analysis_ms DESC
        LIMIT 1
    "#;
    let opt_row = sqlx::query_scalar::<_, Value>(latest_query)
        .bind(region_code)
        .fetch_optional(db)
        .await?;
    Ok(opt_row)
}

pub async fn insert_forecast_run(
    db: &Pool<Postgres>,
    region_code: &str,
    analysis_ms: i64,
    raw_json: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO bmkg_forecast_run(region_code, analysis_ms, raw_json, source) VALUES ($1, $2, $3, 'bmkg') ON CONFLICT DO NOTHING",
    )
    .bind(region_code)
    .bind(analysis_ms)
    .bind(raw_json)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn insert_forecast(
    db: &Pool<Postgres>,
    region_code: &str,
    analysis_ms: i64,
    pred: &WeatherPrediction,
) -> Result<(), sqlx::Error> {
    let valid_ms = parse_iso_to_ms(&pred.utc_datetime)
        .or_else(|| parse_iso_to_ms(&pred.datetime))
        .unwrap_or(Utc::now().timestamp_millis());

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
    .bind(analysis_ms)
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
    .execute(db)
    .await?;

    Ok(())
}

pub async fn list_regions(
    db: &Pool<Postgres>,
    params: &RegionListRequest,
    limit: i32,
    offset: i32,
) -> Result<Vec<RegionMaster>, sqlx::Error> {
    let mut base_from_where = r#"
        FROM village v
        JOIN district d ON v.district_uuid = d.uuid
        JOIN regency r ON d.regency_uuid = r.uuid
        JOIN province p ON r.province_uuid = p.uuid
        WHERE 1=1
    "#
    .to_string();

    base_from_where.push_str(
        " AND v.deleted_at = 0 AND d.deleted_at = 0 AND r.deleted_at = 0 AND p.deleted_at = 0",
    );
    let mut conditions = Vec::new();

    if let Some(provinsi) = &params.provinsi {
        conditions.push(format!("p.name ILIKE '%{}%'", provinsi));
    }
    if let Some(kabupaten_kota) = &params.kabupaten_kota {
        conditions.push(format!("r.name ILIKE '%{}%'", kabupaten_kota));
    }
    if let Some(kecamatan) = &params.kecamatan {
        conditions.push(format!("d.name ILIKE '%{}%'", kecamatan));
    }
    if let Some(search) = &params.search {
        conditions.push(format!(
            "(p.name ILIKE '%{}%' OR r.name ILIKE '%{}%' OR d.name ILIKE '%{}%' OR v.name ILIKE '%{}%')",
            search, search, search, search
        ));
    }

    if !conditions.is_empty() {
        base_from_where.push_str(&format!(" AND {}", conditions.join(" AND ")));
    }

    let mut select_query = format!(
        r#"SELECT 
            ROW_NUMBER() OVER (ORDER BY p.name, r.name, d.name, v.name)::int as id,
            v.code as kode_wilayah,
            p.name as provinsi,
            r.name as kabupaten_kota,
            d.name as kecamatan,
            v.name as kelurahan_desa,
            r.code as kotkab,
            NULL::double precision as latitude,
            NULL::double precision as longitude,
            NULL::text as timezone,
            NULL::timestamptz as created_at,
            NULL::timestamptz as updated_at
        {}"#,
        base_from_where
    );

    select_query.push_str(&format!(
        " ORDER BY p.name, r.name, d.name, v.name LIMIT {} OFFSET {}",
        limit, offset
    ));

    let rows = sqlx::query_as::<_, RegionMaster>(&select_query)
        .fetch_all(db)
        .await?;
    Ok(rows)
}

pub async fn count_regions(
    db: &Pool<Postgres>,
    params: &RegionListRequest,
) -> Result<i64, sqlx::Error> {
    let mut base_from_where = r#"
        FROM village v
        JOIN district d ON v.district_uuid = d.uuid
        JOIN regency r ON d.regency_uuid = r.uuid
        JOIN province p ON r.province_uuid = p.uuid
        WHERE 1=1
    "#
    .to_string();

    base_from_where.push_str(
        " AND v.deleted_at = 0 AND d.deleted_at = 0 AND r.deleted_at = 0 AND p.deleted_at = 0",
    );
    let mut conditions = Vec::new();

    if let Some(provinsi) = &params.provinsi {
        conditions.push(format!("p.name ILIKE '%{}%'", provinsi));
    }
    if let Some(kabupaten_kota) = &params.kabupaten_kota {
        conditions.push(format!("r.name ILIKE '%{}%'", kabupaten_kota));
    }
    if let Some(kecamatan) = &params.kecamatan {
        conditions.push(format!("d.name ILIKE '%{}%'", kecamatan));
    }
    if let Some(search) = &params.search {
        conditions.push(format!(
            "(p.name ILIKE '%{}%' OR r.name ILIKE '%{}%' OR d.name ILIKE '%{}%' OR v.name ILIKE '%{}%')",
            search, search, search, search
        ));
    }

    if !conditions.is_empty() {
        base_from_where.push_str(&format!(" AND {}", conditions.join(" AND ")));
    }

    let count_query = format!("SELECT COUNT(*) {}", base_from_where);
    let total_count: i64 = sqlx::query_scalar(&count_query).fetch_one(db).await?;
    Ok(total_count)
}

pub async fn get_region_by_offset(
    db: &Pool<Postgres>,
    offset: i32,
) -> Result<Option<RegionMaster>, sqlx::Error> {
    let query = r#"
        SELECT 
            ROW_NUMBER() OVER (ORDER BY p.name, r.name, d.name, v.name)::int as id,
            v.code as kode_wilayah,
            p.name as provinsi,
            r.name as kabupaten_kota,
            d.name as kecamatan,
            v.name as kelurahan_desa,
            r.code as kotkab,
            NULL::double precision as latitude,
            NULL::double precision as longitude,
            NULL::text as timezone,
            NULL::timestamptz as created_at,
            NULL::timestamptz as updated_at
        FROM village v
        JOIN district d ON v.district_uuid = d.uuid
        JOIN regency r ON d.regency_uuid = r.uuid
        JOIN province p ON r.province_uuid = p.uuid
        WHERE v.deleted_at = 0 AND d.deleted_at = 0 AND r.deleted_at = 0 AND p.deleted_at = 0
        ORDER BY p.name, r.name, d.name, v.name
        LIMIT 1 OFFSET $1
    "#;

    let res = sqlx::query_as::<_, RegionMaster>(query)
        .bind(offset)
        .fetch_optional(db)
        .await?;
    Ok(res)
}

pub async fn upsert_area_priority(
    db: &Pool<Postgres>,
    region_code: &str,
    priority: i32,
    active: Option<bool>,
    updated_at_ms: i64,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        "INSERT INTO bmkg_area_priority(region_code, priority, active, updated_at) VALUES ($1, $2, COALESCE($3, true), $4) ON CONFLICT (region_code) DO UPDATE SET priority = EXCLUDED.priority, active = COALESCE(EXCLUDED.active, bmkg_area_priority.active), updated_at = EXCLUDED.updated_at",
    )
    .bind(region_code)
    .bind(priority)
    .bind(active)
    .bind(updated_at_ms)
    .execute(db)
    .await?;
    Ok(res.rows_affected())
}
