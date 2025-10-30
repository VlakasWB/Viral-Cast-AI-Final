use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Row};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Check existing roles count
    let count: i64 = sqlx::query("SELECT COUNT(*) AS cnt FROM roles")
        .fetch_one(&pool)
        .await?
        .get("cnt");

    if count == 0 {
        sqlx::query(
            r#"
            INSERT INTO roles (number, name) VALUES
                (1, 'Super Admin'),
                (2, 'Admin'),
                (3, 'Chief Operating Officer (COO)'),
                (4, 'Supervisor'),
                (5, 'Manager'),
                (6, 'Staff'),
                (7, 'Owner'),
                (8, 'Finance');
            "#,
        )
        .execute(&pool)
        .await?;
        println!("✅ roles seeded (8 rows)");
    } else {
        println!("ℹ️ roles already exist ({} rows)", count);
    }

    Ok(())
}
