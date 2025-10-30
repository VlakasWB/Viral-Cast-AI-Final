ALTER TABLE trend_news_articles
    DROP CONSTRAINT IF EXISTS trend_news_articles_source_article_uq;

ALTER TABLE trend_news_articles
    ALTER COLUMN source_article_id DROP NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS trend_news_articles_source_article_idx
  ON trend_news_articles (source_id, source_article_id)
  WHERE source_article_id IS NOT NULL;
