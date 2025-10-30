-- Add up migration script here
-- =============== TREND NEWS SOURCES ===============
CREATE TABLE IF NOT EXISTS trend_news_sources (
  id UUID DEFAULT gen_uuid_v7() PRIMARY KEY,
  code VARCHAR(64) NOT NULL UNIQUE,
  name TEXT NOT NULL,
  source_type VARCHAR(32) NOT NULL DEFAULT 'api',
  base_url TEXT,
  description TEXT,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  metadata JSONB,
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000
);

CREATE INDEX IF NOT EXISTS trend_news_sources_active_idx
  ON trend_news_sources (is_active)
  WHERE is_active = TRUE;

-- =============== TREND NEWS ARTICLES ===============
CREATE TABLE IF NOT EXISTS trend_news_articles (
  uuid UUID DEFAULT gen_uuid_v7() PRIMARY KEY,
  source_id UUID NOT NULL REFERENCES trend_news_sources(id) ON DELETE CASCADE,
  source_article_id TEXT,
  title TEXT NOT NULL,
  description TEXT,
  content TEXT,
  summary TEXT,
  image_url TEXT,
  author TEXT,
  published_at BIGINT,
  fetched_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  country VARCHAR(4),
  language VARCHAR(8),
  category TEXT,
  keywords TEXT[],
  source_url TEXT,
  raw_payload JSONB,
  created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
  deleted_at BIGINT DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS trend_news_articles_source_article_idx
  ON trend_news_articles (source_id, source_article_id)
;
  WHERE source_article_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS trend_news_articles_published_idx
  ON trend_news_articles (published_at DESC NULLS LAST);

CREATE INDEX IF NOT EXISTS trend_news_articles_created_idx
  ON trend_news_articles (created_at DESC);

CREATE INDEX IF NOT EXISTS trend_news_articles_category_idx
  ON trend_news_articles (category)
  WHERE deleted_at = 0;

-- Seed default Serper.dev source for F&B trend aggregation
INSERT INTO trend_news_sources (code, name, source_type, base_url, description, metadata)
VALUES (
  'serper-dev',
  'Serper.dev Google News',
  'api',
  'https://google.serper.dev',
  'Google News aggregation via Serper.dev API',
  jsonb_build_object('auth', 'api-key')
)
ON CONFLICT (code) DO NOTHING;
