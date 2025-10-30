use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, env, fs::File, io::BufReader};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let csv_path = env::var("BMKG_CSV_PATH").unwrap_or_else(|_| "data/regions.csv".to_string());
    let priorities_csv_path =
        env::var("BMKG_PRIORITIES_CSV_PATH").unwrap_or_else(|_| "data/priorities.csv".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let seeded = seed_bmkg_area_from_csv(&pool, &csv_path).await?;
    println!("✅ bmkg_area seeded from CSV: {} rows", seeded);

    let prioritized = upsert_priorities_from_csv(&pool, &priorities_csv_path).await?;
    println!(
        "✅ bmkg_area_priority upserted from priorities CSV: {} rows",
        prioritized
    );
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

async fn seed_bmkg_area_from_csv(pool: &sqlx::PgPool, csv_path: &str) -> Result<i64> {
    let file = File::open(csv_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(BufReader::new(file));

    let mut adm1_names: HashMap<String, String> = HashMap::new();
    let mut adm2_names: HashMap<String, String> = HashMap::new();
    let mut adm3_names: HashMap<String, String> = HashMap::new();

    let mut tx = pool.begin().await?;
    let now_ms: i64 = chrono::Utc::now().timestamp_millis();
    let mut inserted: i64 = 0;

    for rec in rdr.records() {
        let rec = rec?;
        if rec.len() < 2 {
            continue;
        }
        let code = rec.get(0).unwrap().to_string();
        let name = rec.get(1).unwrap().to_string();

        // Classify by code depth
        let depth = code.matches('.').count();
        match depth {
            0 => {
                adm1_names.insert(code.clone(), name.clone());
            }
            1 => {
                adm2_names.insert(code.clone(), name.clone());
            }
            2 => {
                adm3_names.insert(code.clone(), name.clone());
            }
            3 => {
                // adm4 row -> insert into bmkg_area
                let parts: Vec<&str> = code.split('.').collect();
                if parts.len() != 4 {
                    continue;
                }
                let adm1 = parts[0].to_string();
                let adm2 = format!("{}.{}", parts[0], parts[1]);
                let adm3 = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
                let adm4 = code.clone();
                let provinsi = adm1_names.get(&adm1).cloned().unwrap_or_default();
                let kotkab = adm2_names.get(&adm2).cloned().unwrap_or_default();
                let kecamatan = adm3_names.get(&adm3).cloned().unwrap_or_default();
                let desa = name.clone();
                let tz = tz_for_adm1(&adm1);

                sqlx::query(
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
                )
                .bind(&adm4)
                .bind(&adm1)
                .bind(&adm2)
                .bind(&adm3)
                .bind(&adm4)
                .bind(&provinsi)
                .bind(&kotkab)
                .bind(&kecamatan)
                .bind(&desa)
                .bind(&tz)
                .bind(now_ms)
                .execute(&mut *tx)
                .await?;
                inserted += 1;
            }
            _ => {}
        }
    }

    tx.commit().await?;
    Ok(inserted)
}

async fn upsert_priorities_from_csv(pool: &sqlx::PgPool, priorities_csv_path: &str) -> Result<i64> {
    // Read priorities from a dedicated CSV: headers = region_code,priority,active
    // active is optional; default true if blank/missing
    let file = File::open(priorities_csv_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(BufReader::new(file));

    let mut tx = pool.begin().await?;
    let now_ms: i64 = chrono::Utc::now().timestamp_millis();
    let mut upserted: i64 = 0;

    for rec in rdr.records() {
        let rec = rec?;
        if rec.len() == 0 {
            continue;
        }
        let region_code = rec.get(0).unwrap().to_string();
        if region_code.is_empty() {
            continue;
        }
        // priority required
        let prio_str = rec.get(1).unwrap_or("");
        if prio_str.is_empty() {
            continue;
        }
        let priority: i32 = prio_str.parse().unwrap_or(100);
        // active optional
        let active_val = rec.get(2).map(|s| s.to_ascii_lowercase());
        let active_opt: Option<bool> = match active_val.as_deref() {
            Some("true") => Some(true),
            Some("false") => Some(false),
            Some("") | None => None,
            _ => None,
        };

        // Ensure region_code exists in bmkg_area to avoid FK violations
        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1::bigint FROM bmkg_area WHERE region_code = $1 AND deleted_at = 0",
        )
        .bind(&region_code)
        .fetch_optional(&mut *tx)
        .await?;

        if exists.is_none() {
            eprintln!(
                "[seed_bmkg_area_from_csv] skip priority for missing region_code: {}",
                region_code
            );
            continue;
        }

        sqlx::query(
            r#"
            INSERT INTO bmkg_area_priority(region_code, priority, active, updated_at)
            VALUES ($1, $2, COALESCE($3, true), $4)
            ON CONFLICT (region_code) DO UPDATE SET
              priority = EXCLUDED.priority,
              active = COALESCE(EXCLUDED.active, bmkg_area_priority.active),
              updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&region_code)
        .bind(priority)
        .bind(active_opt)
        .bind(now_ms)
        .execute(&mut *tx)
        .await?;
        upserted += 1;
    }

    tx.commit().await?;
    Ok(upserted)
}
