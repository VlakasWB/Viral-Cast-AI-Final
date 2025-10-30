use rust_decimal::Decimal;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::dto::recipe_sets::{
    CreateRecipeSetSchema, GetRecipeSetSchema, ProcessedRecipeSetSchema, UpdateRecipeSetSchema,
};
use crate::models::recipe_sets::RecipeSetsModel;

fn resolve_yield_quantity(value: Option<Decimal>) -> Decimal {
    value.unwrap_or_else(|| Decimal::from(1))
}

fn sanitize_sort_column(sort_by: Option<&str>) -> &'static str {
    match sort_by.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("name") => "name",
        Some("updated_at") => "updated_at",
        Some("yield_quantity") => "yield_quantity",
        _ => "created_at",
    }
}

fn sanitize_sort_direction(sort_order: Option<&str>) -> &'static str {
    match sort_order.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    }
}

// Create recipe set and return processed schema
pub async fn create_recipe_set(
    db: &Pool<Postgres>,
    recipe_set_uuid: Uuid,
    body: CreateRecipeSetSchema,
    timestamp_ms: i64,
) -> Result<ProcessedRecipeSetSchema, sqlx::Error> {
    let yield_quantity = body.yield_quantity.unwrap_or(Decimal::from(1));
    let is_active = body.is_active.unwrap_or(true);

    sqlx::query!(
        r#"INSERT INTO recipe_sets (uuid, name, yield_quantity, effective_from, effective_to, is_active, created_at, updated_at, deleted_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        recipe_set_uuid,
        body.name,
        yield_quantity,
        body.effective_from,
        body.effective_to,
        is_active,
        timestamp_ms,
        timestamp_ms,
        0i64
    )
    .execute(db)
    .await?;

    Ok(ProcessedRecipeSetSchema {
        uuid: recipe_set_uuid,
        name: body.name,
        yield_quantity,
        effective_from: body.effective_from,
        effective_to: body.effective_to,
        is_active,
        created_at: Some(timestamp_ms),
        updated_at: Some(timestamp_ms),
    })
}

// List recipe sets with basic pagination (no filtering yet)
pub async fn list_recipe_sets(
    db: &Pool<Postgres>,
    opts: GetRecipeSetSchema,
) -> Result<(Vec<ProcessedRecipeSetSchema>, i64), sqlx::Error> {
    let page = opts.page.unwrap_or(1).max(1);
    let limit = opts.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;
    // Support explicit name filter, falling back to search if not provided
    let name_term = opts
        .name
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let search_term = opts
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let search_effective = name_term.or(search_term);
    let search_pattern = search_effective.map(|s| format!("%{}%", s));

    let mut data_builder = sqlx::QueryBuilder::<Postgres>::new(
        r#"SELECT uuid, name, yield_quantity, effective_from, effective_to, is_active, created_at, updated_at, deleted_at
           FROM recipe_sets
           WHERE deleted_at = 0"#,
    );
    let mut count_builder = sqlx::QueryBuilder::<Postgres>::new(
        r#"SELECT COUNT(*) FROM recipe_sets WHERE deleted_at = 0"#,
    );

    if let Some(pattern) = search_pattern.as_ref() {
        data_builder
            .push(" AND name ILIKE ")
            .push_bind(pattern.clone());
        count_builder
            .push(" AND name ILIKE ")
            .push_bind(pattern.clone());
    }

    if let Some(is_active) = opts.is_active {
        data_builder.push(" AND is_active = ").push_bind(is_active);
        count_builder.push(" AND is_active = ").push_bind(is_active);
    }

    let sort_column = sanitize_sort_column(opts.sort_by.as_deref());
    let sort_direction = sanitize_sort_direction(opts.sort_order.as_deref());

    data_builder
        .push(" ORDER BY ")
        .push(sort_column)
        .push(" ")
        .push(sort_direction)
        .push(" LIMIT ")
        .push_bind(limit as i64)
        .push(" OFFSET ")
        .push_bind(offset as i64);

    let rows = data_builder
        .build_query_as::<RecipeSetsModel>()
        .fetch_all(db)
        .await?;

    let total: i64 = count_builder.build_query_scalar().fetch_one(db).await?;

    let recipe_sets = rows
        .into_iter()
        .map(|row| ProcessedRecipeSetSchema {
            uuid: row.uuid,
            name: row.name,
            yield_quantity: resolve_yield_quantity(row.yield_quantity),
            effective_from: row.effective_from,
            effective_to: row.effective_to,
            is_active: row.is_active,
            created_at: row.created_at.or(Some(0)),
            updated_at: row.updated_at.or(Some(0)),
        })
        .collect();

    Ok((recipe_sets, total))
}

