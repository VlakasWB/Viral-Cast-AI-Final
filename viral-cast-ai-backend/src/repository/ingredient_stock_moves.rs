use std::collections::HashMap;

use regex::escape;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sqlx::Row;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::dto::ingredient_stock_moves::{
    CreateIngredientStockMoveSchema, GetIngredientStockMoveSchema, IngredientStockMoveResponse,
    UpdateIngredientStockMoveSchema,
};
use crate::repository::ingredient_stocks;

const MILLIS_PER_DAY: i64 = 86_400_000;

// Membuat pergerakan stok bahan baru
pub async fn create_ingredient_stock_move(
    db: &Pool<Postgres>,
    uuid: Uuid,
    body: CreateIngredientStockMoveSchema,
    timestamp_ms: i64,
) -> Result<IngredientStockMoveResponse, sqlx::Error> {
    let CreateIngredientStockMoveSchema {
        name,
        ingredient_catalog_uuid,
        quantity,
        price,
        price_updated_at,
        effective_at,
        expiry_at,
        ref_type,
        ref_uuid,
        unit_of_measure_code,
        unit_of_measure_name,
    } = body;

    let derived_expiry_at =
        derive_expiry_at(db, ingredient_catalog_uuid, expiry_at, effective_at).await?;

    let price_dec = price.and_then(rust_decimal::Decimal::from_f64);
    let quantity_dec =
        rust_decimal::Decimal::from_f64(quantity).unwrap_or(rust_decimal::Decimal::ZERO);

    let mut uom_code = unit_of_measure_code;
    let mut uom_name = unit_of_measure_name;

    if uom_code.is_none() || uom_name.is_none() {
        if let Some(uom_row) = sqlx::query(
            r#"
            SELECT uom.code AS code, uom.name AS name
            FROM ingredient_catalog ic
            LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
            WHERE ic.uuid = $1
            "#,
        )
        .bind(ingredient_catalog_uuid)
        .fetch_optional(db)
        .await?
        {
            if uom_code.is_none() {
                uom_code = uom_row.try_get("code").ok();
            }
            if uom_name.is_none() {
                uom_name = uom_row.try_get("name").ok();
            }
        }
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO ingredient_stock_moves (
            uuid,
            name,
            ingredient_catalog_uuid,
            quantity,
            price,
            price_updated_at,
            effective_at,
            expiry_at,
            ref_type,
            ref_uuid,
            unit_of_measure_code,
            unit_of_measure_name,
            created_at,
            updated_at,
            deleted_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13, 0)
        RETURNING uuid,
                  name,
                  ingredient_catalog_uuid,
                  quantity,
                  price,
                  price_updated_at,
                  effective_at,
                  expiry_at,
                  ref_type,
                  ref_uuid,
                  unit_of_measure_code,
                  unit_of_measure_name,
                  created_at,
                  updated_at
        "#,
        uuid,
        name,
        ingredient_catalog_uuid,
        quantity_dec,
        price_dec,
        price_updated_at,
        effective_at,
        derived_expiry_at,
        ref_type,
        ref_uuid,
        uom_code.clone(),
        uom_name.clone(),
        timestamp_ms
    )
    .fetch_one(db)
    .await?;

    ingredient_stocks::recompute_stock_for_ingredient(db, result.ingredient_catalog_uuid).await?;
    Ok(IngredientStockMoveResponse {
        uuid: result.uuid,
        name: result.name,
        ingredient_catalog_uuid: result.ingredient_catalog_uuid,
        quantity: result.quantity.to_f64().unwrap_or(0.0),
        price: result.price.map(|p| p.to_f64().unwrap_or(0.0)),
        price_updated_at: result.price_updated_at,
        effective_at: result.effective_at,
        expiry_at: result.expiry_at,
        ref_type: result.ref_type,
        ref_uuid: result.ref_uuid,
        created_at: result.created_at,
        updated_at: result.updated_at,
        unit_of_measure_code: result.unit_of_measure_code,
        unit_of_measure_name: result.unit_of_measure_name,
    })
}

