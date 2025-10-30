use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::ai_config::{AiConfig, AiRequestLog, TokenUsage, UserInputControl};

// Get latest AI config; create default if none exists
pub async fn get_or_create_default_config(db: &Pool<Postgres>) -> Result<AiConfig, sqlx::Error> {
    let config = sqlx::query_as::<_, AiConfig>(
        "SELECT id, input_validation_enabled, token_limit_enabled, max_input_length, allowed_topics, daily_token_limit, created_at, updated_at FROM ai_config ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(db)
    .await?;

    if let Some(cfg) = config {
        return Ok(cfg);
    }

    let default_config = AiConfig {
        id: Uuid::new_v4(),
        input_validation_enabled: false,
        token_limit_enabled: false,
        max_input_length: 1000,
        allowed_topics: vec![
            "POS".to_string(),
            "Point of Sale".to_string(),
            "Trends".to_string(),
            "Cuaca".to_string(),
            "Weather".to_string(),
            "Penjualan".to_string(),
            "Sales".to_string(),
            "Inventory".to_string(),
            "Stock".to_string(),
            "Product".to_string(),
            "Produk".to_string(),
        ],
        daily_token_limit: Some(10000),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    sqlx::query(
        "INSERT INTO ai_config (id, input_validation_enabled, token_limit_enabled, max_input_length, allowed_topics, daily_token_limit, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(default_config.id)
    .bind(default_config.input_validation_enabled)
    .bind(default_config.token_limit_enabled)
    .bind(default_config.max_input_length)
    .bind(&default_config.allowed_topics)
    .bind(default_config.daily_token_limit)
    .bind(default_config.created_at)
    .bind(default_config.updated_at)
    .execute(db)
    .await?;

    Ok(default_config)
}

pub async fn update_ai_config(db: &Pool<Postgres>, config: &AiConfig) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE ai_config SET input_validation_enabled = $1, token_limit_enabled = $2, max_input_length = $3, updated_at = $4 WHERE id = $5"
    )
    .bind(config.input_validation_enabled)
    .bind(config.token_limit_enabled)
    .bind(config.max_input_length)
    .bind(Utc::now())
    .bind(config.id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn get_token_usage_by_date(
    db: &Pool<Postgres>,
    date: NaiveDate,
) -> Result<Option<TokenUsage>, sqlx::Error> {
    sqlx::query_as::<_, TokenUsage>(
        "SELECT id, date, tokens_used, requests_count, created_at, updated_at FROM token_usage WHERE date = $1"
    )
    .bind(date)
    .fetch_optional(db)
    .await
}

pub async fn update_or_insert_token_usage(
    db: &Pool<Postgres>,
    date: NaiveDate,
    tokens_used: i32,
) -> Result<(), sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE token_usage SET tokens_used = tokens_used + $1, requests_count = requests_count + 1, updated_at = $2 WHERE date = $3"
    )
    .bind(tokens_used)
    .bind(Utc::now())
    .bind(date)
    .execute(db)
    .await?;

    if updated.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO token_usage (id, date, tokens_used, requests_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(Uuid::new_v4())
        .bind(date)
        .bind(tokens_used)
        .bind(1)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(db)
        .await?;
    }

    Ok(())
}

pub async fn insert_ai_request_log(
    db: &Pool<Postgres>,
    log: &AiRequestLog,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO ai_request_logs (id, prompt, response, tokens_used, model, success, error_message, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(log.id)
    .bind(&log.prompt)
    .bind(&log.response)
    .bind(log.tokens_used)
    .bind(&log.model)
    .bind(log.success)
    .bind(&log.error_message)
    .bind(log.created_at)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn get_token_usage_history(
    db: &Pool<Postgres>,
    limit: i64,
) -> Result<Vec<TokenUsage>, sqlx::Error> {
    sqlx::query_as::<_, TokenUsage>(
        "SELECT id, date, tokens_used, requests_count, created_at, updated_at FROM token_usage ORDER BY date DESC LIMIT $1"
    )
    .bind(limit)
    .fetch_all(db)
    .await
}

pub async fn get_latest_user_input_controls(
    db: &Pool<Postgres>,
) -> Result<Option<UserInputControl>, sqlx::Error> {
    sqlx::query_as::<_, UserInputControl>(
        "SELECT id, max_input_length, rate_limit_per_minute, blocked_keywords, required_keywords, created_at, updated_at FROM user_input_controls ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(db)
    .await
}

pub async fn insert_user_input_controls(
    db: &Pool<Postgres>,
    max_input_length: i32,
    rate_limit_per_minute: i32,
    blocked_keywords: Vec<String>,
    required_keywords: Vec<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_input_controls (max_input_length, rate_limit_per_minute, blocked_keywords, required_keywords, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(max_input_length)
    .bind(rate_limit_per_minute)
    .bind(&blocked_keywords)
    .bind(&required_keywords)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(db)
    .await?;
    Ok(())
}

pub async fn get_user_rate_limit_count(
    db: &Pool<Postgres>,
    user_ip: &str,
    minute_window: DateTime<Utc>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(request_count), 0) FROM user_rate_limits WHERE user_ip = $1 AND minute_window = $2"
    )
    .bind(user_ip)
    .bind(minute_window)
    .fetch_one(db)
    .await
}

pub async fn increment_user_rate_limit(
    db: &Pool<Postgres>,
    user_ip: &str,
    minute_window: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE user_rate_limits SET request_count = request_count + 1, updated_at = $1 WHERE user_ip = $2 AND minute_window = $3"
    )
    .bind(Utc::now())
    .bind(user_ip)
    .bind(minute_window)
    .execute(db)
    .await?;

    if updated.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO user_rate_limits (id, user_ip, minute_window, request_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(Uuid::new_v4())
        .bind(user_ip)
        .bind(minute_window)
        .bind(1)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(db)
        .await?;
    }

    Ok(())
}
