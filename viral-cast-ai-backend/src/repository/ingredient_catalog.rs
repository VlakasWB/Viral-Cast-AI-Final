use rust_decimal::Decimal;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::dto::ingredient_catalog::{
    CreateIngredientSchema, ProcessedIngredientSchema, UomInfo, UpdateIngredientSchema,
};
use crate::models::ingredient_catalog::IngredientCatalogModel;

use sqlx::Row;

fn sanitize_sort_column(sort_by: Option<&str>) -> &'static str {
    match sort_by.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("name") => "i.name",
        Some("updated_at") => "i.updated_at",
        _ => "i.created_at",
    }
}

fn sanitize_sort_direction(sort_order: Option<&str>) -> &'static str {
    match sort_order.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    }
}

// Create ingredient and return processed schema with UOM info
pub async fn create_ingredient(
    db: &Pool<Postgres>,
    ingredient_uuid: Uuid,
    body: CreateIngredientSchema,
    timestamp_ms: i64,
) -> Result<ProcessedIngredientSchema, sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO ingredient_catalog (uuid, name, unit_of_measure_uuid, minimum_stock, shelf_life_days, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(ingredient_uuid)
    .bind(body.name.to_string())
    .bind(body.base_uom_uuid)
    .bind(body.minimum_stock)
    .bind(body.shelf_life_days)
    .bind(timestamp_ms)
    .bind(timestamp_ms)
    .execute(db)
    .await?;

    // Fetch UOM info
    let uom_row = sqlx::query(r#"SELECT code, name FROM units_of_measure WHERE uuid = $1"#)
        .bind(body.base_uom_uuid)
        .fetch_optional(db)
        .await?;

    let uom_info = match uom_row {
        Some(row) => UomInfo {
            uuid: body.base_uom_uuid,
            code: row.get::<Option<String>, _>("code").unwrap_or_default(),
            name: row.get::<Option<String>, _>("name").unwrap_or_default(),
        },
        None => UomInfo {
            uuid: body.base_uom_uuid,
            code: String::new(),
            name: String::new(),
        },
    };

    Ok(ProcessedIngredientSchema {
        uuid: ingredient_uuid,
        name: body.name,
        base_uom_uuid: uom_info,
        minimum_stock: body.minimum_stock,
        shelf_life_days: body.shelf_life_days,
        created_at: Some(timestamp_ms),
        updated_at: Some(timestamp_ms),
    })
}

// List ingredients with pagination and search; returns items and total count
pub async fn list_ingredients(
    db: &Pool<Postgres>,
    page: usize,
    limit: usize,
    search: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
) -> Result<(Vec<ProcessedIngredientSchema>, i64), sqlx::Error> {
    let offset = (page.saturating_sub(1) * limit) as i64;
    let search_pattern = search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s));
    let search_pattern_for_count = search_pattern.clone();

    let sort_column = sanitize_sort_column(sort_by.as_deref());
    let sort_direction = sanitize_sort_direction(sort_order.as_deref());

    let base_query = r#"SELECT 
            i.uuid, 
            i.name, 
            i.unit_of_measure_uuid as base_uom_uuid, 
            i.minimum_stock,
            i.shelf_life_days, 
            i.created_at, 
            i.updated_at,
            u.code as uom_code,
            u.name as uom_name
        FROM ingredient_catalog i
        JOIN units_of_measure u ON i.unit_of_measure_uuid = u.uuid
        WHERE i.deleted_at = 0 
          AND (
                $1::text IS NULL 
                OR i.name ILIKE $1 
                OR COALESCE(u.code, '') ILIKE $1
                OR COALESCE(u.name, '') ILIKE $1
          )"#;
    let query = format!("{base_query} ORDER BY {sort_column} {sort_direction} LIMIT $2 OFFSET $3");

    let rows = sqlx::query(&query)
        .bind(search_pattern.clone())
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(db)
        .await?;

    let items: Vec<ProcessedIngredientSchema> = rows
        .into_iter()
        .map(|r| ProcessedIngredientSchema {
            uuid: r.get::<Uuid, _>("uuid"),
            name: r.get::<String, _>("name"),
            base_uom_uuid: UomInfo {
                uuid: r.get::<Uuid, _>("base_uom_uuid"),
                code: r.get::<Option<String>, _>("uom_code").unwrap_or_default(),
                name: r.get::<Option<String>, _>("uom_name").unwrap_or_default(),
            },
            minimum_stock: r.get::<Option<Decimal>, _>("minimum_stock"),
            shelf_life_days: r.get::<Option<i32>, _>("shelf_life_days"),
            created_at: r.get::<Option<i64>, _>("created_at").or(Some(0)),
            updated_at: r.get::<Option<i64>, _>("updated_at").or(Some(0)),
        })
        .collect();

    let count_row = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM ingredient_catalog i
        JOIN units_of_measure u ON i.unit_of_measure_uuid = u.uuid
        WHERE i.deleted_at = 0
          AND (
                $1::text IS NULL 
                OR i.name ILIKE $1 
                OR COALESCE(u.code, '') ILIKE $1
                OR COALESCE(u.name, '') ILIKE $1
          )
        "#,
    )
    .bind(search_pattern_for_count)
    .fetch_one(db)
    .await?;

    let total: i64 = count_row.get::<i64, _>("count");

    Ok((items, total))
}

