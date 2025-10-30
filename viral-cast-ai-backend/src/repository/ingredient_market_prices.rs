use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sqlx::Row;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::dto::ingredient_market_prices::{
    CreateIngredientMarketPriceSchema, GetIngredientMarketPriceSchema,
    IngredientMarketPriceResponse, UpdateIngredientMarketPriceSchema,
};

// Membuat harga pasar bahan baru
pub async fn create_ingredient_market_price(
    db: &Pool<Postgres>,
    uuid: Uuid,
    body: CreateIngredientMarketPriceSchema,
    timestamp_ms: i64,
) -> Result<IngredientMarketPriceResponse, sqlx::Error> {
    let body_price_dec: Option<Decimal> = body.price.and_then(Decimal::from_f64);

    let mut uom_code = body.unit_of_measure_code.clone();
    let mut uom_name = body.unit_of_measure_name.clone();

    if uom_code.is_none() || uom_name.is_none() {
        if let Some(uom_row) = sqlx::query(
            r#"
            SELECT uom.code AS code, uom.name AS name
            FROM ingredient_catalog ic
            LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
            WHERE ic.uuid = $1
            "#,
        )
        .bind(body.ingredient_catalog_uuid)
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

    let row = sqlx::query(
        r#"
        INSERT INTO ingredient_market_prices (
            uuid,
            ingredient_catalog_uuid,
            name,
            price,
            effective_at,
            unit_of_measure_code,
            unit_of_measure_name,
            created_at,
            updated_at,
            deleted_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8, 0)
        RETURNING
            uuid,
            ingredient_catalog_uuid,
            name,
            price,
            effective_at,
            unit_of_measure_code,
            unit_of_measure_name,
            created_at,
            updated_at
        "#,
    )
    .bind(uuid)
    .bind(body.ingredient_catalog_uuid)
    .bind(body.name)
    .bind(body_price_dec)
    .bind(body.effective_at)
    .bind(uom_code.clone())
    .bind(uom_name.clone())
    .bind(timestamp_ms)
    .fetch_one(db)
    .await?;

    Ok(IngredientMarketPriceResponse {
        uuid: row.try_get("uuid")?,
        ingredient_catalog_uuid: row.try_get("ingredient_catalog_uuid")?,
        name: row.try_get("name")?,
        price: row
            .try_get::<Option<Decimal>, _>("price")?
            .map(|p| p.to_f64().unwrap_or(0.0)),
        effective_at: row.try_get("effective_at")?,
        created_at: row.try_get::<Option<i64>, _>("created_at")?,
        updated_at: row.try_get::<Option<i64>, _>("updated_at")?,
        unit_of_measure_code: row.try_get("unit_of_measure_code")?,
        unit_of_measure_name: row.try_get("unit_of_measure_name")?,
    })
}

// Mendapatkan daftar harga pasar bahan dengan paginasi dan filter
pub async fn list_ingredient_market_prices(
    db: &Pool<Postgres>,
    page: usize,
    limit: usize,
    ingredient_catalog_uuid: Option<Uuid>,
) -> Result<(Vec<IngredientMarketPriceResponse>, i64), sqlx::Error> {
    let offset = (page.saturating_sub(1) * limit) as i64;

    if let Some(ingredient_uuid) = ingredient_catalog_uuid {
        // Dengan filter ingredient_catalog_uuid
        let rows = sqlx::query(
            r#"
            SELECT uuid,
                   ingredient_catalog_uuid,
                   name,
                   price,
                   effective_at,
                   created_at,
                   updated_at,
                   unit_of_measure_code,
                   unit_of_measure_name
            FROM ingredient_market_prices
            WHERE deleted_at = 0 AND ingredient_catalog_uuid = $1
            ORDER BY effective_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(ingredient_uuid)
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(db)
        .await?;

        let count_row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM ingredient_market_prices
            WHERE deleted_at = 0 AND ingredient_catalog_uuid = $1
            "#,
        )
        .bind(ingredient_uuid)
        .fetch_one(db)
        .await?;
        let count: i64 = count_row.try_get("count")?;

        let mut items: Vec<IngredientMarketPriceResponse> = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(IngredientMarketPriceResponse {
                uuid: row.try_get("uuid")?,
                ingredient_catalog_uuid: row.try_get("ingredient_catalog_uuid")?,
                name: row.try_get("name")?,
                price: row
                    .try_get::<Option<Decimal>, _>("price")?
                    .map(|p| p.to_f64().unwrap_or(0.0)),
                effective_at: row.try_get("effective_at")?,
                created_at: row.try_get::<Option<i64>, _>("created_at")?,
                updated_at: row.try_get::<Option<i64>, _>("updated_at")?,
                unit_of_measure_code: row.try_get("unit_of_measure_code")?,
                unit_of_measure_name: row.try_get("unit_of_measure_name")?,
            });
        }

        Ok((items, count))
    } else {
        // Tanpa filter
        let rows = sqlx::query(
            r#"
            SELECT uuid,
                   ingredient_catalog_uuid,
                   name,
                   price,
                   effective_at,
                   created_at,
                   updated_at,
                   unit_of_measure_code,
                   unit_of_measure_name
            FROM ingredient_market_prices
            WHERE deleted_at = 0
            ORDER BY effective_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(db)
        .await?;

        let count_row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM ingredient_market_prices
            WHERE deleted_at = 0
            "#,
        )
        .fetch_one(db)
        .await?;
        let count: i64 = count_row.try_get("count")?;

        let mut items: Vec<IngredientMarketPriceResponse> = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(IngredientMarketPriceResponse {
                uuid: row.try_get("uuid")?,
                ingredient_catalog_uuid: row.try_get("ingredient_catalog_uuid")?,
                name: row.try_get("name")?,
                price: row
                    .try_get::<Option<Decimal>, _>("price")?
                    .map(|p| p.to_f64().unwrap_or(0.0)),
                effective_at: row.try_get("effective_at")?,
                created_at: row.try_get::<Option<i64>, _>("created_at")?,
                updated_at: row.try_get::<Option<i64>, _>("updated_at")?,
                unit_of_measure_code: row.try_get("unit_of_measure_code")?,
                unit_of_measure_name: row.try_get("unit_of_measure_name")?,
            });
        }

        Ok((items, count))
    }
}

