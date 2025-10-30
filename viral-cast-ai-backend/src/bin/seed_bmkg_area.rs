use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Row};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    seed_bmkg_area(&pool).await?;
    println!("✅ bmkg_area seeding aligned with master regions");
    Ok(())
}

fn tz_for_adm1(adm1: &str) -> &'static str {
    let code: i32 = adm1.parse().unwrap_or(31);
    match code {
        // WIT (UTC+9)
        81 | 82 | 91 | 92 => "Asia/Jayapura",
        // WITA (UTC+8)
        51 | 52 | 53 | 62 | 63 | 64 | 65 | 71 | 72 | 73 | 74 | 75 | 76 => "Asia/Makassar",
        // WIB (UTC+7)
        _ => "Asia/Jakarta",
    }
}

#[derive(Debug, Clone)]
struct AreaRow {
    adm1: String,
    adm2: String,
    adm3: String,
    adm4: String,
    provinsi: String,
    kotkab: String,
    kecamatan: String,
    desa: String,
}

async fn seed_bmkg_area(pool: &sqlx::PgPool) -> Result<()> {
    // Gather all villages with their hierarchy
    let raw_rows = sqlx::query(
        r#"
        SELECT 
          p.code AS adm1,
          r.code AS adm2,
          d.code AS adm3,
          v.code AS adm4,
          p.name AS provinsi,
          r.name AS kotkab,
          d.name AS kecamatan,
          v.name AS desa
        FROM village v
        JOIN district d ON v.district_uuid = d.uuid
        JOIN regency r ON d.regency_uuid = r.uuid
        JOIN province p ON r.province_uuid = p.uuid
        WHERE v.deleted_at = 0 AND d.deleted_at = 0 AND r.deleted_at = 0 AND p.deleted_at = 0
        ORDER BY p.code, r.code, d.code, v.code
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut rows: Vec<AreaRow> = Vec::with_capacity(raw_rows.len());
    for row in raw_rows.into_iter() {
        rows.push(AreaRow {
            adm1: row.try_get::<String, _>("adm1")?,
            adm2: row.try_get::<String, _>("adm2")?,
            adm3: row.try_get::<String, _>("adm3")?,
            adm4: row.try_get::<String, _>("adm4")?,
            provinsi: row.try_get::<String, _>("provinsi")?,
            kotkab: row.try_get::<String, _>("kotkab")?,
            kecamatan: row.try_get::<String, _>("kecamatan")?,
            desa: row.try_get::<String, _>("desa")?,
        });
    }

    let mut tx = pool.begin().await?;
    let now_ms: i64 = chrono::Utc::now().timestamp_millis();

    for row in rows {
        let tz = tz_for_adm1(&row.adm1);
        // Insert or update bmkg_area with available fields; leave lat/lon NULL
        sqlx::query!(
            r#"
            INSERT INTO bmkg_area (
                region_code, adm1, adm2, adm3, adm4,
                provinsi, kotkab, kecamatan, desa,
                timezone, lat, lon, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, NULL, NULL, $11, $11
            )
            ON CONFLICT (region_code) DO UPDATE SET
                adm1 = EXCLUDED.adm1,
                adm2 = EXCLUDED.adm2,
                adm3 = EXCLUDED.adm3,
                adm4 = EXCLUDED.adm4,
                provinsi = EXCLUDED.provinsi,
                kotkab = EXCLUDED.kotkab,
                kecamatan = EXCLUDED.kecamatan,
                desa = EXCLUDED.desa,
                timezone = EXCLUDED.timezone,
                updated_at = EXCLUDED.updated_at
            "#,
            row.adm4, // region_code uses village code (adm4)
            row.adm1,
            row.adm2,
            row.adm3,
            row.adm4,
            row.provinsi,
            row.kotkab,
            row.kecamatan,
            row.desa,
            tz,
            now_ms,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