// Get single ingredient by UUID
pub async fn get_ingredient_by_uuid(
    db: &Pool<Postgres>,
    id: Uuid,
) -> Result<Option<ProcessedIngredientSchema>, sqlx::Error> {
    let row = sqlx::query(
        r#"SELECT 
            i.uuid, 
            i.name, 
            i.unit_of_measure_uuid as base_uom_uuid, 
            i.minimum_stock,
            i.shelf_life_days, 
            i.created_at, 
            i.updated_at,
            u.code as uom_code,
            u.name as uom_name
        FROM ingredient_catalog i
        JOIN units_of_measure u ON i.unit_of_measure_uuid = u.uuid
        WHERE i.uuid = $1 AND i.deleted_at = 0"#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?;

    Ok(row.map(|ingredient| ProcessedIngredientSchema {
        uuid: ingredient.get::<Uuid, _>("uuid"),
        name: ingredient.get::<String, _>("name"),
        base_uom_uuid: UomInfo {
            uuid: ingredient.get::<Uuid, _>("base_uom_uuid"),
            code: ingredient
                .get::<Option<String>, _>("uom_code")
                .unwrap_or_default(),
            name: ingredient
                .get::<Option<String>, _>("uom_name")
                .unwrap_or_default(),
        },
        minimum_stock: ingredient.get::<Option<Decimal>, _>("minimum_stock"),
        shelf_life_days: ingredient.get::<Option<i32>, _>("shelf_life_days"),
        created_at: ingredient.get::<Option<i64>, _>("created_at").or(Some(0)),
        updated_at: ingredient.get::<Option<i64>, _>("updated_at").or(Some(0)),
    }))
}

// Update ingredient and return processed schema
pub async fn update_ingredient(
    db: &Pool<Postgres>,
    id: Uuid,
    body: UpdateIngredientSchema,
    timestamp_ms: i64,
) -> Result<Option<ProcessedIngredientSchema>, sqlx::Error> {
    // Fetch current ingredient
    let current_row = sqlx::query(
        r#"SELECT uuid, name, unit_of_measure_uuid, minimum_stock, shelf_life_days, created_at, updated_at, deleted_at FROM ingredient_catalog WHERE uuid = $1 AND deleted_at = 0"#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?;

    let existing = match current_row {
        Some(r) => IngredientCatalogModel {
            uuid: r.get::<Uuid, _>("uuid"),
            name: r.get::<String, _>("name"),
            base_uom_uuid: r.get::<Uuid, _>("unit_of_measure_uuid"),
            minimum_stock: r.get::<Option<Decimal>, _>("minimum_stock"),
            shelf_life_days: r.get::<Option<i32>, _>("shelf_life_days"),
            created_at: r.get::<Option<i64>, _>("created_at"),
            updated_at: r.get::<Option<i64>, _>("updated_at"),
            deleted_at: r.get::<Option<i64>, _>("deleted_at"),
        },
        None => return Ok(None),
    };

    let new_name = body.name.unwrap_or(existing.name);
    let new_base_uom_uuid = body.base_uom_uuid.unwrap_or(existing.base_uom_uuid);
    let new_minimum_stock: Option<Decimal> = body.minimum_stock.or(existing.minimum_stock);
    let new_shelf_life_days: Option<i32> = body.shelf_life_days.or(existing.shelf_life_days);

    let result = sqlx::query(
        r#"UPDATE ingredient_catalog SET name = $1, unit_of_measure_uuid = $2, minimum_stock = $3, shelf_life_days = $4, updated_at = $5 WHERE uuid = $6 AND deleted_at = 0"#,
    )
    .bind(new_name.clone())
    .bind(new_base_uom_uuid)
    .bind(new_minimum_stock)
    .bind(new_shelf_life_days)
    .bind(timestamp_ms)
    .bind(id)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    // Fetch UOM info for final base_uom_uuid
    let uom_row = sqlx::query(r#"SELECT code, name FROM units_of_measure WHERE uuid = $1"#)
        .bind(new_base_uom_uuid)
        .fetch_optional(db)
        .await?;

    let uom_info = match uom_row {
        Some(uom) => UomInfo {
            uuid: new_base_uom_uuid,
            code: uom.get::<Option<String>, _>("code").unwrap_or_default(),
            name: uom.get::<Option<String>, _>("name").unwrap_or_default(),
        },
        None => UomInfo {
            uuid: new_base_uom_uuid,
            code: String::new(),
            name: String::new(),
        },
    };

    Ok(Some(ProcessedIngredientSchema {
        uuid: existing.uuid,
        name: new_name,
        base_uom_uuid: uom_info,
        minimum_stock: new_minimum_stock,
        shelf_life_days: new_shelf_life_days,
        created_at: existing.created_at.or(Some(0)),
        updated_at: Some(timestamp_ms),
    }))
}

// Soft delete ingredient
pub async fn soft_delete_ingredient(
    db: &Pool<Postgres>,
    id: Uuid,
    deleted_at: i64,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"UPDATE ingredient_catalog SET deleted_at = $1 WHERE uuid = $2 AND deleted_at = 0"#,
    )
    .bind(deleted_at)
    .bind(id)
    .execute(db)
    .await?;

    Ok(res.rows_affected())
}
