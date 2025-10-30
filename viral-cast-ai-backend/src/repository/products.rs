use rust_decimal::Decimal;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

use crate::dto::products::{CreateProductSchema, ProcessedProductSchema, UpdateProductSchema};
use crate::models::products::ProductsModel;

fn sanitize_sort_column(sort_by: Option<&str>) -> &'static str {
    match sort_by.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("name") => "p.name",
        Some("price") => "p.price",
        Some("updated_at") => "p.updated_at",
        Some("status") => "p.status",
        _ => "p.created_at",
    }
}

fn sanitize_sort_direction(sort_order: Option<&str>) -> &'static str {
    match sort_order.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    }
}

pub async fn create_product(
    db: &Pool<Postgres>,
    product_uuid: Uuid,
    body: CreateProductSchema,
    timestamp_ms: i64,
) -> sqlx::Result<ProcessedProductSchema> {
    sqlx::query!(
        r#"INSERT INTO products (uuid, category_uuid, name, sku, price, recipe_sets_uuid, status, image_url, created_at, updated_at, deleted_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
        product_uuid,
        body.category_uuid,
        body.name,
        body.sku,
        body.price,
        body.recipe_sets_uuid,
        body.status.clone().unwrap_or_else(|| "ACTIVE".to_string()),
        body.image_url,
        timestamp_ms,
        timestamp_ms,
        0i64
    )
    .execute(db)
    .await?;

    Ok(ProcessedProductSchema {
        uuid: product_uuid,
        category_uuid: body.category_uuid,
        name: body.name,
        sku: body.sku,
        price: body.price,
        recipe_sets_uuid: body.recipe_sets_uuid,
        current_recipe_name: None,
        current_recipe_yield_qty: None,
        status: body.status.unwrap_or_else(|| "ACTIVE".to_string()),
        image_url: body.image_url,
        created_at: Some(timestamp_ms),
        updated_at: Some(timestamp_ms),
    })
}

pub async fn list_products(
    db: &Pool<Postgres>,
    search: Option<String>,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    sort_order: Option<String>,
) -> sqlx::Result<(Vec<ProcessedProductSchema>, i64)> {
    let page = if page == 0 { 1 } else { page };
    let limit = if limit == 0 { 50 } else { limit };
    let offset = (page - 1) * limit;
    let search_pattern = search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s));
    let search_pattern_for_count = search_pattern.clone();

    let sort_column = sanitize_sort_column(sort_by.as_deref());
    let sort_direction = sanitize_sort_direction(sort_order.as_deref());

    let base_query = r#"SELECT 
            p.uuid, 
            p.category_uuid, 
            p.name, 
            p.sku, 
            p.price, 
            p.recipe_sets_uuid, 
            p.status, 
            p.image_url, 
            p.created_at, 
            p.updated_at, 
            p.deleted_at, 
            rs.name as recipe_name, 
            rs.yield_quantity as recipe_yield_qty 
        FROM products p 
        LEFT JOIN recipe_sets rs ON p.recipe_sets_uuid = rs.uuid AND rs.deleted_at = 0 
        WHERE p.deleted_at = 0
          AND ($1::text IS NULL OR p.name ILIKE $1 OR p.sku ILIKE $1)
        ORDER BY {sort_column} {sort_direction} 
        LIMIT $2 OFFSET $3"#;

    let query = base_query
        .replace("{sort_column}", sort_column)
        .replace("{sort_direction}", sort_direction);

    let rows = sqlx::query(&query)
        .bind(search_pattern.clone())
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(db)
        .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM products p WHERE p.deleted_at = 0 AND ($1::text IS NULL OR p.name ILIKE $1 OR p.sku ILIKE $1)"#,
    )
    .bind(search_pattern_for_count)
    .fetch_one(db)
    .await?;

    let products = rows
        .into_iter()
        .map(|row| ProcessedProductSchema {
            uuid: row.get("uuid"),
            category_uuid: row.get("category_uuid"),
            name: row.get("name"),
            sku: row.get("sku"),
            price: row.get("price"),
            recipe_sets_uuid: row.get("recipe_sets_uuid"),
            current_recipe_name: row.get::<Option<String>, _>("recipe_name"),
            current_recipe_yield_qty: row.get::<Option<Decimal>, _>("recipe_yield_qty"),
            status: row.get("status"),
            image_url: row.get("image_url"),
            created_at: row.get::<Option<i64>, _>("created_at").or(Some(0)),
            updated_at: row.get::<Option<i64>, _>("updated_at").or(Some(0)),
        })
        .collect();

    Ok((products, total))
}