// Mendapatkan daftar pergerakan stok bahan dengan paginasi dan filter
pub async fn list_ingredient_stock_moves(
    db: &Pool<Postgres>,
    page: usize,
    limit: usize,
    filters: GetIngredientStockMoveSchema,
) -> Result<(Vec<IngredientStockMoveResponse>, i64), sqlx::Error> {
    let offset = (page.saturating_sub(1) * limit) as i64;
    let normalized_ref_type = filters
        .ref_type
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_uppercase());
    // Samakan dengan pola search UOM: trim, bungkus dengan %...%, dan gunakan ILIKE
    // Mendukung parameter `name`, jika ada maka diprioritaskan daripada `search`
    let name_pattern = filters
        .name
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s));

    let search_pattern = filters
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s));

    let name_or_search = name_pattern.as_ref().or(search_pattern.as_ref());

    let order_direction = if filters
        .sort_direction
        .as_deref()
        .map(|dir| dir.eq_ignore_ascii_case("asc"))
        .unwrap_or(false)
    {
        "ASC"
    } else {
        "DESC"
    };

    let list_query = format!(
        r#"
        SELECT m.uuid,
               m.ingredient_catalog_uuid,
               m.quantity,
               m.price,
               m.price_updated_at,
               m.effective_at,
               m.expiry_at,
               m.ref_type,
               m.ref_uuid,
               m.name,
               m.created_at,
               m.updated_at,
               m.unit_of_measure_code,
               m.unit_of_measure_name
        FROM ingredient_stock_moves m
        WHERE m.deleted_at = 0
          AND ($1::uuid IS NULL OR m.ingredient_catalog_uuid = $1)
          AND ($2::text IS NULL OR UPPER(m.ref_type) = $2)
          AND ($3::uuid IS NULL OR m.ref_uuid = $3)
          AND ($4::bigint IS NULL OR m.effective_at >= $4)
          AND ($5::bigint IS NULL OR m.effective_at <= $5)
          AND ($6::text IS NULL OR m.name ILIKE $6)
        ORDER BY m.effective_at {dir}
        LIMIT $7 OFFSET $8
        "#,
        dir = order_direction
    );

    let rows = sqlx::query(&list_query)
        .bind(filters.ingredient_catalog_uuid)
        .bind(normalized_ref_type.as_deref())
        .bind(filters.ref_uuid)
        .bind(filters.from_date)
        .bind(filters.to_date)
        .bind(name_or_search.map(|s| s.as_str()))
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(db)
        .await?;

    let count_row = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM ingredient_stock_moves
        WHERE deleted_at = 0
          AND ($1::uuid IS NULL OR ingredient_catalog_uuid = $1)
          AND ($2::text IS NULL OR UPPER(ref_type) = $2)
          AND ($3::uuid IS NULL OR ref_uuid = $3)
          AND ($4::bigint IS NULL OR effective_at >= $4)
          AND ($5::bigint IS NULL OR effective_at <= $5)
          AND ($6::text IS NULL OR name ILIKE $6)
        "#,
    )
    .bind(filters.ingredient_catalog_uuid)
    .bind(normalized_ref_type.as_deref())
    .bind(filters.ref_uuid)
    .bind(filters.from_date)
    .bind(filters.to_date)
    .bind(name_or_search.map(|s| s.as_str()))
    .fetch_one(db)
    .await?;

    let count: i64 = count_row.get::<i64, _>("count");

    let mut shelf_life_cache: HashMap<Uuid, Option<i32>> = HashMap::new();
    let mut items = Vec::with_capacity(rows.len());

    for row in rows {
        let ingredient_catalog_uuid: Uuid = row.get("ingredient_catalog_uuid");
        let effective_at = row.get::<i64, _>("effective_at");
        let stored_expiry = row.get::<Option<i64>, _>("expiry_at");
        let expiry_at = if stored_expiry.is_some() {
            stored_expiry
        } else {
            let shelf_life_days =
                if let Some(&cached) = shelf_life_cache.get(&ingredient_catalog_uuid) {
                    cached
                } else {
                    let shelf = fetch_shelf_life_days(db, ingredient_catalog_uuid).await?;
                    shelf_life_cache.insert(ingredient_catalog_uuid, shelf);
                    shelf
                };
            calculate_expiry_at(effective_at, shelf_life_days)
        };

        let quantity_dec: Decimal = row.get("quantity");
        let price_dec: Option<Decimal> = row.get("price");

        items.push(IngredientStockMoveResponse {
            uuid: row.get::<Uuid, _>("uuid"),
            ingredient_catalog_uuid,
            quantity: quantity_dec.to_f64().unwrap_or(0.0),
            price: price_dec.map(|p| p.to_f64().unwrap_or(0.0)),
            price_updated_at: row.get::<Option<i64>, _>("price_updated_at"),
            effective_at,
            expiry_at,
            ref_type: row.get::<Option<String>, _>("ref_type"),
            ref_uuid: row.get::<Option<Uuid>, _>("ref_uuid"),
            name: row.get::<Option<String>, _>("name"),
            created_at: row.get::<Option<i64>, _>("created_at"),
            updated_at: row.get::<Option<i64>, _>("updated_at"),
            unit_of_measure_code: row.get::<Option<String>, _>("unit_of_measure_code"),
            unit_of_measure_name: row.get::<Option<String>, _>("unit_of_measure_name"),
        });
    }

    Ok((items, count))
}

