use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::models::trend_news::{TrendNewsArticleWithSource, TrendNewsSource};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TrendNewsQuery {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub has_image: Option<bool>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub refresh: Option<bool>,
    #[serde(default)]
    pub refresh_with_images: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TrendNewsSyncRequest {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub only_with_images: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrendNewsSourceResponse {
    pub code: String,
    pub name: String,
    pub source_type: String,
    pub base_url: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrendNewsArticleResponse {
    pub uuid: Uuid,
    pub source_article_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub image_url: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<i64>,
    pub fetched_at: i64,
    pub country: Option<String>,
    pub language: Option<String>,
    pub category: Option<String>,
    pub keywords: Vec<String>,
    pub source_url: Option<String>,
    pub source: TrendNewsSourceResponse,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrendNewsListResponse {
    pub items: Vec<TrendNewsArticleResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrendNewsSyncResponse {
    pub source: TrendNewsSourceResponse,
    pub items: Vec<TrendNewsArticleResponse>,
    pub fetched_at: i64,
    pub count: usize,
}

impl From<&TrendNewsSource> for TrendNewsSourceResponse {
    fn from(source: &TrendNewsSource) -> Self {
        Self {
            code: source.code.clone(),
            name: source.name.clone(),
            source_type: source.source_type.clone(),
            base_url: source.base_url.clone(),
            description: source.description.clone(),
            metadata: source.metadata.clone(),
        }
    }
}

impl From<&TrendNewsArticleWithSource> for TrendNewsSourceResponse {
    fn from(value: &TrendNewsArticleWithSource) -> Self {
        (&value.source).into()
    }
}

impl From<TrendNewsArticleWithSource> for TrendNewsArticleResponse {
    fn from(value: TrendNewsArticleWithSource) -> Self {
        let source = TrendNewsSourceResponse::from(&value);
        let article = value.article;
        Self {
            uuid: article.uuid,
            source_article_id: article.source_article_id,
            title: article.title,
            description: article.description,
            summary: article.summary,
            content: article.content,
            image_url: article.image_url,
            author: article.author,
            published_at: article.published_at,
            fetched_at: article.fetched_at,
            country: article.country,
            language: article.language,
            category: article.category,
            keywords: article.keywords.unwrap_or_default(),
            source_url: article.source_url,
            source,
        }
    }
}
