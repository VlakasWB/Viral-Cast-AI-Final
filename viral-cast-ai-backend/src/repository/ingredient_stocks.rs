use chrono::Utc;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::dto::ingredient_stocks::{
    CreateIngredientStockSchema, GetIngredientStockSchema, IngredientStockResponse,
    UpdateIngredientStockSchema,
};
use crate::models::ingredient_stocks::IngredientStockModel;

fn map_model_to_response(model: IngredientStockModel) -> IngredientStockResponse {
    IngredientStockResponse {
        uuid: model.uuid,
        ingredient_stock_move_uuid: model.ingredient_stock_moves_uuid,
        ingredient_catalog_uuid: model.ingredient_catalog_uuid,
        ingredient_name: model.ingredient_name,
        unit_of_measure_code: model.unit_of_measure_code,
        unit_of_measure_name: model.unit_of_measure_name,
        total_quantity: model.total_quantity,
        total_value: model.total_value,
        current_cost: model.current_cost,
        avg_cost: model.avg_cost,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

fn sanitize_sort_column(sort_by: Option<&str>) -> &'static str {
    match sort_by.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("total_quantity") => "s.total_quantity",
        Some("total_value") => "s.total_value",
        Some("updated_at") => "s.updated_at",
        _ => "s.created_at",
    }
}

fn sanitize_sort_direction(sort_order: Option<&str>) -> &'static str {
    match sort_order.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    }
}

pub async fn create_ingredient_stock(
    db: &Pool<Postgres>,
    body: &CreateIngredientStockSchema,
) -> Result<IngredientStockResponse, sqlx::Error> {
    let uuid = Uuid::new_v4();
    let now = Utc::now().timestamp_millis();

    let move_row = sqlx::query!(
        r#"
        SELECT ingredient_catalog_uuid,
               unit_of_measure_code,
               unit_of_measure_name
        FROM ingredient_stock_moves
        WHERE uuid = $1 AND deleted_at = 0
        "#,
        body.ingredient_stock_move_uuid
    )
    .fetch_optional(db)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;

    let mut unit_of_measure_code = body
        .unit_of_measure_code
        .clone()
        .or(move_row.unit_of_measure_code.clone());
    let mut unit_of_measure_name = body
        .unit_of_measure_name
        .clone()
        .or(move_row.unit_of_measure_name.clone());

    if unit_of_measure_code.is_none() || unit_of_measure_name.is_none() {
        if let Some(row) = sqlx::query!(
            r#"
            SELECT uom.code AS "code?", uom.name AS "name?"
            FROM ingredient_catalog ic
            LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
            WHERE ic.uuid = $1
            "#,
            move_row.ingredient_catalog_uuid
        )
        .fetch_optional(db)
        .await?
        {
            if unit_of_measure_code.is_none() {
                unit_of_measure_code = row.code;
            }
            if unit_of_measure_name.is_none() {
                unit_of_measure_name = row.name;
            }
        }
    }

    sqlx::query!(
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
        "#,
        uuid,
        body.ingredient_stock_move_uuid,
        body.total_quantity.unwrap_or_else(Decimal::zero),
        body.total_value.unwrap_or_else(Decimal::zero),
        body.current_cost,
        body.avg_cost,
        unit_of_measure_code.clone(),
        unit_of_measure_name.clone(),
        now
    )
    .execute(db)
    .await?;

    get_ingredient_stock_by_uuid(db, uuid)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn get_ingredient_stocks(
    db: &Pool<Postgres>,
    params: &GetIngredientStockSchema,
) -> Result<(Vec<IngredientStockResponse>, i64), sqlx::Error> {
    let limit = params.limit.unwrap_or(10) as i64;
    let offset = (params.page.unwrap_or(1) as i64 - 1) * limit;
    let include_deleted = params.include_deleted.unwrap_or(false);
    let search_pattern = params
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s));

    let search_pattern_for_count = search_pattern.clone();

    let sort_column = sanitize_sort_column(params.sort_by.as_deref());
    let sort_direction = sanitize_sort_direction(params.sort_order.as_deref());

    let list_query = format!(
        r#"
        SELECT
            s.uuid,
            s.ingredient_stock_moves_uuid,
            m.ingredient_catalog_uuid,
            ic.name as ingredient_name,
            s.unit_of_measure_code,
            s.unit_of_measure_name,
            s.total_quantity,
            s.total_value,
            s.current_cost,
            s.avg_cost,
            s.created_at,
            s.updated_at,
            s.deleted_at
        FROM ingredient_stocks s
        JOIN ingredient_stock_moves m ON s.ingredient_stock_moves_uuid = m.uuid
        LEFT JOIN ingredient_catalog ic ON m.ingredient_catalog_uuid = ic.uuid
        WHERE (s.deleted_at = 0 OR $4)
          AND ($3::uuid IS NULL OR m.ingredient_catalog_uuid = $3)
          AND ($5::text IS NULL OR ic.name ILIKE $5)
        ORDER BY {sort_column} {sort_direction}
        LIMIT $1 OFFSET $2
        "#,
    );

    let rows = sqlx::query_as::<_, IngredientStockModel>(&list_query)
        .bind(limit)
        .bind(offset)
        .bind(params.ingredient_catalog_uuid)
        .bind(include_deleted)
        .bind(search_pattern.clone())
        .fetch_all(db)
        .await?;

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM ingredient_stocks s
        JOIN ingredient_stock_moves m ON s.ingredient_stock_moves_uuid = m.uuid
        LEFT JOIN ingredient_catalog ic ON m.ingredient_catalog_uuid = ic.uuid
        WHERE (s.deleted_at = 0 OR $2)
          AND ($1::uuid IS NULL OR m.ingredient_catalog_uuid = $1)
          AND ($3::text IS NULL OR ic.name ILIKE $3)
        "#,
    )
    .bind(params.ingredient_catalog_uuid)
    .bind(include_deleted)
    .bind(search_pattern_for_count)
    .fetch_one(db)
    .await?;

    Ok((rows.into_iter().map(map_model_to_response).collect(), count))
}

