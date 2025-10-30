use sqlx::{Pool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::dto::categories::{GetCategorySchema, ListCategoryQuery};
use crate::models::categories::CategoriesModel;

// Create a new category
pub async fn create_category(
    db: &Pool<Postgres>,
    name: String,
) -> Result<CategoriesModel, sqlx::Error> {
    sqlx::query_as!(
        CategoriesModel,
        "INSERT INTO categories (name) VALUES ($1) RETURNING *",
        name,
    )
    .fetch_one(db)
    .await
}

fn apply_filters(builder: &mut QueryBuilder<'_, Postgres>, params: &ListCategoryQuery) {
    if let Some(search) = params
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        builder
            .push(" AND name ILIKE ")
            .push_bind(format!("%{}%", search));
    }

    if let Some(created_from) = params.created_from {
        builder.push(" AND created_at >= ").push_bind(created_from);
    }

    if let Some(created_to) = params.created_to {
        builder.push(" AND created_at <= ").push_bind(created_to);
    }
}

fn sanitize_sort_column(sort_by: &Option<String>) -> &'static str {
    match sort_by.as_ref().map(|s| s.trim().to_lowercase()).as_deref() {
        Some("name") => "name",
        Some("updated_at") => "updated_at",
        _ => "created_at",
    }
}

fn sanitize_sort_direction(sort_order: &Option<String>) -> &'static str {
    match sort_order
        .as_ref()
        .map(|s| s.trim().to_lowercase())
        .as_deref()
    {
        Some("asc") => "ASC",
        _ => "DESC",
    }
}

// List all categories (not deleted)
pub async fn list_categories(
    db: &Pool<Postgres>,
    params: &ListCategoryQuery,
    page: i64,
    limit: i64,
) -> Result<(Vec<GetCategorySchema>, i64), sqlx::Error> {
    let offset = (page - 1) * limit;

    let mut data_builder = QueryBuilder::new(
        "SELECT uuid, name, created_at, updated_at FROM categories WHERE deleted_at = 0",
    );
    apply_filters(&mut data_builder, params);

    let sort_column = sanitize_sort_column(&params.sort_by);
    let sort_direction = sanitize_sort_direction(&params.sort_order);

    data_builder
        .push(" ORDER BY ")
        .push(sort_column)
        .push(" ")
        .push(sort_direction)
        .push(" LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let mut count_builder =
        QueryBuilder::new("SELECT COUNT(*) FROM categories WHERE deleted_at = 0");
    apply_filters(&mut count_builder, params);

    let categories = data_builder
        .build_query_as::<GetCategorySchema>()
        .fetch_all(db)
        .await?;

    let total: i64 = count_builder.build_query_scalar().fetch_one(db).await?;

    Ok((categories, total))
}

// Get category by UUID (not deleted)
pub async fn find_category_by_uuid(
    db: &Pool<Postgres>,
    category_uuid: Uuid,
) -> Result<Option<GetCategorySchema>, sqlx::Error> {
    sqlx::query_as!(
        GetCategorySchema,
        "SELECT uuid, name, created_at, updated_at FROM categories WHERE uuid = $1 AND deleted_at = 0",
        category_uuid
    )
    .fetch_optional(db)
    .await
}

// Update category name with timestamp
pub async fn update_category(
    db: &Pool<Postgres>,
    category_uuid: Uuid,
    name: Option<String>,
    updated_at: i64,
) -> Result<Option<CategoriesModel>, sqlx::Error> {
    sqlx::query_as!(
        CategoriesModel,
        "UPDATE categories SET name = COALESCE($1, name), updated_at = $2 WHERE uuid = $3 AND deleted_at = 0 RETURNING *",
        name,
        updated_at,
        category_uuid
    )
    .fetch_optional(db)
    .await
}

// Soft delete category by setting deleted_at
pub async fn soft_delete_category(
    db: &Pool<Postgres>,
    category_uuid: Uuid,
    deleted_at: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE categories SET deleted_at = $1 WHERE uuid = $2 AND deleted_at = 0",
        deleted_at,
        category_uuid
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
