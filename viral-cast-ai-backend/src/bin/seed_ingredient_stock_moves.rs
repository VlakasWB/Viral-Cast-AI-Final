use anyhow::{Context, Result};
use chrono::Utc;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

const MILLIS_PER_DAY: i64 = 86_400_000;

#[derive(Clone)]
struct StockMoveSeed {
    ingredient_name: &'static str,
    quantity: f64,
    price: f64,
}

#[derive(Clone)]
struct IngredientCatalogEntry {
    uuid: Uuid,
    shelf_life_days: Option<i32>,
    unit_of_measure_code: Option<String>,
    unit_of_measure_name: Option<String>,
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

    seed_ingredient_stock_moves(&pool).await?;
    println!("seeding ingredient_stock_moves & ingredient_stocks selesai");

    Ok(())
}

fn deterministic_uuid(key: &str) -> Uuid {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0F) | 0x50; // set version 5
    bytes[8] = (bytes[8] & 0x3F) | 0x80; // set variant RFC4122
    Uuid::from_bytes(bytes)
}

async fn seed_ingredient_stock_moves(pool: &PgPool) -> Result<()> {
    let seeds = vec![
        StockMoveSeed {
            ingredient_name: "Nasi Putih Medium Grain",
            quantity: 200.0,
            price: 11500.0,
        },
        StockMoveSeed {
            ingredient_name: "Kecap Manis Premium",
            quantity: 80.0,
            price: 28500.0,
        },
        StockMoveSeed {
            ingredient_name: "Saus Tiram Premium",
            quantity: 60.0,
            price: 32500.0,
        },
        StockMoveSeed {
            ingredient_name: "Bumbu Nasi Goreng Tradisional",
            quantity: 55.0,
            price: 54000.0,
        },
        StockMoveSeed {
            ingredient_name: "Bumbu Nasi Goreng Spesial",
            quantity: 45.0,
            price: 62500.0,
        },
        StockMoveSeed {
            ingredient_name: "Bumbu Nasi Goreng Seafood",
            quantity: 40.0,
            price: 67500.0,
        },
        StockMoveSeed {
            ingredient_name: "Bumbu Nasi Kuning",
            quantity: 48.0,
            price: 49500.0,
        },
        StockMoveSeed {
            ingredient_name: "Tepung Bumbu Gurame",
            quantity: 38.0,
            price: 38500.0,
        },
        StockMoveSeed {
            ingredient_name: "Roti Burger Sesame",
            quantity: 420.0,
            price: 6500.0,
        },
        StockMoveSeed {
            ingredient_name: "Patty Daging Sapi Burger",
            quantity: 65.0,
            price: 98500.0,
        },
        StockMoveSeed {
            ingredient_name: "Pasta Spaghetti Durum",
            quantity: 170.0,
            price: 24500.0,
        },
        StockMoveSeed {
            ingredient_name: "Saus Pasta Marinara",
            quantity: 95.0,
            price: 31500.0,
        },
        StockMoveSeed {
            ingredient_name: "Keju Mozzarella Block",
            quantity: 70.0,
            price: 128500.0,
        },
        StockMoveSeed {
            ingredient_name: "Adonan Pizza Beku",
            quantity: 72.0,
            price: 42500.0,
        },
        StockMoveSeed {
            ingredient_name: "Bubuk Matcha Premium",
            quantity: 28.0,
            price: 212500.0,
        },
        StockMoveSeed {
            ingredient_name: "Biji Kopi Arabika Medium Roast",
            quantity: 75.0,
            price: 189000.0,
        },
        StockMoveSeed {
            ingredient_name: "Sirup Gula Cair",
            quantity: 135.0,
            price: 18500.0,
        },
        StockMoveSeed {
            ingredient_name: "Krim Kental Manis Cair",
            quantity: 110.0,
            price: 31500.0,
        },
        StockMoveSeed {
            ingredient_name: "Susu Segar Pasteurisasi",
            quantity: 220.0,
            price: 19500.0,
        },
        StockMoveSeed {
            ingredient_name: "Filling Pie Apel",
            quantity: 68.0,
            price: 42500.0,
        },
        StockMoveSeed {
            ingredient_name: "Kulit Pie Beku",
            quantity: 320.0,
            price: 4500.0,
        },
        StockMoveSeed {
            ingredient_name: "Cake Mix Vanilla Premium",
            quantity: 90.0,
            price: 48500.0,
        },
        StockMoveSeed {
            ingredient_name: "Sayuran Campur Beku",
            quantity: 150.0,
            price: 28500.0,
        },
        StockMoveSeed {
            ingredient_name: "Bok Choy Segar",
            quantity: 95.0,
            price: 21000.0,
        },
        StockMoveSeed {
            ingredient_name: "Paprika Merah Besar",
            quantity: 85.0,
            price: 45500.0,
        },
        StockMoveSeed {
            ingredient_name: "Daging Sapi Cincang Premium",
            quantity: 80.0,
            price: 112500.0,
        },
        StockMoveSeed {
            ingredient_name: "Dada Ayam Fillet Tanpa Kulit",
            quantity: 115.0,
            price: 62500.0,
        },
    ];

    let mut ingredient_cache: HashMap<&'static str, IngredientCatalogEntry> = HashMap::new();

    for seed in seeds {
        let ingredient_entry = if let Some(entry) = ingredient_cache.get(seed.ingredient_name) {
            entry.clone()
        } else {
            let row = sqlx::query(
                r#"
                SELECT
                    ic.uuid,
                    ic.shelf_life_days,
                    uom.code,
                    uom.name
                FROM ingredient_catalog ic
                LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
                WHERE ic.name = $1 AND ic.deleted_at = 0
                "#,
            )
            .bind(seed.ingredient_name)
            .fetch_optional(pool)
            .await?
            .context(format!(
                "Ingredient '{}' belum tersedia. Jalankan seeder ingredient catalog terlebih dahulu.",
                seed.ingredient_name
            ))?;
            let entry = IngredientCatalogEntry {
                uuid: row.try_get("uuid")?,
                shelf_life_days: row.try_get::<Option<i32>, _>("shelf_life_days")?,
                unit_of_measure_code: row.try_get::<Option<String>, _>("code")?,
                unit_of_measure_name: row.try_get::<Option<String>, _>("name")?,
            };
            ingredient_cache.insert(seed.ingredient_name, entry.clone());
            entry
        };
        let ingredient_uuid = ingredient_entry.uuid;

        let quantity_dec =
            Decimal::from_f64(seed.quantity).context("gagal konversi quantity ke Decimal")?;
        let price_dec = Decimal::from_f64(seed.price).context("gagal konversi harga ke Decimal")?;
        let total_value = (quantity_dec * price_dec).round_dp(4);

        let now = Utc::now().timestamp_millis();
        let expiry_at = compute_expiry_at(now, ingredient_entry.shelf_life_days);
        let move_uuid = deterministic_uuid(&format!("{}_move", seed.ingredient_name));
        let stock_uuid = deterministic_uuid(&format!("{}_stock", seed.ingredient_name));
        let mut unit_of_measure_code = ingredient_entry.unit_of_measure_code.clone();
        let mut unit_of_measure_name = ingredient_entry.unit_of_measure_name.clone();

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
            .fetch_optional(pool)
            .await?
            {
                if unit_of_measure_code.is_none() {
                    unit_of_measure_code = row.try_get::<Option<String>, _>("code")?;
                }
                if unit_of_measure_name.is_none() {
                    unit_of_measure_name = row.try_get::<Option<String>, _>("name")?;
                }
            }
        }

        sqlx::query(
            r#"
            INSERT INTO ingredient_stock_moves (
                uuid, ingredient_catalog_uuid, quantity, price, price_updated_at,
                effective_at, expiry_at, ref_type, ref_uuid, name,
                unit_of_measure_code, unit_of_measure_name,
                created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13, 0)
            ON CONFLICT (uuid) DO UPDATE SET
                ingredient_catalog_uuid = EXCLUDED.ingredient_catalog_uuid,
                quantity = EXCLUDED.quantity,
                price = EXCLUDED.price,
                price_updated_at = EXCLUDED.price_updated_at,
                effective_at = EXCLUDED.effective_at,
                expiry_at = EXCLUDED.expiry_at,
                ref_type = EXCLUDED.ref_type,
                ref_uuid = EXCLUDED.ref_uuid,
                name = EXCLUDED.name,
                unit_of_measure_code = EXCLUDED.unit_of_measure_code,
                unit_of_measure_name = EXCLUDED.unit_of_measure_name,
                updated_at = EXCLUDED.updated_at,
                deleted_at = 0
            "#,
        )
        .bind(move_uuid)
        .bind(ingredient_uuid)
        .bind(quantity_dec)
        .bind(Some(price_dec))
        .bind(Some(now))
        .bind(now)
        .bind(expiry_at)
        .bind(Some("PURCHASE"))
        .bind(Option::<Uuid>::None)
        .bind(Some(seed.ingredient_name.to_string()))
        .bind(unit_of_measure_code.clone())
        .bind(unit_of_measure_name.clone())
        .bind(now)
        .execute(pool)
        .await
        .with_context(|| format!("gagal menulis stock move untuk {}", seed.ingredient_name))?;

        sqlx::query(
            r#"
            INSERT INTO ingredient_stocks (
                uuid, ingredient_stock_moves_uuid, total_quantity, total_value,
                current_cost, avg_cost, unit_of_measure_code, unit_of_measure_name, created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $5, $6, $7, $8, $8, 0)
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
        .bind(move_uuid)
        .bind(quantity_dec)
        .bind(total_value)
        .bind(Some(price_dec))
        .bind(unit_of_measure_code.clone())
        .bind(unit_of_measure_name.clone())
        .bind(now)
        .execute(pool)
        .await
        .with_context(|| {
            format!(
                "gagal menulis ingredient stock untuk {}",
                seed.ingredient_name
            )
        })?;
    }

    Ok(())
}

fn compute_expiry_at(effective_at: i64, shelf_life_days: Option<i32>) -> Option<i64> {
    shelf_life_days.and_then(|days| {
        let shelf_life_ms = i64::from(days).checked_mul(MILLIS_PER_DAY)?;
        effective_at.checked_add(shelf_life_ms)
    })
}