// Mendapatkan pergerakan stok bahan berdasarkan UUID
pub async fn get_ingredient_stock_move_by_uuid(
    db: &Pool<Postgres>,
    id: Uuid,
) -> Result<Option<IngredientStockMoveResponse>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT m.uuid,
               m.name,
               m.ingredient_catalog_uuid,
               m.quantity,
               m.price,
               m.price_updated_at,
               m.effective_at,
               m.expiry_at,
               m.ref_type,
               m.ref_uuid,
               m.created_at,
               m.updated_at,
               m.unit_of_measure_code AS "unit_of_measure_code?",
               m.unit_of_measure_name AS "unit_of_measure_name?"
        FROM ingredient_stock_moves m
        WHERE m.uuid = $1 AND m.deleted_at = 0
        "#,
        id
    )
    .fetch_optional(db)
    .await?;

    let result = if let Some(r) = row {
        let mut expiry_at = r.expiry_at;
        if expiry_at.is_none() {
            let shelf_life_days = fetch_shelf_life_days(db, r.ingredient_catalog_uuid).await?;
            expiry_at = calculate_expiry_at(r.effective_at, shelf_life_days);
        }

        Some(IngredientStockMoveResponse {
            uuid: r.uuid,
            name: r.name,
            ingredient_catalog_uuid: r.ingredient_catalog_uuid,
            quantity: r.quantity.to_f64().unwrap_or(0.0),
            price: r.price.map(|p| p.to_f64().unwrap_or(0.0)),
            price_updated_at: r.price_updated_at,
            effective_at: r.effective_at,
            expiry_at,
            ref_type: r.ref_type,
            ref_uuid: r.ref_uuid,
            created_at: r.created_at,
            updated_at: r.updated_at,
            unit_of_measure_code: r.unit_of_measure_code,
            unit_of_measure_name: r.unit_of_measure_name,
        })
    } else {
        None
    };

    Ok(result)
}

// Memperbarui pergerakan stok bahan
pub async fn update_ingredient_stock_move(
    db: &Pool<Postgres>,
    id: Uuid,
    body: UpdateIngredientStockMoveSchema,
    timestamp_ms: i64,
) -> Result<Option<IngredientStockMoveResponse>, sqlx::Error> {
    // Cek apakah data ada
    let existing = sqlx::query!(
        r#"
        SELECT * FROM ingredient_stock_moves
        WHERE uuid = $1 AND deleted_at = 0
        "#,
        id
    )
    .fetch_optional(db)
    .await?;

    let Some(existing) = existing else {
        return Ok(None);
    };

    // Persiapkan nilai yang akan diupdate
    let name = body.name.or(existing.name);
    let ingredient_catalog_uuid = body
        .ingredient_catalog_uuid
        .unwrap_or(existing.ingredient_catalog_uuid);
    let quantity = body
        .quantity
        .unwrap_or_else(|| existing.quantity.to_f64().unwrap_or(0.0));
    let price_f64 = body
        .price
        .or_else(|| existing.price.map(|p| p.to_f64().unwrap_or(0.0)));
    let price_dec = price_f64.and_then(rust_decimal::Decimal::from_f64);
    let quantity_dec =
        rust_decimal::Decimal::from_f64(quantity).unwrap_or(rust_decimal::Decimal::ZERO);
    let price_updated_at = body.price_updated_at.or(existing.price_updated_at);
    let effective_at = body.effective_at.unwrap_or(existing.effective_at);
    let computed_expiry =
        derive_expiry_at(db, ingredient_catalog_uuid, body.expiry_at, effective_at).await?;
    let expiry_at = computed_expiry.or(existing.expiry_at);
    let ref_type = body.ref_type.or(existing.ref_type);
    let ref_uuid = body.ref_uuid.or(existing.ref_uuid);
    let mut unit_of_measure_code = body
        .unit_of_measure_code
        .or(existing.unit_of_measure_code.clone());
    let mut unit_of_measure_name = body
        .unit_of_measure_name
        .or(existing.unit_of_measure_name.clone());

    if unit_of_measure_code.is_none() || unit_of_measure_name.is_none() {
        if let Some(uom_row) = sqlx::query(
            r#"
            SELECT uom.code AS code, uom.name AS name
            FROM ingredient_catalog ic
            LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
            WHERE ic.uuid = $1
            "#,
        )
        .bind(ingredient_catalog_uuid)
        .fetch_optional(db)
        .await?
        {
            if unit_of_measure_code.is_none() {
                unit_of_measure_code = uom_row.try_get("code").ok();
            }
            if unit_of_measure_name.is_none() {
                unit_of_measure_name = uom_row.try_get("name").ok();
            }
        }
    }

    // Update data
    let updated = sqlx::query!(
        r#"
        UPDATE ingredient_stock_moves
        SET name = $1,
            ingredient_catalog_uuid = $2,
            quantity = $3,
            price = $4,
            price_updated_at = $5,
            effective_at = $6,
            expiry_at = $7,
            ref_type = $8,
            ref_uuid = $9,
            unit_of_measure_code = $10,
            unit_of_measure_name = $11,
            updated_at = $12
        WHERE uuid = $13 AND deleted_at = 0
        RETURNING uuid,
                  name,
                  ingredient_catalog_uuid,
                  quantity,
                  price,
                  price_updated_at,
                  effective_at,
                  expiry_at,
                  ref_type,
                  ref_uuid,
                  unit_of_measure_code,
                  unit_of_measure_name,
                  created_at,
                  updated_at
        "#,
        name,
        ingredient_catalog_uuid,
        quantity_dec,
        price_dec,
        price_updated_at,
        effective_at,
        expiry_at,
        ref_type,
        ref_uuid,
        unit_of_measure_code.clone(),
        unit_of_measure_name.clone(),
        timestamp_ms,
        id
    )
    .fetch_optional(db)
    .await?;

    let response = updated.map(|r| IngredientStockMoveResponse {
        uuid: r.uuid,
        name: r.name,
        ingredient_catalog_uuid: r.ingredient_catalog_uuid,
        quantity: r.quantity.to_f64().unwrap_or(0.0),
        price: r.price.map(|p| p.to_f64().unwrap_or(0.0)),
        price_updated_at: r.price_updated_at,
        effective_at: r.effective_at,
        expiry_at: r.expiry_at,
        ref_type: r.ref_type,
        ref_uuid: r.ref_uuid,
        created_at: r.created_at,
        updated_at: r.updated_at,
        unit_of_measure_code: r.unit_of_measure_code,
        unit_of_measure_name: r.unit_of_measure_name,
    });

    if let Some(ref row) = response {
        ingredient_stocks::recompute_stock_for_ingredient(db, row.ingredient_catalog_uuid).await?;
        if row.ingredient_catalog_uuid != existing.ingredient_catalog_uuid {
            ingredient_stocks::recompute_stock_for_ingredient(db, existing.ingredient_catalog_uuid)
                .await?;
        }
    }

    Ok(response)
}

