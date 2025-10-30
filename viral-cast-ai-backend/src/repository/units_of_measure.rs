use sqlx::{Pool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::dto::units_of_measure::{
    CreateUnitOfMeasureSchema, GetUnitOfMeasureSchema, PaginationInfo,
    ProcessedUnitOfMeasureSchema, SearchUnitOfMeasureSchema, UnitOfMeasureListResponse,
    UpdateUnitOfMeasureSchema,
};
use crate::models::units_of_measure::UnitsOfMeasureModel;

fn sanitize_sort_column(sort_by: Option<&str>) -> &'static str {
    match sort_by.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("code") => "code",
        Some("name") => "name",
        Some("updated_at") => "updated_at",
        _ => "created_at",
    }
}

fn sanitize_sort_direction(sort_order: Option<&str>) -> &'static str {
    match sort_order.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    }
}

pub async fn create_unit_of_measure(
    db: &Pool<Postgres>,
    body: CreateUnitOfMeasureSchema,
) -> sqlx::Result<ProcessedUnitOfMeasureSchema> {
    let uom = sqlx::query_as!(
        UnitsOfMeasureModel,
        "INSERT INTO units_of_measure (code, name) VALUES ($1, $2) RETURNING *",
        body.code,
        body.name
    )
    .fetch_one(db)
    .await?;

    Ok(ProcessedUnitOfMeasureSchema {
        uuid: uom.uuid,
        code: uom.code,
        name: uom.name,
        created_at: uom.created_at.or(Some(0)),
        updated_at: uom.updated_at.or(Some(0)),
    })
}

pub async fn list_units_of_measure(
    db: &Pool<Postgres>,
) -> sqlx::Result<Vec<ProcessedUnitOfMeasureSchema>> {
    let rows = sqlx::query_as!(
        GetUnitOfMeasureSchema,
        "SELECT uuid, code, name, created_at, updated_at FROM units_of_measure WHERE deleted_at = 0 ORDER BY created_at DESC"
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|uom| ProcessedUnitOfMeasureSchema {
            uuid: uom.uuid,
            code: uom.code,
            name: uom.name,
            created_at: uom.created_at.or(Some(0)),
            updated_at: uom.updated_at.or(Some(0)),
        })
        .collect())
}

pub async fn search_units_of_measure(
    db: &Pool<Postgres>,
    params: SearchUnitOfMeasureSchema,
) -> sqlx::Result<UnitOfMeasureListResponse> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;

    let search_pattern = params
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s));
    let code_pattern = params
        .code
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s));
    let name_pattern = params
        .name
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s));

    let mut data_builder = QueryBuilder::<Postgres>::new(
        "SELECT uuid, code, name, created_at, updated_at FROM units_of_measure WHERE deleted_at = 0",
    );
    let mut count_builder =
        QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM units_of_measure WHERE deleted_at = 0");

    if let Some(pattern) = search_pattern.as_ref() {
        data_builder
            .push(" AND (code ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR name ILIKE ")
            .push_bind(pattern.clone())
            .push(")");
        count_builder
            .push(" AND (code ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR name ILIKE ")
            .push_bind(pattern.clone())
            .push(")");
    }

    if let Some(pattern) = code_pattern.as_ref() {
        data_builder
            .push(" AND code ILIKE ")
            .push_bind(pattern.clone());
        count_builder
            .push(" AND code ILIKE ")
            .push_bind(pattern.clone());
    }

    if let Some(pattern) = name_pattern.as_ref() {
        data_builder
            .push(" AND name ILIKE ")
            .push_bind(pattern.clone());
        count_builder
            .push(" AND name ILIKE ")
            .push_bind(pattern.clone());
    }

    let sort_column = sanitize_sort_column(params.sort_by.as_deref());
    let sort_direction = sanitize_sort_direction(params.sort_order.as_deref());

    data_builder
        .push(" ORDER BY ")
        .push(sort_column)
        .push(" ")
        .push(sort_direction)
        .push(" LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let rows = data_builder
        .build_query_as::<GetUnitOfMeasureSchema>()
        .fetch_all(db)
        .await?;

    let total_count: i64 = count_builder.build_query_scalar().fetch_one(db).await?;

    let total_pages = ((total_count + limit - 1) / limit).max(1);

    // Membuat response dengan pagination
    let items = rows
        .into_iter()
        .map(|uom| ProcessedUnitOfMeasureSchema {
            uuid: uom.uuid,
            code: uom.code,
            name: uom.name,
            created_at: uom.created_at.or(Some(0)),
            updated_at: uom.updated_at.or(Some(0)),
        })
        .collect();

    Ok(UnitOfMeasureListResponse {
        items,
        pagination: PaginationInfo {
            current_page: page,
            total_pages,
            total_items: total_count,
            items_per_page: limit,
        },
    })
}

pub async fn get_unit_of_measure_by_uuid(
    db: &Pool<Postgres>,
    uom_uuid: Uuid,
) -> sqlx::Result<Option<ProcessedUnitOfMeasureSchema>> {
    let row = sqlx::query_as!(
        GetUnitOfMeasureSchema,
        "SELECT uuid, code, name, created_at, updated_at FROM units_of_measure WHERE uuid = $1 AND deleted_at = 0",
        uom_uuid
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|uom| ProcessedUnitOfMeasureSchema {
        uuid: uom.uuid,
        code: uom.code,
        name: uom.name,
        created_at: uom.created_at.or(Some(0)),
        updated_at: uom.updated_at.or(Some(0)),
    }))
}

pub async fn update_unit_of_measure(
    db: &Pool<Postgres>,
    uom_uuid: Uuid,
    body: UpdateUnitOfMeasureSchema,
    timestamp_ms: i64,
) -> sqlx::Result<Option<ProcessedUnitOfMeasureSchema>> {
    let uom = sqlx::query_as!(
        UnitsOfMeasureModel,
        "UPDATE units_of_measure SET code = COALESCE($1, code), name = COALESCE($2, name), updated_at = $3 WHERE uuid = $4 AND deleted_at = 0 RETURNING *",
        body.code,
        body.name,
        timestamp_ms,
        uom_uuid
    )
    .fetch_optional(db)
    .await?;

    Ok(uom.map(|uom| ProcessedUnitOfMeasureSchema {
        uuid: uom.uuid,
        code: uom.code,
        name: uom.name,
        created_at: uom.created_at.or(Some(0)),
        updated_at: uom.updated_at.or(Some(0)),
    }))
}

pub async fn soft_delete_unit_of_measure(
    db: &Pool<Postgres>,
    uom_uuid: Uuid,
    timestamp_ms: i64,
) -> sqlx::Result<u64> {
    let res = sqlx::query!(
        "UPDATE units_of_measure SET deleted_at = $1 WHERE uuid = $2 AND deleted_at = 0",
        timestamp_ms,
        uom_uuid
    )
    .execute(db)
    .await?;

    Ok(res.rows_affected())
}