pub async fn get_ingredient_stock_by_uuid(
    db: &Pool<Postgres>,
    id: Uuid,
) -> Result<Option<IngredientStockResponse>, sqlx::Error> {
    let row = sqlx::query_as!(
        IngredientStockModel,
        r#"
        SELECT
            s.uuid,
            s.ingredient_stock_moves_uuid,
            m.ingredient_catalog_uuid,
            ic.name as "ingredient_name?",
            s.unit_of_measure_code as "unit_of_measure_code?",
            s.unit_of_measure_name as "unit_of_measure_name?",
            s.total_quantity,
            s.total_value,
            s.current_cost,
            s.avg_cost,
            s.created_at,
            s.updated_at,
            s.deleted_at
        FROM ingredient_stocks s
        JOIN ingredient_stock_moves m ON s.ingredient_stock_moves_uuid = m.uuid
        LEFT JOIN ingredient_catalog ic ON m.ingredient_catalog_uuid = ic.uuid
        WHERE s.uuid = $1
        LIMIT 1
        "#,
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(map_model_to_response))
}

pub async fn update_ingredient_stock(
    db: &Pool<Postgres>,
    id: Uuid,
    body: &UpdateIngredientStockSchema,
) -> Result<IngredientStockResponse, sqlx::Error> {
    let now = Utc::now().timestamp_millis();

    let current = get_ingredient_stock_by_uuid(db, id).await?;
    let current = current.ok_or(sqlx::Error::RowNotFound)?;

    let ingredient_stock_move_uuid = body
        .ingredient_stock_move_uuid
        .unwrap_or(current.ingredient_stock_move_uuid);
    let total_quantity = body.total_quantity.unwrap_or(current.total_quantity);
    let total_value = body.total_value.unwrap_or(current.total_value);
    let current_cost = body.current_cost.or(current.current_cost);
    let avg_cost = body.avg_cost.or(current.avg_cost);
    let mut unit_of_measure_code = body
        .unit_of_measure_code
        .clone()
        .or(current.unit_of_measure_code.clone());
    let mut unit_of_measure_name = body
        .unit_of_measure_name
        .clone()
        .or(current.unit_of_measure_name.clone());

    if unit_of_measure_code.is_none() || unit_of_measure_name.is_none() {
        if let Some(row) = sqlx::query!(
            r#"
            SELECT ingredient_catalog_uuid,
                   unit_of_measure_code,
                   unit_of_measure_name
            FROM ingredient_stock_moves
            WHERE uuid = $1 AND deleted_at = 0
            "#,
            ingredient_stock_move_uuid
        )
        .fetch_optional(db)
        .await?
        {
            if unit_of_measure_code.is_none() {
                unit_of_measure_code = row.unit_of_measure_code.clone();
            }
            if unit_of_measure_name.is_none() {
                unit_of_measure_name = row.unit_of_measure_name.clone();
            }

            if unit_of_measure_code.is_none() || unit_of_measure_name.is_none() {
                if let Some(cat_row) = sqlx::query!(
                    r#"
                    SELECT uom.code AS "code?", uom.name AS "name?"
                    FROM ingredient_catalog ic
                    LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
                    WHERE ic.uuid = $1
                    "#,
                    row.ingredient_catalog_uuid
                )
                .fetch_optional(db)
                .await?
                {
                    if unit_of_measure_code.is_none() {
                        unit_of_measure_code = cat_row.code;
                    }
                    if unit_of_measure_name.is_none() {
                        unit_of_measure_name = cat_row.name;
                    }
                }
            }
        }
    }

    sqlx::query!(
        r#"
        UPDATE ingredient_stocks
        SET ingredient_stock_moves_uuid = $1,
            total_quantity = $2,
            total_value = $3,
            current_cost = $4,
            avg_cost = $5,
            unit_of_measure_code = $6,
            unit_of_measure_name = $7,
            updated_at = $8
        WHERE uuid = $9
        "#,
        ingredient_stock_move_uuid,
        total_quantity,
        total_value,
        current_cost,
        avg_cost,
        unit_of_measure_code.clone(),
        unit_of_measure_name.clone(),
        now,
        id
    )
    .execute(db)
    .await?;

    get_ingredient_stock_by_uuid(db, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn delete_ingredient_stock(db: &Pool<Postgres>, id: Uuid) -> Result<(), sqlx::Error> {
    let now = Utc::now().timestamp_millis();

    sqlx::query!(
        r#"
        UPDATE ingredient_stocks
        SET deleted_at = $1
        WHERE uuid = $2 AND deleted_at = 0
        "#,
        now,
        id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn recompute_stock_for_ingredient(
    db: &Pool<Postgres>,
    ingredient_catalog_uuid: Uuid,
) -> Result<(), sqlx::Error> {
    let moves = sqlx::query!(
        r#"
        SELECT uuid, quantity, price, ref_type, unit_of_measure_code, unit_of_measure_name
        FROM ingredient_stock_moves
        WHERE ingredient_catalog_uuid = $1 AND deleted_at = 0
        ORDER BY effective_at ASC, created_at ASC
        "#,
        ingredient_catalog_uuid
    )
    .fetch_all(db)
    .await?;

    let mut total_quantity = Decimal::ZERO;
    let mut total_value = Decimal::ZERO;
    let mut avg_cost: Option<Decimal> = None;
    let mut current_cost: Option<Decimal> = None;
    let mut latest_move_uuid: Option<Uuid> = None;
    let mut unit_of_measure_code: Option<String> = None;
    let mut unit_of_measure_name: Option<String> = None;

    for record in moves {
        let quantity = record.quantity;
        if quantity.is_zero() {
            continue;
        }

        let delta = signed_quantity(record.ref_type.as_deref(), quantity);
        if delta.is_zero() {
            continue;
        }

        let price = record.price;
        let abs_delta = delta.abs();

        if delta > Decimal::ZERO {
            let unit_cost = price.or(avg_cost).unwrap_or(Decimal::ZERO);
            total_quantity += abs_delta;
            total_value += abs_delta * unit_cost;
            if price.is_some() {
                current_cost = price;
            }
        } else {
            let unit_cost = price.or(avg_cost).unwrap_or(Decimal::ZERO);
            if total_quantity > abs_delta {
                total_quantity -= abs_delta;
                total_value -= abs_delta * unit_cost;
            } else {
                total_quantity = Decimal::ZERO;
                total_value = Decimal::ZERO;
            }
        }

        if total_quantity > Decimal::ZERO {
            avg_cost = Some((total_value / total_quantity).round_dp(4));
        } else {
            avg_cost = None;
            total_value = Decimal::ZERO;
        }

        latest_move_uuid = Some(record.uuid);

        if record.unit_of_measure_code.is_some() {
            unit_of_measure_code = record.unit_of_measure_code.clone();
        }
        if record.unit_of_measure_name.is_some() {
            unit_of_measure_name = record.unit_of_measure_name.clone();
        }
    }

    if unit_of_measure_code.is_none() || unit_of_measure_name.is_none() {
        if let Some(row) = sqlx::query!(
            r#"
            SELECT uom.code AS "code?", uom.name AS "name?"
            FROM ingredient_catalog ic
            LEFT JOIN units_of_measure uom ON ic.unit_of_measure_uuid = uom.uuid
            WHERE ic.uuid = $1
            "#,
            ingredient_catalog_uuid
        )
        .fetch_optional(db)
        .await?
        {
            if unit_of_measure_code.is_none() {
                unit_of_measure_code = row.code;
            }
            if unit_of_measure_name.is_none() {
                unit_of_measure_name = row.name;
            }
        }
    }

    if current_cost.is_none() && avg_cost.is_some() {
        current_cost = avg_cost;
    }

    let now = Utc::now().timestamp_millis();

    if let Some(latest_uuid) = latest_move_uuid {
        let total_quantity = total_quantity.max(Decimal::ZERO);
        let mut total_value = total_value.max(Decimal::ZERO);

        if total_quantity.is_zero() {
            total_value = Decimal::ZERO;
        }

        let existing = sqlx::query!(
            r#"
            SELECT s.uuid
            FROM ingredient_stocks s
            JOIN ingredient_stock_moves m ON s.ingredient_stock_moves_uuid = m.uuid
            WHERE m.ingredient_catalog_uuid = $1
              AND s.deleted_at = 0
            ORDER BY s.updated_at DESC
            LIMIT 1
            "#,
            ingredient_catalog_uuid
        )
        .fetch_optional(db)
        .await?;

        let active_uuid = if let Some(existing) = existing {
            sqlx::query!(
                r#"
                UPDATE ingredient_stocks
                SET ingredient_stock_moves_uuid = $1,
                    total_quantity = $2,
                    total_value = $3,
                    current_cost = $4,
                    avg_cost = $5,
                    unit_of_measure_code = $6,
                    unit_of_measure_name = $7,
                    updated_at = $8,
                    deleted_at = 0
                WHERE uuid = $9
                "#,
                latest_uuid,
                total_quantity,
                total_value,
                current_cost,
                avg_cost,
                unit_of_measure_code.clone(),
                unit_of_measure_name.clone(),
                now,
                existing.uuid
            )
            .execute(db)
            .await?;
            existing.uuid
        } else {
            let stock_uuid = Uuid::new_v4();
            sqlx::query!(
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
                "#,
                stock_uuid,
                latest_uuid,
                total_quantity,
                total_value,
                current_cost,
                avg_cost,
                unit_of_measure_code.clone(),
                unit_of_measure_name.clone(),
                now
            )
            .execute(db)
            .await?;
            stock_uuid
        };

        sqlx::query!(
            r#"
            UPDATE ingredient_stocks
            SET deleted_at = $2
            WHERE uuid <> $1
              AND ingredient_stock_moves_uuid IN (
                  SELECT uuid FROM ingredient_stock_moves
                  WHERE ingredient_catalog_uuid = $3
              )
            "#,
            active_uuid,
            now,
            ingredient_catalog_uuid
        )
        .execute(db)
        .await?;
    } else {
        sqlx::query!(
            r#"
            UPDATE ingredient_stocks
            SET deleted_at = $2
            WHERE ingredient_stock_moves_uuid IN (
                SELECT uuid FROM ingredient_stock_moves
                WHERE ingredient_catalog_uuid = $1
            )
            "#,
            ingredient_catalog_uuid,
            now
        )
        .execute(db)
        .await?;
    }

    Ok(())
}

fn signed_quantity(ref_type: Option<&str>, quantity: Decimal) -> Decimal {
    match ref_type.map(|s| s.to_ascii_uppercase()).as_deref() {
        Some("PRODUCTION") | Some("WASTE") => -quantity.abs(),
        Some("ADJUSTMENT") => quantity,
        Some("RETURN") | Some("PURCHASE") => quantity.abs(),
        _ => quantity,
    }
}
