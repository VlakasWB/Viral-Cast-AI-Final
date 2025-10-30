use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TrendNewsSource {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub source_type: String,
    pub base_url: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub metadata: Option<Value>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TrendNewsArticle {
    pub uuid: Uuid,
    pub source_id: Uuid,
    pub source_article_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub image_url: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<i64>,
    pub fetched_at: i64,
    pub country: Option<String>,
    pub language: Option<String>,
    pub category: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub source_url: Option<String>,
    pub raw_payload: Option<Value>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InboundTrendNewsArticle {
    pub source_article_id: String,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub image_url: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<i64>,
    pub fetched_at: i64,
    pub country: Option<String>,
    pub language: Option<String>,
    pub category: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub source_url: Option<String>,
    pub raw_payload: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrendNewsArticleWithSource {
    #[serde(flatten)]
    pub article: TrendNewsArticle,
    pub source: TrendNewsSource,
}