// Soft delete pergerakan stok bahan
pub async fn soft_delete_ingredient_stock_move(
    db: &Pool<Postgres>,
    id: Uuid,
    timestamp_ms: i64,
) -> Result<i64, sqlx::Error> {
    let existing = sqlx::query!(
        r#"
        SELECT ingredient_catalog_uuid
        FROM ingredient_stock_moves
        WHERE uuid = $1 AND deleted_at = 0
        "#,
        id
    )
    .fetch_optional(db)
    .await?;

    let Some(existing) = existing else {
        return Ok(0);
    };

    let result = sqlx::query!(
        r#"
        UPDATE ingredient_stock_moves
        SET deleted_at = $1
        WHERE uuid = $2 AND deleted_at = 0
        "#,
        timestamp_ms,
        id
    )
    .execute(db)
    .await?;

    if result.rows_affected() > 0 {
        ingredient_stocks::recompute_stock_for_ingredient(db, existing.ingredient_catalog_uuid)
            .await?;
    }

    Ok(result.rows_affected() as i64)
}

fn calculate_expiry_at(effective_at: i64, shelf_life_days: Option<i32>) -> Option<i64> {
    shelf_life_days.and_then(|days| {
        let shelf_life_ms = i64::from(days).checked_mul(MILLIS_PER_DAY)?;
        effective_at.checked_add(shelf_life_ms)
    })
}

async fn fetch_shelf_life_days(
    db: &Pool<Postgres>,
    ingredient_catalog_uuid: Uuid,
) -> Result<Option<i32>, sqlx::Error> {
    let shelf_life_days = sqlx::query_scalar!(
        r#"
        SELECT shelf_life_days
        FROM ingredient_catalog
        WHERE uuid = $1 AND deleted_at = 0
        "#,
        ingredient_catalog_uuid
    )
    .fetch_optional(db)
    .await?;

    Ok(shelf_life_days.flatten())
}

async fn derive_expiry_at(
    db: &Pool<Postgres>,
    ingredient_catalog_uuid: Uuid,
    provided_expiry_at: Option<i64>,
    effective_at: i64,
) -> Result<Option<i64>, sqlx::Error> {
    if provided_expiry_at.is_some() {
        return Ok(provided_expiry_at);
    }

    let shelf_life_days = fetch_shelf_life_days(db, ingredient_catalog_uuid).await?;

    Ok(calculate_expiry_at(effective_at, shelf_life_days))
}
