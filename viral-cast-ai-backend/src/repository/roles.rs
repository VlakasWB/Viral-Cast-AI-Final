use crate::dto::roles::GetRolesSchema;
use sqlx::{Pool, Postgres};

pub async fn list_roles(db: &Pool<Postgres>) -> Result<Vec<GetRolesSchema>, sqlx::Error> {
    let rows = sqlx::query_as!(GetRolesSchema, "SELECT uuid, name FROM roles ORDER BY name")
        .fetch_all(db)
        .await?;
    Ok(rows)
}

pub async fn get_role_by_number(
    db: &Pool<Postgres>,
    number: i32,
) -> Result<Option<GetRolesSchema>, sqlx::Error> {
    let row = sqlx::query_as!(
        GetRolesSchema,
        "SELECT uuid, name FROM roles WHERE number = $1",
        number
    )
    .fetch_optional(db)
    .await?;
    Ok(row)
}
