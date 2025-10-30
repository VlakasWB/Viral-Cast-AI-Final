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

    println!("== bmkg_area_priority (top 20 by priority & next_due) ==");
    let rows = sqlx::query(
        r#"
        SELECT region_code, priority, active, last_hit_ms, next_due_ms, updated_at
        FROM bmkg_area_priority
        WHERE deleted_at = 0
        ORDER BY priority ASC, COALESCE(next_due_ms, 0) ASC
        LIMIT 20
        "#,
    )
    .fetch_all(&pool)
    .await?;

    for r in rows.iter() {
        let rc: String = r.get("region_code");
        let prio: i32 = r.get("priority");
        let active: bool = r.get("active");
        let last_hit_ms: Option<i64> = r.try_get("last_hit_ms").ok();
        let next_due_ms: Option<i64> = r.try_get("next_due_ms").ok();
        let updated_at: i64 = r.get("updated_at");
        println!(
            "{rc}: priority={prio}, active={active}, last_hit_ms={last_hit_ms:?}, next_due_ms={next_due_ms:?}, updated_at={updated_at}"
        );
    }

    // Check forecast counts for a few known codes
    let codes_to_check = vec![
        "31.71.01.1001", // Jakarta Pusat
        "31.71.03.1001", // Jakarta code sample
        "32.01.01.2001", // Known failing code from scheduler logs
    ];
    println!("\n== bmkg_forecast counts for selected region codes ==");
    for code in codes_to_check {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM bmkg_forecast WHERE region_code = $1 AND deleted_at = 0",
        )
        .bind(code)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
        println!("{code}: {count}");
    }

    Ok(())
}