pub async fn get_product_by_uuid(
    db: &Pool<Postgres>,
    id: Uuid,
) -> sqlx::Result<Option<ProcessedProductSchema>> {
    let row = sqlx::query!(
        r#"SELECT 
            p.uuid, 
            p.category_uuid, 
            p.name, 
            p.sku, 
            p.price, 
            p.recipe_sets_uuid, 
            p.status, 
            p.image_url, 
            p.created_at, 
            p.updated_at, 
            p.deleted_at, 
            rs.name as "recipe_name: Option<String>", 
            rs.yield_quantity as "recipe_yield_qty: Option<Decimal>" 
        FROM products p 
        LEFT JOIN recipe_sets rs ON p.recipe_sets_uuid = rs.uuid AND rs.deleted_at = 0 
        WHERE p.uuid = $1 AND p.deleted_at = 0"#,
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| ProcessedProductSchema {
        uuid: r.uuid,
        category_uuid: r.category_uuid,
        name: r.name,
        sku: r.sku,
        price: r.price,
        recipe_sets_uuid: r.recipe_sets_uuid,
        current_recipe_name: r.recipe_name,
        current_recipe_yield_qty: r.recipe_yield_qty.flatten(),
        status: r.status,
        image_url: r.image_url,
        created_at: r.created_at.or(Some(0)),
        updated_at: r.updated_at.or(Some(0)),
    }))
}

pub async fn update_product(
    db: &Pool<Postgres>,
    id: Uuid,
    body: UpdateProductSchema,
    timestamp_ms: i64,
) -> sqlx::Result<Option<ProcessedProductSchema>> {
    // Fetch existing product
    let existing = sqlx::query_as!(
        ProductsModel,
        r#"SELECT uuid, category_uuid, name, sku, price, recipe_sets_uuid, status, image_url, created_at, updated_at, deleted_at
           FROM products WHERE uuid = $1 AND deleted_at = 0"#,
        id
    )
    .fetch_optional(db)
    .await?;

    let Some(current) = existing else {
        return Ok(None);
    };

    let new_category_uuid = body.category_uuid.unwrap_or(current.category_uuid);
    let new_name = body.name.unwrap_or(current.name);
    let new_sku = body.sku.or(current.sku);
    let new_price = body.price.unwrap_or(current.price);
    let new_recipe_uuid = body.recipe_sets_uuid.or(current.recipe_sets_uuid);
    let new_status = body.status.unwrap_or(current.status);
    let new_image_url = body.image_url.or(current.image_url);

    let res = sqlx::query!(
        r#"UPDATE products SET 
            category_uuid = $1,
            name = $2,
            sku = $3,
            price = $4,
            recipe_sets_uuid = $5,
            status = $6,
            image_url = $7,
            updated_at = $8
        WHERE uuid = $9 AND deleted_at = 0"#,
        new_category_uuid,
        new_name,
        new_sku,
        new_price,
        new_recipe_uuid,
        new_status,
        new_image_url,
        timestamp_ms,
        id
    )
    .execute(db)
    .await?;

    if res.rows_affected() == 0 {
        return Ok(None);
    }

    Ok(Some(ProcessedProductSchema {
        uuid: id,
        category_uuid: new_category_uuid,
        name: new_name,
        sku: new_sku,
        price: new_price,
        recipe_sets_uuid: new_recipe_uuid,
        current_recipe_name: None,
        current_recipe_yield_qty: None,
        status: new_status,
        image_url: new_image_url,
        created_at: current.created_at.or(Some(0)),
        updated_at: Some(timestamp_ms),
    }))
}

pub async fn soft_delete_product(
    db: &Pool<Postgres>,
    id: Uuid,
    timestamp_ms: i64,
) -> sqlx::Result<u64> {
    let res = sqlx::query!(
        "UPDATE products SET deleted_at = $1 WHERE uuid = $2 AND deleted_at = 0",
        timestamp_ms,
        id
    )
    .execute(db)
    .await?;

    Ok(res.rows_affected())
}
