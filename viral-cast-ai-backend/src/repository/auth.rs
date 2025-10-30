use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::user::User;

pub async fn username_exists(db: &Pool<Postgres>, username: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
        .bind(username)
        .fetch_one(db)
        .await
}

pub async fn email_exists(db: &Pool<Postgres>, email: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(email)
        .fetch_one(db)
        .await
}

pub async fn insert_user(
    db: &Pool<Postgres>,
    username: &str,
    email: Option<&str>,
    hashed_password: &str,
) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"
            INSERT INTO users (username, email, password, access_token, refresh_token)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
            uuid,
            username,
            email,
            password,
            access_token,
            refresh_token,
            created_at,
            updated_at,
            deleted_at
        "#,
        username,
        email,
        hashed_password,
        None::<String>,
        None::<String>
    )
    .fetch_one(db)
    .await
}

pub async fn create_profile_for_user(db: &Pool<Postgres>, user_uuid: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
            INSERT INTO profiles (user_uuid)
            VALUES ($1)
        "#,
        user_uuid
    )
    .execute(db)
    .await
    .map(|_| ())
}

pub async fn find_user_by_username(
    db: &Pool<Postgres>,
    username: &str,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        "SELECT uuid, username, email, password, access_token, refresh_token, created_at, updated_at, deleted_at FROM users WHERE username = $1",
        username
    )
    .fetch_optional(db)
    .await
}

pub async fn find_user_by_uuid(db: &Pool<Postgres>, user_uuid: Uuid) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        "SELECT uuid, username, email, password, access_token, refresh_token, created_at, updated_at, deleted_at FROM users WHERE uuid = $1",
        user_uuid
    )
    .fetch_optional(db)
    .await
}
