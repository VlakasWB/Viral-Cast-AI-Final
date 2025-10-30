use sqlx::{postgres::PgPool, Error};

use crate::data::master::roles::initial_roles;

pub async fn master_roles(pool: &PgPool) -> Result<(), Error> {
    let count: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM roles")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        initial_roles(pool).await?;
        println!("✅ [startup] Initial master roles seeded (8 rows)");
    } else {
        println!("ℹ️ [startup] Master roles already exist ({} rows)", count);
    }

    Ok(())
}
