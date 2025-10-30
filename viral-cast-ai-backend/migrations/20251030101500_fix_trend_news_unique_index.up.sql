-- Step 1: Remove any leftover duplicate rows so a strict unique constraint can be applied safely
WITH ranked AS (
    SELECT
        uuid,
        ROW_NUMBER() OVER (
            PARTITION BY source_id, source_article_id
            ORDER BY fetched_at DESC, uuid
        ) AS rn
    FROM trend_news_articles
    WHERE source_article_id IS NOT NULL
)
DELETE FROM trend_news_articles
WHERE uuid IN (SELECT uuid FROM ranked WHERE rn > 1);

-- Step 2: Normalise NULL or blank identifiers to generated UUIDs to guarantee presence
UPDATE trend_news_articles
SET source_article_id = uuid_generate_v4()::text
WHERE source_article_id IS NULL OR btrim(source_article_id) = '';

-- Step 3: Trim whitespace for deterministic matching
UPDATE trend_news_articles
SET source_article_id = btrim(source_article_id);

-- Step 4: Drop legacy partial index so a full unique constraint can take over
DROP INDEX IF EXISTS trend_news_articles_source_article_idx;

-- Step 5: Enforce NOT NULL + unique constraint (auto-creates backing unique index)
ALTER TABLE trend_news_articles
    ALTER COLUMN source_article_id SET NOT NULL;

ALTER TABLE trend_news_articles
    ADD CONSTRAINT trend_news_articles_source_article_uq
    UNIQUE (source_id, source_article_id);
