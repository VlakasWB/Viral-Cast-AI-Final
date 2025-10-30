use std::collections::HashSet;

use crate::{
    config::config::Config,
    models::trend_news::{InboundTrendNewsArticle, TrendNewsArticleWithSource, TrendNewsSource},
    repository::trend_news::{ensure_source, list_articles, upsert_articles, TrendNewsListParams},
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct TrendNewsSyncOptions {
    pub query: Option<String>,
    pub country: Option<String>,
    pub language: Option<String>,
    pub limit: Option<u32>,
    pub refresh_only_with_images: bool,
}

impl Default for TrendNewsSyncOptions {
    fn default() -> Self {
        Self {
            query: None,
            country: None,
            language: None,
            limit: None,
            refresh_only_with_images: false,
        }
    }
}

#[derive(Debug, Error)]
pub enum TrendNewsServiceError {
    #[error("Serper API key is not configured")]
    MissingApiKey,
    #[error("Serper API request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Failed to decode Serper response: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone)]
pub struct TrendNewsSyncResult {
    pub source: TrendNewsSource,
    pub articles: Vec<TrendNewsArticleWithSource>,
}

const DEFAULT_QUERY: &str = "food and beverage trends in Indonesia";
const DEFAULT_COUNTRY: &str = "id";
const DEFAULT_LANGUAGE: &str = "id";
const DEFAULT_LIMIT: u32 = 10;
const SERPER_DEFAULT_BASE_URL: &str = "https://google.serper.dev";

pub async fn sync_serper_trends(
    db: &sqlx::Pool<sqlx::Postgres>,
    config: &Config,
    options: TrendNewsSyncOptions,
) -> Result<TrendNewsSyncResult, TrendNewsServiceError> {
    let api_key = config
        .serper_api_key
        .as_ref()
        .ok_or(TrendNewsServiceError::MissingApiKey)?;
    if api_key.trim().is_empty() {
        return Err(TrendNewsServiceError::MissingApiKey);
    }

    let query = options
        .query
        .as_deref()
        .filter(|q| !q.trim().is_empty())
        .unwrap_or(DEFAULT_QUERY);
    let country = options
        .country
        .as_deref()
        .filter(|c| !c.trim().is_empty())
        .or_else(|| config.serper_default_gl.as_deref())
        .unwrap_or(DEFAULT_COUNTRY);
    let language = options
        .language
        .as_deref()
        .filter(|l| !l.trim().is_empty())
        .or_else(|| config.serper_default_hl.as_deref())
        .unwrap_or(DEFAULT_LANGUAGE);
    let limit = options.limit.unwrap_or(DEFAULT_LIMIT).max(1).min(20);
    let base_url = config
        .serper_base_url
        .as_deref()
        .unwrap_or(SERPER_DEFAULT_BASE_URL)
        .trim_end_matches('/');

    let client = Client::new();
    // ID: Trim API key untuk mencegah kegagalan header akibat spasi tidak sengaja.
    // EN: Trim API key to prevent header failures due to accidental spaces.
    let articles = fetch_serper_trend_news(
        &client,
        api_key.trim(),
        base_url,
        query,
        country,
        language,
        limit,
    )
    .await?;

    let filtered_articles: Vec<_> = if options.refresh_only_with_images {
        articles
            .into_iter()
            .filter(|article| article.image_url.is_some())
            .collect()
    } else {
        articles
    };

    let source = ensure_source(
        db,
        "serper-dev",
        "Serper.dev Google News",
        "api",
        Some(base_url),
        Some("Google News aggregation via Serper.dev search API"),
        Some(serde_json::json!({
            "query": query,
            "country": country,
            "language": language
        })),
    )
    .await?;

    let stored_articles = upsert_articles(db, source.id, &filtered_articles).await?;

    // Compose response with joined source
    let mut articles_with_source: Vec<TrendNewsArticleWithSource> = stored_articles
        .into_iter()
        .map(|article| TrendNewsArticleWithSource {
            article,
            source: source.clone(),
        })
        .collect();

    // Sort newest first (published_at if available, otherwise fallback to fetched_at)
    articles_with_source.sort_by(|a, b| {
        let a_ts = a.article.published_at.unwrap_or(a.article.fetched_at);
        let b_ts = b.article.published_at.unwrap_or(b.article.fetched_at);
        b_ts.cmp(&a_ts)
    });

    Ok(TrendNewsSyncResult {
        source,
        articles: articles_with_source,
    })
}

pub async fn list_latest_trends(
    db: &sqlx::Pool<sqlx::Postgres>,
    params: TrendNewsListParams,
) -> Result<(Vec<TrendNewsArticleWithSource>, i64), TrendNewsServiceError> {
    let (items, total) = list_articles(db, &params).await?;
    Ok((items, total))
}

async fn fetch_serper_trend_news(
    client: &Client,
    api_key: &str,
    base_url: &str,
    query: &str,
    country: &str,
    language: &str,
    limit: u32,
) -> Result<Vec<InboundTrendNewsArticle>, TrendNewsServiceError> {
    let endpoint = format!("{}/news", base_url);

    let payload = serde_json::json!({
        "q": query,
        "gl": country,
        "hl": language,
        "num": limit,
        "autocorrect": true,
        "tbs": "qdr:d" // last 24 hours to capture recent trends
    });

    let response = client
        .post(endpoint)
        .header("X-API-KEY", api_key)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    let mut body: Value = response.json().await?;
    let fetched_at = current_timestamp_millis();
    let mut articles = Vec::new();

    if let Some(items) = body.get_mut("news").and_then(|v| v.as_array_mut()) {
        for item in items.drain(..) {
            if let Some(article) = map_serper_entry(&item, fetched_at, country, language) {
                articles.push(article);
            }
        }
    }

    if let Some(items) = body.get_mut("stories").and_then(|v| v.as_array_mut()) {
        for item in items.drain(..) {
            if let Some(article) = map_serper_entry(&item, fetched_at, country, language) {
                articles.push(article);
            }
        }
    }

    Ok(articles)
}

fn map_serper_entry(
    entry: &Value,
    fetched_at: i64,
    country: &str,
    language: &str,
) -> Option<InboundTrendNewsArticle> {
    let title = entry
        .get("title")
        .and_then(Value::as_str)?
        .trim()
        .to_owned();

    let link = entry
        .get("link")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| title.clone());

    if link.is_empty() {
        return None;
    }

    let snippet = entry
        .get("snippet")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_owned());
    let summary = entry
        .get("summary")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_owned())
        .or_else(|| snippet.clone());
    let description = entry
        .get("description")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_owned())
        .or_else(|| snippet.clone());

    let content = entry
        .get("content")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_owned());

    let image_url = entry
        .get("imageUrl")
        .or_else(|| entry.get("thumbnailUrl"))
        .and_then(Value::as_str)
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());

    let author = entry
        .get("source")
        .or_else(|| entry.get("publisher"))
        .and_then(Value::as_str)
        .map(|s| s.trim().to_owned());

    let category = entry
        .get("category")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_owned());

    let published_at = entry
        .get("date")
        .and_then(Value::as_str)
        .and_then(parse_timestamp_to_millis);

    let mut keywords_set: HashSet<String> = HashSet::from([
        "food & beverage".to_string(),
        "restaurant".to_string(),
        "cafe".to_string(),
        "trend".to_string(),
        country.to_lowercase(),
    ]);

    if let Some(ref cat) = category {
        keywords_set.insert(cat.to_lowercase());
    }
    if let Some(tags) = entry.get("keywords").and_then(Value::as_array) {
        for tag in tags.iter().filter_map(|v| v.as_str()) {
            if !tag.trim().is_empty() {
                keywords_set.insert(tag.trim().to_lowercase());
            }
        }
    }

    let keywords: Vec<String> = keywords_set.into_iter().collect();

    Some(InboundTrendNewsArticle {
        source_article_id: link.clone(),
        title,
        description,
        content,
        summary,
        image_url,
        author,
        published_at,
        fetched_at,
        country: Some(country.to_lowercase()),
        language: Some(language.to_lowercase()),
        category,
        keywords: Some(keywords),
        source_url: Some(link),
        raw_payload: Some(entry.clone()),
    })
}

fn parse_timestamp_to_millis(raw: &str) -> Option<i64> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt.timestamp_millis());
    }

    if let Ok(dt) = DateTime::parse_from_rfc2822(raw) {
        return Some(dt.timestamp_millis());
    }

    None
}

fn current_timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}