// Mendapatkan harga pasar bahan berdasarkan UUID
pub async fn get_ingredient_market_price_by_uuid(
    db: &Pool<Postgres>,
    id: Uuid,
) -> Result<Option<IngredientMarketPriceResponse>, sqlx::Error> {
    let row_opt = sqlx::query(
        r#"
        SELECT uuid,
               ingredient_catalog_uuid,
               name,
               price,
               effective_at,
               created_at,
               updated_at,
               unit_of_measure_code,
               unit_of_measure_name
        FROM ingredient_market_prices
        WHERE uuid = $1 AND deleted_at = 0
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?;

    let resp = if let Some(row) = row_opt {
        Some(IngredientMarketPriceResponse {
            uuid: row.try_get("uuid")?,
            ingredient_catalog_uuid: row.try_get("ingredient_catalog_uuid")?,
            name: row.try_get("name")?,
            price: row
                .try_get::<Option<Decimal>, _>("price")?
                .map(|p| p.to_f64().unwrap_or(0.0)),
            effective_at: row.try_get("effective_at")?,
            created_at: row.try_get::<Option<i64>, _>("created_at")?,
            updated_at: row.try_get::<Option<i64>, _>("updated_at")?,
            unit_of_measure_code: row.try_get("unit_of_measure_code")?,
            unit_of_measure_name: row.try_get("unit_of_measure_name")?,
        })
    } else {
        None
    };

    Ok(resp)
}

// Memperbarui harga pasar bahan
pub async fn update_ingredient_market_price(
    db: &Pool<Postgres>,
    id: Uuid,
    body: UpdateIngredientMarketPriceSchema,
    timestamp_ms: i64,
) -> Result<Option<IngredientMarketPriceResponse>, sqlx::Error> {
    // Cek apakah data ada
    let existing_row_opt = sqlx::query(
        r#"
        SELECT uuid,
               ingredient_catalog_uuid,
               name,
               price,
               effective_at,
               created_at,
               updated_at,
               unit_of_measure_code,
               unit_of_measure_name
        FROM ingredient_market_prices
        WHERE uuid = $1 AND deleted_at = 0
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?;

    let Some(existing_row) = existing_row_opt else {
        return Ok(None);
    };

    // Persiapkan nilai yang akan diupdate
    let existing_ingredient_catalog_uuid: Uuid = existing_row.try_get("ingredient_catalog_uuid")?;
    let existing_name: Option<String> = existing_row.try_get("name")?;
    let existing_price_dec: Option<Decimal> = existing_row.try_get("price")?;
    let existing_effective_at: i64 = existing_row.try_get("effective_at")?;
    let mut unit_of_measure_code: Option<String> = existing_row.try_get("unit_of_measure_code")?;
    let mut unit_of_measure_name: Option<String> = existing_row.try_get("unit_of_measure_name")?;

    let ingredient_catalog_uuid = body
        .ingredient_catalog_uuid
        .unwrap_or(existing_ingredient_catalog_uuid);
    let name = body.name.or(existing_name);
    let price_f64 = body
        .price
        .or_else(|| existing_price_dec.map(|p| p.to_f64().unwrap_or(0.0)));
    let price_dec: Option<Decimal> = price_f64.and_then(Decimal::from_f64);
    let effective_at = body.effective_at.unwrap_or(existing_effective_at);

    if body.unit_of_measure_code.is_some() {
        unit_of_measure_code = body.unit_of_measure_code.clone();
    }
    if body.unit_of_measure_name.is_some() {
        unit_of_measure_name = body.unit_of_measure_name.clone();
    }

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
    let updated_row_opt = sqlx::query(
        r#"
        UPDATE ingredient_market_prices
        SET ingredient_catalog_uuid = $1,
            name = $2,
            price = $3,
            effective_at = $4,
            unit_of_measure_code = $5,
            unit_of_measure_name = $6,
            updated_at = $7
        WHERE uuid = $8 AND deleted_at = 0
        RETURNING uuid,
                  ingredient_catalog_uuid,
                  name,
                  price,
                  effective_at,
                  unit_of_measure_code,
                  unit_of_measure_name,
                  created_at,
                  updated_at
        "#,
    )
    .bind(ingredient_catalog_uuid)
    .bind(name)
    .bind(price_dec)
    .bind(effective_at)
    .bind(unit_of_measure_code.clone())
    .bind(unit_of_measure_name.clone())
    .bind(timestamp_ms)
    .bind(id)
    .fetch_optional(db)
    .await?;

    let resp = if let Some(row) = updated_row_opt {
        Some(IngredientMarketPriceResponse {
            uuid: row.try_get("uuid")?,
            ingredient_catalog_uuid: row.try_get("ingredient_catalog_uuid")?,
            name: row.try_get("name")?,
            price: row
                .try_get::<Option<Decimal>, _>("price")?
                .map(|p| p.to_f64().unwrap_or(0.0)),
            effective_at: row.try_get("effective_at")?,
            created_at: row.try_get::<Option<i64>, _>("created_at")?,
            updated_at: row.try_get::<Option<i64>, _>("updated_at")?,
            unit_of_measure_code: row.try_get("unit_of_measure_code")?,
            unit_of_measure_name: row.try_get("unit_of_measure_name")?,
        })
    } else {
        None
    };

    Ok(resp)
}

// Soft delete harga pasar bahan
pub async fn soft_delete_ingredient_market_price(
    db: &Pool<Postgres>,
    id: Uuid,
    timestamp_ms: i64,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE ingredient_market_prices
        SET deleted_at = $1
        WHERE uuid = $2 AND deleted_at = 0
        "#,
        timestamp_ms,
        id
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected() as i64)
}
