use sqlx::postgres::PgPool;
use sqlx::Error;

pub async fn initial_roles(pool: &PgPool) -> Result<(), Error> {
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
    .execute(pool)
    .await?;

    Ok(())
}
