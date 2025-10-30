use rust_decimal::Decimal;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::dto::recipe_items::{
    CreateRecipeItemSchema, GetRecipeItemSchema, ProcessedRecipeItemSchema, UpdateRecipeItemSchema,
};
use crate::models::recipe_items::RecipeItemsModel;

// Create recipe item and return processed schema
pub async fn create_recipe_item(
    db: &Pool<Postgres>,
    recipe_item_uuid: Uuid,
    body: CreateRecipeItemSchema,
    timestamp_ms: i64,
) -> Result<ProcessedRecipeItemSchema, sqlx::Error> {
    let waste_percent = body.waste_percent.unwrap_or(Decimal::ZERO);

    sqlx::query!(
        r#"INSERT INTO recipe_items (uuid, recipe_sets_uuid, ingredient_stocks_uuid, quantity, waste_percent, created_at, updated_at, deleted_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        recipe_item_uuid,
        body.recipe_sets_uuid,
        body.ingredient_stocks_uuid,
        body.quantity,
        waste_percent,
        timestamp_ms,
        timestamp_ms,
        0i64
    )
    .execute(db)
    .await?;

    Ok(ProcessedRecipeItemSchema {
        uuid: recipe_item_uuid,
        recipe_sets_uuid: body.recipe_sets_uuid,
        ingredient_stocks_uuid: body.ingredient_stocks_uuid,
        quantity: body.quantity,
        waste_percent: Some(waste_percent),
        created_at: Some(timestamp_ms),
        updated_at: Some(timestamp_ms),
    })
}

// List recipe items with optional filtering by recipe_sets_uuid and/or ingredient_stocks_uuid
pub async fn list_recipe_items(
    db: &Pool<Postgres>,
    opts: GetRecipeItemSchema,
) -> Result<Vec<ProcessedRecipeItemSchema>, sqlx::Error> {
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let rows = if let (Some(recipe_sets_uuid), Some(ingredient_stocks_uuid)) =
        (opts.recipe_sets_uuid, opts.ingredient_stocks_uuid)
    {
        sqlx::query_as!(
            RecipeItemsModel,
            r#"SELECT uuid, recipe_sets_uuid, ingredient_stocks_uuid, quantity, waste_percent, created_at, updated_at, deleted_at
               FROM recipe_items WHERE deleted_at = 0 AND recipe_sets_uuid = $1 AND ingredient_stocks_uuid = $2
               ORDER BY created_at DESC LIMIT $3 OFFSET $4"#,
            recipe_sets_uuid,
            ingredient_stocks_uuid,
            limit as i64,
            offset as i64
        )
        .fetch_all(db)
        .await?
    } else if let Some(recipe_sets_uuid) = opts.recipe_sets_uuid {
        sqlx::query_as!(
            RecipeItemsModel,
            r#"SELECT uuid, recipe_sets_uuid, ingredient_stocks_uuid, quantity, waste_percent, created_at, updated_at, deleted_at
               FROM recipe_items WHERE deleted_at = 0 AND recipe_sets_uuid = $1
               ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
            recipe_sets_uuid,
            limit as i64,
            offset as i64
        )
        .fetch_all(db)
        .await?
    } else if let Some(ingredient_stocks_uuid) = opts.ingredient_stocks_uuid {
        sqlx::query_as!(
            RecipeItemsModel,
            r#"SELECT uuid, recipe_sets_uuid, ingredient_stocks_uuid, quantity, waste_percent, created_at, updated_at, deleted_at
               FROM recipe_items WHERE deleted_at = 0 AND ingredient_stocks_uuid = $1
               ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
            ingredient_stocks_uuid,
            limit as i64,
            offset as i64
        )
        .fetch_all(db)
        .await?
    } else {
        sqlx::query_as!(
            RecipeItemsModel,
            r#"SELECT uuid, recipe_sets_uuid, ingredient_stocks_uuid, quantity, waste_percent, created_at, updated_at, deleted_at
               FROM recipe_items WHERE deleted_at = 0
               ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            limit as i64,
            offset as i64
        )
        .fetch_all(db)
        .await?
    };

    Ok(rows
        .into_iter()
        .map(|ri| ProcessedRecipeItemSchema {
            uuid: ri.uuid,
            recipe_sets_uuid: ri.recipe_sets_uuid,
            ingredient_stocks_uuid: ri.ingredient_stocks_uuid,
            quantity: ri.quantity,
            waste_percent: ri.waste_percent,
            created_at: ri.created_at.or(Some(0)),
            updated_at: ri.updated_at.or(Some(0)),
        })
        .collect())
}

pub async fn get_recipe_item_by_uuid(
    db: &Pool<Postgres>,
    id: Uuid,
) -> Result<Option<ProcessedRecipeItemSchema>, sqlx::Error> {
    let row = sqlx::query_as!(
        RecipeItemsModel,
        r#"SELECT uuid, recipe_sets_uuid, ingredient_stocks_uuid, quantity, waste_percent, created_at, updated_at, deleted_at
           FROM recipe_items WHERE uuid = $1 AND deleted_at = 0"#,
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|ri| ProcessedRecipeItemSchema {
        uuid: ri.uuid,
        recipe_sets_uuid: ri.recipe_sets_uuid,
        ingredient_stocks_uuid: ri.ingredient_stocks_uuid,
        quantity: ri.quantity,
        waste_percent: ri.waste_percent,
        created_at: ri.created_at.or(Some(0)),
        updated_at: ri.updated_at.or(Some(0)),
    }))
}

// Update recipe item and return processed schema
pub async fn update_recipe_item(
    db: &Pool<Postgres>,
    id: Uuid,
    body: UpdateRecipeItemSchema,
    timestamp_ms: i64,
) -> Result<Option<ProcessedRecipeItemSchema>, sqlx::Error> {
    // Fetch current
    let current = sqlx::query_as!(
        RecipeItemsModel,
        r#"SELECT uuid, recipe_sets_uuid, ingredient_stocks_uuid, quantity, waste_percent, created_at, updated_at, deleted_at
           FROM recipe_items WHERE uuid = $1 AND deleted_at = 0"#,
        id
    )
    .fetch_optional(db)
    .await?;

    let Some(existing) = current else {
        return Ok(None);
    };

    let new_quantity = body.quantity.unwrap_or(existing.quantity);
    let new_waste_percent = body.waste_percent.or(existing.waste_percent);

    let res = sqlx::query!(
        r#"UPDATE recipe_items SET quantity = $1, waste_percent = $2, updated_at = $3 WHERE uuid = $4 AND deleted_at = 0"#,
        new_quantity,
        new_waste_percent,
        timestamp_ms,
        id
    )
    .execute(db)
    .await?;

    if res.rows_affected() == 0 {
        return Ok(None);
    }

    Ok(Some(ProcessedRecipeItemSchema {
        uuid: id,
        recipe_sets_uuid: existing.recipe_sets_uuid,
        ingredient_stocks_uuid: existing.ingredient_stocks_uuid,
        quantity: new_quantity,
        waste_percent: new_waste_percent,
        created_at: existing.created_at.or(Some(0)),
        updated_at: Some(timestamp_ms),
    }))
}

// Soft delete recipe item
pub async fn soft_delete_recipe_item(
    db: &Pool<Postgres>,
    id: Uuid,
    timestamp_ms: i64,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        r#"UPDATE recipe_items SET deleted_at = $1 WHERE uuid = $2 AND deleted_at = 0"#,
        timestamp_ms,
        id
    )
    .execute(db)
    .await?;

    Ok(res.rows_affected() > 0)
}