pub async fn get_recipe_set_by_uuid(
    db: &Pool<Postgres>,
    id: Uuid,
) -> Result<Option<ProcessedRecipeSetSchema>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT uuid, name, yield_quantity, effective_from, effective_to, is_active, created_at, updated_at, deleted_at
           FROM recipe_sets WHERE uuid = $1 AND deleted_at = 0"#,
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| ProcessedRecipeSetSchema {
        uuid: r.uuid,
        name: r.name,
        yield_quantity: resolve_yield_quantity(r.yield_quantity),
        effective_from: r.effective_from,
        effective_to: r.effective_to,
        is_active: r.is_active,
        created_at: r.created_at.or(Some(0)),
        updated_at: r.updated_at.or(Some(0)),
    }))
}

// Helper to fetch raw model for advanced validations if needed
pub async fn get_recipe_set_model_by_uuid(
    db: &Pool<Postgres>,
    id: Uuid,
) -> Result<Option<RecipeSetsModel>, sqlx::Error> {
    let row = sqlx::query_as!(
        RecipeSetsModel,
        r#"SELECT uuid, name, yield_quantity, effective_from, effective_to, is_active, created_at, updated_at, deleted_at
           FROM recipe_sets WHERE uuid = $1 AND deleted_at = 0"#,
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(row)
}

// Update recipe set and return processed schema
pub async fn update_recipe_set(
    db: &Pool<Postgres>,
    id: Uuid,
    body: UpdateRecipeSetSchema,
    timestamp_ms: i64,
) -> Result<Option<ProcessedRecipeSetSchema>, sqlx::Error> {
    // Fetch current
    let current = sqlx::query_as!(
        RecipeSetsModel,
        r#"SELECT uuid, name, yield_quantity, effective_from, effective_to, is_active, created_at, updated_at, deleted_at
           FROM recipe_sets WHERE uuid = $1 AND deleted_at = 0"#,
        id
    )
    .fetch_optional(db)
    .await?;

    let Some(existing) = current else {
        return Ok(None);
    };

    let new_name = body.name.unwrap_or(existing.name);
    let current_yield = resolve_yield_quantity(existing.yield_quantity);
    let new_yield_quantity = body.yield_quantity.unwrap_or(current_yield);
    let new_effective_from = body.effective_from.or(existing.effective_from);
    let new_effective_to = body.effective_to.or(existing.effective_to);
    let new_is_active = body.is_active.unwrap_or(existing.is_active);

    let res = sqlx::query!(
        r#"UPDATE recipe_sets SET name = $1, yield_quantity = $2, effective_from = $3, effective_to = $4, is_active = $5, updated_at = $6
           WHERE uuid = $7 AND deleted_at = 0"#,
        new_name,
        new_yield_quantity,
        new_effective_from,
        new_effective_to,
        new_is_active,
        timestamp_ms,
        id
    )
    .execute(db)
    .await?;

    if res.rows_affected() == 0 {
        return Ok(None);
    }

    Ok(Some(ProcessedRecipeSetSchema {
        uuid: id,
        name: new_name,
        yield_quantity: new_yield_quantity,
        effective_from: new_effective_from,
        effective_to: new_effective_to,
        is_active: new_is_active,
        created_at: existing.created_at.or(Some(0)),
        updated_at: Some(timestamp_ms),
    }))
}

// Soft delete recipe set
pub async fn soft_delete_recipe_set(
    db: &Pool<Postgres>,
    id: Uuid,
    timestamp_ms: i64,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        r#"UPDATE recipe_sets SET deleted_at = $1 WHERE uuid = $2 AND deleted_at = 0"#,
        timestamp_ms,
        id
    )
    .execute(db)
    .await?;

    Ok(res.rows_affected() > 0)
}
