use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct User {
    pub uuid: uuid::Uuid,
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserSchema {
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    pub username: String,
    pub password: String,
}
