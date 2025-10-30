use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

use crate::models::store_ingredient_predictions::StoreIngredientPredictionWithIngredient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientProductUsage {
    pub product_uuid: Option<Uuid>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub recipe_sets_uuid: Option<Uuid>,
    pub recipe_yield_qty: Option<Decimal>,
    pub ingredient_quantity: Option<Decimal>,
    pub waste_percent: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientSnapshot {
    pub ingredient_catalog_uuid: Uuid,
    pub ingredient_name: Option<String>,
    pub ingredient_stock_uuid: Option<Uuid>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
    pub current_stock_qty: Option<Decimal>,
    pub minimum_stock_qty: Option<Decimal>,
    pub linked_products: Vec<IngredientProductUsage>,
}

#[derive(Debug, Clone)]
pub struct NewStoreIngredientPrediction {
    pub store_uuid: Uuid,
    pub ingredient_catalog_uuid: Uuid,
    pub region_code: Option<String>,
    pub restock_label: String,
    pub restock_probability: Option<f32>,
    pub recommended_restock_qty: Decimal,
    pub current_stock_qty: Option<Decimal>,
    pub minimum_stock_qty: Option<Decimal>,
    pub unit_of_measure_code: Option<String>,
    pub unit_of_measure_name: Option<String>,
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

pub async fn fetch_ingredient_snapshots(
    db: &Pool<Postgres>,
    limit: Option<usize>,
) -> Result<Vec<IngredientSnapshot>, sqlx::Error> {
    let rows = sqlx::query_as::<_, IngredientUsageRow>(
        r#"
            SELECT
                ic.uuid AS ingredient_catalog_uuid,
                ic.name AS ingredient_name,
                s.uuid AS ingredient_stock_uuid,
                s.unit_of_measure_code,
                s.unit_of_measure_name,
                s.total_quantity AS current_stock_qty,
                ic.minimum_stock AS minimum_stock_qty,
                p.uuid AS product_uuid,
                p.name AS product_name,
                p.sku AS product_sku,
                rs.uuid AS recipe_sets_uuid,
                rs.yield_quantity,
                ri.quantity AS ingredient_quantity,
                ri.waste_percent
            FROM products p
            LEFT JOIN recipe_sets rs
                ON p.recipe_sets_uuid = rs.uuid
                AND (rs.deleted_at IS NULL OR rs.deleted_at = 0)
            LEFT JOIN recipe_items ri
                ON rs.uuid = ri.recipe_sets_uuid
                AND (ri.deleted_at IS NULL OR ri.deleted_at = 0)
            LEFT JOIN ingredient_stocks s
                ON ri.ingredient_stocks_uuid = s.uuid
                AND s.deleted_at = 0
            LEFT JOIN ingredient_stock_moves sm
                ON s.ingredient_stock_moves_uuid = sm.uuid
                AND sm.deleted_at = 0
            LEFT JOIN ingredient_catalog ic
                ON sm.ingredient_catalog_uuid = ic.uuid
                AND (ic.deleted_at IS NULL OR ic.deleted_at = 0)
            WHERE p.deleted_at = 0
              AND p.status = 'ACTIVE'
            ORDER BY ic.name NULLS LAST, p.name NULLS LAST
        "#,
    )
    .fetch_all(db)
    .await?;

    let mut snapshot_map: HashMap<Uuid, IngredientSnapshot> = HashMap::new();

    for row in rows {
        let ingredient_catalog_uuid = match row.ingredient_catalog_uuid {
            Some(uuid) => uuid,
            None => continue,
        };

        let entry = snapshot_map
            .entry(ingredient_catalog_uuid)
            .or_insert_with(|| IngredientSnapshot {
                ingredient_catalog_uuid,
                ingredient_name: row.ingredient_name.clone(),
                ingredient_stock_uuid: row.ingredient_stock_uuid,
                unit_of_measure_code: row.unit_of_measure_code.clone(),
                unit_of_measure_name: row.unit_of_measure_name.clone(),
                current_stock_qty: row.current_stock_qty,
                minimum_stock_qty: row.minimum_stock_qty,
                linked_products: Vec::new(),
            });

        if entry.ingredient_name.is_none() {
            entry.ingredient_name = row.ingredient_name.clone();
        }
        if entry.ingredient_stock_uuid.is_none() {
            entry.ingredient_stock_uuid = row.ingredient_stock_uuid;
        }
        if entry.unit_of_measure_code.is_none() {
            entry.unit_of_measure_code = row.unit_of_measure_code.clone();
        }
        if entry.unit_of_measure_name.is_none() {
            entry.unit_of_measure_name = row.unit_of_measure_name.clone();
        }
        if entry.current_stock_qty.is_none() {
            entry.current_stock_qty = row.current_stock_qty;
        }
        if entry.minimum_stock_qty.is_none() {
            entry.minimum_stock_qty = row.minimum_stock_qty;
        }

        if let Some(product_uuid) = row.product_uuid {
            entry.linked_products.push(IngredientProductUsage {
                product_uuid: Some(product_uuid),
                product_name: row.product_name.clone(),
                product_sku: row.product_sku.clone(),
                recipe_sets_uuid: row.recipe_sets_uuid,
                recipe_yield_qty: row.yield_quantity,
                ingredient_quantity: row.ingredient_quantity,
                waste_percent: row.waste_percent,
            });
        }
    }

    let mut snapshots: Vec<_> = snapshot_map.into_values().collect();
    snapshots.sort_by(|a, b| {
        a.ingredient_name
            .as_deref()
            .unwrap_or("")
            .cmp(b.ingredient_name.as_deref().unwrap_or(""))
    });

    if let Some(limit) = limit {
        if snapshots.len() > limit {
            snapshots.truncate(limit);
        }
    }

    Ok(snapshots)
}

#[derive(Debug, FromRow)]
struct IngredientUsageRow {
    ingredient_catalog_uuid: Option<Uuid>,
    ingredient_name: Option<String>,
    ingredient_stock_uuid: Option<Uuid>,
    unit_of_measure_code: Option<String>,
    unit_of_measure_name: Option<String>,
    current_stock_qty: Option<Decimal>,
    minimum_stock_qty: Option<Decimal>,
    product_uuid: Option<Uuid>,
    product_name: Option<String>,
    product_sku: Option<String>,
    recipe_sets_uuid: Option<Uuid>,
    yield_quantity: Option<Decimal>,
    ingredient_quantity: Option<Decimal>,
    waste_percent: Option<Decimal>,
}
pub async fn upsert_predictions(
    db: &Pool<Postgres>,
    items: &[NewStoreIngredientPrediction],
) -> Result<(), sqlx::Error> {
    if items.is_empty() {
        return Ok(());
    }

    let mut tx = db.begin().await?;
    for item in items {
        sqlx::query(
            r#"
            INSERT INTO store_ingredient_predictions (
                store_uuid,
                ingredient_catalog_uuid,
                region_code,
                restock_label,
                restock_probability,
                recommended_restock_qty,
                current_stock_qty,
                minimum_stock_qty,
                unit_of_measure_code,
                unit_of_measure_name,
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
            )
            VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,
                (EXTRACT(EPOCH FROM NOW()) * 1000)::bigint,
                (EXTRACT(EPOCH FROM NOW()) * 1000)::bigint,
                0
            )
            ON CONFLICT (store_uuid, ingredient_catalog_uuid)
            DO UPDATE SET
                region_code = EXCLUDED.region_code,
                restock_label = EXCLUDED.restock_label,
                restock_probability = EXCLUDED.restock_probability,
                recommended_restock_qty = EXCLUDED.recommended_restock_qty,
                current_stock_qty = EXCLUDED.current_stock_qty,
                minimum_stock_qty = EXCLUDED.minimum_stock_qty,
                unit_of_measure_code = EXCLUDED.unit_of_measure_code,
                unit_of_measure_name = EXCLUDED.unit_of_measure_name,
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
        .bind(item.ingredient_catalog_uuid)
        .bind(item.region_code.clone())
        .bind(&item.restock_label)
        .bind(item.restock_probability)
        .bind(item.recommended_restock_qty)
        .bind(item.current_stock_qty)
        .bind(item.minimum_stock_qty)
        .bind(item.unit_of_measure_code.clone())
        .bind(item.unit_of_measure_name.clone())
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
) -> Result<Vec<StoreIngredientPredictionWithIngredient>, sqlx::Error> {
    let query = r#"
        SELECT
            sip.uuid,
            sip.store_uuid,
            sip.ingredient_catalog_uuid,
            sip.region_code,
            sip.restock_label,
            sip.restock_probability,
            sip.recommended_restock_qty,
            sip.current_stock_qty,
            sip.minimum_stock_qty,
            sip.unit_of_measure_code,
            sip.unit_of_measure_name,
            sip.weather_summary,
            sip.weather_temp_min_c,
            sip.weather_temp_max_c,
            sip.weather_precip_mm,
            sip.weather_humidity,
            sip.llm_reasoning,
            sip.llm_model,
            sip.llm_prompt,
            sip.llm_response,
            sip.created_at,
            sip.updated_at,
            sip.deleted_at,
            ic.name AS ingredient_name
        FROM store_ingredient_predictions sip
        LEFT JOIN ingredient_catalog ic
            ON ic.uuid = sip.ingredient_catalog_uuid
        WHERE sip.store_uuid = $1
          AND sip.deleted_at = 0
        ORDER BY ic.name ASC NULLS LAST
    "#;

    sqlx::query_as::<_, StoreIngredientPredictionWithIngredient>(query)
        .bind(store_uuid)
        .fetch_all(db)
        .await
}

pub fn aggregate_prediction_fields(
    predictions: &[StoreIngredientPredictionWithIngredient],
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
