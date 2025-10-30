use chrono::Utc;
use serde_json::Value;
use sqlx::{Pool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::trend_news::{
    InboundTrendNewsArticle, TrendNewsArticle, TrendNewsArticleWithSource, TrendNewsSource,
};

#[derive(Debug, Clone)]
pub struct TrendNewsListParams {
    pub source_codes: Option<Vec<String>>,
    pub category: Option<String>,
    pub country: Option<String>,
    pub language: Option<String>,
    pub has_image: Option<bool>,
    pub query: Option<String>,
    pub limit: i64,
    pub offset: i64,
    pub only_active_sources: bool,
    pub include_deleted: bool,
}

impl Default for TrendNewsListParams {
    fn default() -> Self {
        Self {
            source_codes: None,
            category: None,
            country: None,
            language: None,
            has_image: None,
            query: None,
            limit: 20,
            offset: 0,
            only_active_sources: true,
            include_deleted: false,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TrendNewsArticleRow {
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
    pub source_code: String,
    pub source_name: String,
    pub source_type: String,
    pub source_base_url: Option<String>,
    pub source_description: Option<String>,
    pub source_is_active: bool,
    pub source_metadata: Option<Value>,
    pub source_created_at: Option<i64>,
    pub source_updated_at: Option<i64>,
}

impl From<TrendNewsArticleRow> for TrendNewsArticleWithSource {
    fn from(row: TrendNewsArticleRow) -> Self {
        let article = TrendNewsArticle {
            uuid: row.uuid,
            source_id: row.source_id,
            source_article_id: row.source_article_id,
            title: row.title,
            description: row.description,
            content: row.content,
            summary: row.summary,
            image_url: row.image_url,
            author: row.author,
            published_at: row.published_at,
            fetched_at: row.fetched_at,
            country: row.country,
            language: row.language,
            category: row.category,
            keywords: row.keywords,
            source_url: row.source_url,
            raw_payload: row.raw_payload,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        };

        let source = TrendNewsSource {
            id: row.source_id,
            code: row.source_code,
            name: row.source_name,
            source_type: row.source_type,
            base_url: row.source_base_url,
            description: row.source_description,
            is_active: row.source_is_active,
            metadata: row.source_metadata,
            created_at: row.source_created_at,
            updated_at: row.source_updated_at,
        };

        Self { article, source }
    }
}

fn now_millis() -> i64 {
    Utc::now().timestamp_millis()
}

pub async fn ensure_source(
    db: &Pool<Postgres>,
    code: &str,
    name: &str,
    source_type: &str,
    base_url: Option<&str>,
    description: Option<&str>,
    metadata: Option<Value>,
) -> Result<TrendNewsSource, sqlx::Error> {
    sqlx::query_as::<_, TrendNewsSource>(
        r#"
        INSERT INTO trend_news_sources (code, name, source_type, base_url, description, metadata, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, TRUE, $7, $7)
        ON CONFLICT (code)
        DO UPDATE
        SET name = EXCLUDED.name,
            source_type = EXCLUDED.source_type,
            base_url = EXCLUDED.base_url,
            description = EXCLUDED.description,
            metadata = EXCLUDED.metadata,
            is_active = TRUE,
            updated_at = EXCLUDED.updated_at
        RETURNING id, code, name, source_type, base_url, description, is_active, metadata, created_at, updated_at
        "#,
    )
    .bind(code)
    .bind(name)
    .bind(source_type)
    .bind(base_url)
    .bind(description)
    .bind(metadata)
    .bind(now_millis())
    .fetch_one(db)
    .await
}

pub async fn upsert_articles(
    db: &Pool<Postgres>,
    source_id: Uuid,
    articles: &[InboundTrendNewsArticle],
) -> Result<Vec<TrendNewsArticle>, sqlx::Error> {
    if articles.is_empty() {
        return Ok(vec![]);
    }

    let mut tx = db.begin().await?;
    let mut stored = Vec::with_capacity(articles.len());

    for article in articles {
        let trimmed_id = article.source_article_id.trim();
        if trimmed_id.is_empty() {
            continue;
        }
        let normalized_source_article_id = trimmed_id.to_string();
        let normalized_source_url = article
            .source_url
            .as_ref()
            .map(|url| url.trim())
            .filter(|url| !url.is_empty())
            .map(|url| url.to_string());

        // ID: Gunakan ON CONSTRAINT agar sesuai dengan nama constraint unik terbaru,
        // mencegah mismatch ketika indeks partial lama dihapus.
        // EN: Use ON CONSTRAINT to align with the latest unique constraint name,
        // preventing mismatch after the legacy partial index was dropped.
        let record = sqlx::query_as::<_, TrendNewsArticle>(
            r#"
            INSERT INTO trend_news_articles (
                source_id,
                source_article_id,
                title,
                description,
                content,
                summary,
                image_url,
                author,
                published_at,
                fetched_at,
                country,
                language,
                category,
                keywords,
                source_url,
                raw_payload
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16
            )
            ON CONFLICT ON CONSTRAINT trend_news_articles_source_article_uq
            DO UPDATE SET
                title = EXCLUDED.title,
                description = COALESCE(EXCLUDED.description, trend_news_articles.description),
                content = COALESCE(EXCLUDED.content, trend_news_articles.content),
                summary = COALESCE(EXCLUDED.summary, trend_news_articles.summary),
                image_url = COALESCE(EXCLUDED.image_url, trend_news_articles.image_url),
                author = COALESCE(EXCLUDED.author, trend_news_articles.author),
                published_at = COALESCE(EXCLUDED.published_at, trend_news_articles.published_at),
                fetched_at = EXCLUDED.fetched_at,
                country = COALESCE(EXCLUDED.country, trend_news_articles.country),
                language = COALESCE(EXCLUDED.language, trend_news_articles.language),
                category = COALESCE(EXCLUDED.category, trend_news_articles.category),
                keywords = COALESCE(EXCLUDED.keywords, trend_news_articles.keywords),
                source_url = COALESCE(EXCLUDED.source_url, trend_news_articles.source_url),
                raw_payload = COALESCE(EXCLUDED.raw_payload, trend_news_articles.raw_payload),
                updated_at = EXTRACT(EPOCH FROM NOW()) * 1000,
                deleted_at = 0
            RETURNING
                uuid,
                source_id,
                source_article_id,
                title,
                description,
                content,
                summary,
                image_url,
                author,
                published_at,
                fetched_at,
                country,
                language,
                category,
                keywords,
                source_url,
                raw_payload,
                created_at,
                updated_at,
                deleted_at
            "#,
        )
        .bind(source_id)
        .bind(&normalized_source_article_id)
        .bind(&article.title)
        .bind(&article.description)
        .bind(&article.content)
        .bind(&article.summary)
        .bind(&article.image_url)
        .bind(&article.author)
        .bind(article.published_at)
        .bind(article.fetched_at)
        .bind(&article.country)
        .bind(&article.language)
        .bind(&article.category)
        .bind(&article.keywords)
        .bind(&normalized_source_url)
        .bind(&article.raw_payload)
        .fetch_one(&mut *tx)
        .await?;

        stored.push(record);
    }

    tx.commit().await?;
    Ok(stored)
}

pub async fn list_articles(
    db: &Pool<Postgres>,
    params: &TrendNewsListParams,
) -> Result<(Vec<TrendNewsArticleWithSource>, i64), sqlx::Error> {
    let select_sql = r#"
        SELECT
            a.uuid,
            a.source_id,
            a.source_article_id,
            a.title,
            a.description,
            a.content,
            a.summary,
            a.image_url,
            a.author,
            a.published_at,
            a.fetched_at,
            a.country,
            a.language,
            a.category,
            a.keywords,
            a.source_url,
            a.raw_payload,
            a.created_at,
            a.updated_at,
            a.deleted_at,
            s.code AS source_code,
            s.name AS source_name,
            s.source_type,
            s.base_url AS source_base_url,
            s.description AS source_description,
            s.is_active AS source_is_active,
            s.metadata AS source_metadata,
            s.created_at AS source_created_at,
            s.updated_at AS source_updated_at
        FROM trend_news_articles a
        JOIN trend_news_sources s ON s.id = a.source_id
        WHERE 1 = 1
    "#;

    let count_sql = r#"
        SELECT COUNT(*)
        FROM trend_news_articles a
        JOIN trend_news_sources s ON s.id = a.source_id
        WHERE 1 = 1
    "#;

    let mut data_builder = QueryBuilder::new(select_sql);
    apply_filters(&mut data_builder, params);
    data_builder.push(" ORDER BY COALESCE(a.published_at, a.fetched_at) DESC ");
    data_builder
        .push(" LIMIT ")
        .push_bind(params.limit.max(1))
        .push(" OFFSET ")
        .push_bind(params.offset.max(0));

    let rows: Vec<TrendNewsArticleRow> = data_builder.build_query_as().fetch_all(db).await?;

    let mut count_builder = QueryBuilder::new(count_sql);
    apply_filters(&mut count_builder, params);
    let total: i64 = count_builder.build_query_scalar().fetch_one(db).await?;

    Ok((
        rows.into_iter()
            .map(TrendNewsArticleWithSource::from)
            .collect(),
        total,
    ))
}

fn apply_filters(builder: &mut QueryBuilder<'_, Postgres>, params: &TrendNewsListParams) {
    if !params.include_deleted {
        builder.push(" AND a.deleted_at = 0");
    }

    if params.only_active_sources {
        builder.push(" AND s.is_active = TRUE");
    }

    if let Some(ref codes) = params.source_codes {
        if !codes.is_empty() {
            builder
                .push(" AND s.code = ANY(")
                .push_bind(codes.clone())
                .push(")");
        }
    }

    if let Some(ref category) = params.category {
        builder
            .push(" AND LOWER(a.category) = LOWER(")
            .push_bind(category.clone())
            .push(")");
    }

    if let Some(ref country) = params.country {
        builder
            .push(" AND LOWER(COALESCE(a.country, '')) = LOWER(")
            .push_bind(country.clone())
            .push(")");
    }

    if let Some(ref language) = params.language {
        builder
            .push(" AND LOWER(COALESCE(a.language, '')) = LOWER(")
            .push_bind(language.clone())
            .push(")");
    }

    if let Some(has_image) = params.has_image {
        if has_image {
            builder.push(" AND a.image_url IS NOT NULL AND a.image_url <> ''");
        } else {
            builder.push(" AND (a.image_url IS NULL OR a.image_url = '')");
        }
    }

    if let Some(ref query) = params.query {
        let pattern = format!("%{}%", query);
        builder
            .push(" AND (a.title ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR a.description ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR a.summary ILIKE ")
            .push_bind(pattern)
            .push(")");
    }
}
