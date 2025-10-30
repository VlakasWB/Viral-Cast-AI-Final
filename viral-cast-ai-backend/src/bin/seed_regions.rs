use anyhow::Result;
use csv::ReaderBuilder;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Row {
    code: String,
    name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let csv_path = env::var("REGION_CSV_PATH").unwrap_or_else(|_| "data/regions.csv".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    seed_from_csv(&pool, &csv_path).await?;
    println!("✅ seeding regions done");
    Ok(())
}

fn level(code: &str) -> usize {
    code.split('.').count()
}

fn parent_code(code: &str) -> Option<String> {
    let mut parts: Vec<&str> = code.split('.').collect();
    if parts.len() <= 1 {
        return None;
    }
    parts.pop();
    Some(parts.join("."))
}

async fn seed_from_csv(pool: &PgPool, csv_path: &str) -> Result<()> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .quote(b'"')
        .double_quote(true)
        .flexible(true) // Allow records with different number of fields
        .from_path(csv_path)?;

    let mut l1 = Vec::new(); // province
    let mut l2 = Vec::new(); // regency
    let mut l3 = Vec::new(); // district
    let mut l4 = Vec::new(); // village

    for result in rdr.records() {
        let record = result?;

        // Skip records that don't have exactly 2 fields
        if record.len() != 2 {
            continue;
        }

        let code = record.get(0).unwrap_or("").trim().to_string();
        let name = record.get(1).unwrap_or("").trim().to_string();

        // Skip empty records
        if code.is_empty() || name.is_empty() {
            continue;
        }

        let row = Row { code, name };

        match level(&row.code) {
            1 => l1.push(row),
            2 => l2.push(row),
            3 => l3.push(row),
            4 => l4.push(row),
            _ => {} // ignore out-of-scope rows
        }
    }

    let mut tx = pool.begin().await?;

    use std::collections::HashMap;
    let mut prov_uuid: HashMap<String, Uuid> = HashMap::new();
    let mut reg_uuid: HashMap<String, Uuid> = HashMap::new();
    let mut dist_uuid: HashMap<String, Uuid> = HashMap::new();

    // Provinces
    for r in l1 {
        let uuid: Uuid = sqlx::query_scalar!(
            r#"
            INSERT INTO province (code, name)
            VALUES ($1, $2)
            ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name
            RETURNING uuid
            "#,
            r.code,
            r.name
        )
        .fetch_one(&mut *tx)
        .await?;
        prov_uuid.insert(r.code, uuid);
    }

    // Regencies
    for r in l2 {
        let parent = parent_code(&r.code).expect("regency must have province parent");
        let parent_uuid = *prov_uuid
            .get(&parent)
            .ok_or_else(|| anyhow::anyhow!("missing province {}", parent))?;
        let uuid: Uuid = sqlx::query_scalar!(
            r#"
            INSERT INTO regency (code, name, province_uuid)
            VALUES ($1, $2, $3)
            ON CONFLICT (code) DO UPDATE
              SET name = EXCLUDED.name,
                  province_uuid = EXCLUDED.province_uuid
            RETURNING uuid
            "#,
            r.code,
            r.name,
            parent_uuid
        )
        .fetch_one(&mut *tx)
        .await?;
        reg_uuid.insert(r.code, uuid);
    }

    // Districts
    for r in l3 {
        let parent = parent_code(&r.code).expect("district must have regency parent");
        let parent_uuid = *reg_uuid
            .get(&parent)
            .ok_or_else(|| anyhow::anyhow!("missing regency {}", parent))?;
        let uuid: Uuid = sqlx::query_scalar!(
            r#"
            INSERT INTO district (code, name, regency_uuid)
            VALUES ($1, $2, $3)
            ON CONFLICT (code) DO UPDATE
              SET name = EXCLUDED.name,
                  regency_uuid = EXCLUDED.regency_uuid
            RETURNING uuid
            "#,
            r.code,
            r.name,
            parent_uuid
        )
        .fetch_one(&mut *tx)
        .await?;
        dist_uuid.insert(r.code, uuid);
    }

    // Villages
    for r in l4 {
        let parent = parent_code(&r.code).expect("village must have district parent");

        // Skip villages whose district doesn't exist
        if !dist_uuid.contains_key(&parent) {
            println!(
                "Warning: Skipping village {} - district {} not found",
                r.code, parent
            );
            continue;
        }

        let parent_uuid = *dist_uuid.get(&parent).unwrap();
        let _uuid: Uuid = sqlx::query_scalar!(
            r#"
            INSERT INTO village (code, name, district_uuid)
            VALUES ($1, $2, $3)
            ON CONFLICT (code) DO UPDATE
              SET name = EXCLUDED.name,
                  district_uuid = EXCLUDED.district_uuid
            RETURNING uuid
            "#,
            r.code,
            r.name,
            parent_uuid
        )
        .fetch_one(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
