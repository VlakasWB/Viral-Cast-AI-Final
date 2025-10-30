use anyhow::{Context, Result};
use chrono::Utc;
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

#[derive(Default)]
struct StockAccumulator {
    total_quantity: Decimal,
    total_value: Decimal,
    last_move_uuid: Option<Uuid>,
    last_price: Option<Decimal>,
    last_timestamp: i64,
    unit_of_measure_code: Option<String>,
    unit_of_measure_name: Option<String>,
}

fn deterministic_uuid(key: &str) -> Uuid {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0F) | 0x50; // version 5 style deterministic
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    Uuid::from_bytes(bytes)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("gagal terhubung ke database")?;

    seed_ingredient_stocks(&pool).await?;
    println!("seeding ingredient_stocks selesai");
    Ok(())
}

async fn seed_ingredient_stocks(pool: &PgPool) -> Result<()> {
    let rows = sqlx::query(
        r#"
        SELECT uuid,
               ingredient_catalog_uuid,
               quantity,
               price,
               effective_at,
               created_at,
               unit_of_measure_code,
               unit_of_measure_name
        FROM ingredient_stock_moves
        WHERE deleted_at = 0
        ORDER BY ingredient_catalog_uuid, effective_at, created_at
        "#,
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        println!("tidak ada data di ingredient_stock_moves, lewati seeding stocks");
        return Ok(());
    }

    let mut accumulators: HashMap<Uuid, StockAccumulator> = HashMap::new();

    for row in rows {
        let ingredient_catalog_uuid: Uuid = row.try_get("ingredient_catalog_uuid")?;
        let quantity: Decimal = row.try_get("quantity")?;
        if quantity.is_zero() {
            continue;
        }
        let price = row
            .try_get::<Option<Decimal>, _>("price")?
            .unwrap_or(Decimal::ZERO);
        let value = (quantity * price).round_dp(4);

        let entry = accumulators.entry(ingredient_catalog_uuid).or_default();
        entry.total_quantity += quantity;
        entry.total_value += value;
        entry.last_move_uuid = Some(row.try_get("uuid")?);
        entry.last_price = if price > Decimal::ZERO {
            Some(price)
        } else {
            entry.last_price
        };
        let effective_at: i64 = row.try_get("effective_at")?;
        let created_at: Option<i64> = row.try_get("created_at")?;
        let created_at = created_at.unwrap_or(effective_at);
        entry.last_timestamp = effective_at.max(created_at);
        let unit_code: Option<String> = row.try_get("unit_of_measure_code")?;
        if unit_code.is_some() {
            entry.unit_of_measure_code = unit_code;
        }
        let unit_name: Option<String> = row.try_get("unit_of_measure_name")?;
        if unit_name.is_some() {
            entry.unit_of_measure_name = unit_name;
        }
    }

    let mut tx = pool.begin().await?;
    for (ingredient_uuid, summary) in accumulators {
        let Some(last_move_uuid) = summary.last_move_uuid else {
            continue;
        };

        let avg_cost = if summary.total_quantity > Decimal::ZERO {
            Some((summary.total_value / summary.total_quantity).round_dp(4))
        } else {
            None
        };

        let current_cost = summary.last_price.or(avg_cost);
        let stock_uuid = deterministic_uuid(&format!("seed-stock-{}", ingredient_uuid));
        let timestamp = summary.last_timestamp.max(Utc::now().timestamp_millis());
        let mut unit_of_measure_code = summary.unit_of_measure_code.clone();
        let mut unit_of_measure_name = summary.unit_of_measure_name.clone();

        if unit_of_measure_code.is_none() || unit_of_measure_name.is_none() {
            if let Some(row) = sqlx::query(
                r#"
                SELECT uom.code, uom.name
                FROM ingredient_catalog ic
                LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
                WHERE ic.uuid = $1
                "#,
            )
            .bind(ingredient_uuid)
            .fetch_optional(&mut *tx)
            .await?
            {
                if unit_of_measure_code.is_none() {
                    unit_of_measure_code = row.try_get("code")?;
                }
                if unit_of_measure_name.is_none() {
                    unit_of_measure_name = row.try_get("name")?;
                }
            }
        }

        sqlx::query(
            r#"
            INSERT INTO ingredient_stocks (
                uuid,
                ingredient_stock_moves_uuid,
                total_quantity,
                total_value,
                current_cost,
                avg_cost,
                unit_of_measure_code,
                unit_of_measure_name,
                created_at,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9, 0)
            ON CONFLICT (uuid) DO UPDATE SET
                ingredient_stock_moves_uuid = EXCLUDED.ingredient_stock_moves_uuid,
                total_quantity = EXCLUDED.total_quantity,
                total_value = EXCLUDED.total_value,
                current_cost = EXCLUDED.current_cost,
                avg_cost = EXCLUDED.avg_cost,
                unit_of_measure_code = EXCLUDED.unit_of_measure_code,
                unit_of_measure_name = EXCLUDED.unit_of_measure_name,
                updated_at = EXCLUDED.updated_at,
                deleted_at = 0
            "#,
        )
        .bind(stock_uuid)
        .bind(last_move_uuid)
        .bind(summary.total_quantity)
        .bind(summary.total_value)
        .bind(current_cost)
        .bind(avg_cost)
        .bind(unit_of_measure_code.clone())
        .bind(unit_of_measure_name.clone())
        .bind(timestamp)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("gagal menyimpan stock untuk ingredient {}", ingredient_uuid))?;
    }

    tx.commit().await?;
    Ok(())
}
