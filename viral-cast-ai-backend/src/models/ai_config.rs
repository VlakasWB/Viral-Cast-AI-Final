use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AiConfig {
    pub id: Uuid,
    pub input_validation_enabled: bool,
    pub token_limit_enabled: bool,
    pub max_input_length: i32,
    pub allowed_topics: Vec<String>,
    pub daily_token_limit: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TokenUsage {
    pub id: Uuid,
    pub date: chrono::NaiveDate,
    pub tokens_used: i32,
    pub requests_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AiRequestLog {
    pub id: Uuid,
    pub prompt: String,
    pub response: String,
    pub tokens_used: i32,
    pub model: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserInputControl {
    pub id: Uuid,
    pub max_input_length: i32,
    pub rate_limit_per_minute: i32,
    pub blocked_keywords: Vec<String>,
    pub required_keywords: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserRateLimit {
    pub id: Uuid,
    pub user_ip: String,
    pub minute_window: DateTime<Utc>,
    pub request_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
