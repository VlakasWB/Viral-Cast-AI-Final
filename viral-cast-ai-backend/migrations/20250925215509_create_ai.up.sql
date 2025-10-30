-- Add up migration script here
-- Create AI configuration table
CREATE TABLE ai_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    input_validation_enabled BOOLEAN NOT NULL DEFAULT false,
    token_limit_enabled BOOLEAN NOT NULL DEFAULT false,
    max_input_length INTEGER NOT NULL DEFAULT 1000,
    allowed_topics TEXT[] NOT NULL DEFAULT '{}',
    daily_token_limit INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create token usage tracking table
CREATE TABLE token_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL UNIQUE,
    tokens_used INTEGER NOT NULL DEFAULT 0,
    requests_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create AI request logs table
CREATE TABLE ai_request_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prompt TEXT NOT NULL,
    response TEXT NOT NULL,
    tokens_used INTEGER NOT NULL DEFAULT 0,
    model VARCHAR(100) NOT NULL,
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX idx_token_usage_date ON token_usage(date);
CREATE INDEX idx_ai_request_logs_created_at ON ai_request_logs(created_at);
CREATE INDEX idx_ai_request_logs_success ON ai_request_logs(success);

-- Insert default AI configuration
INSERT INTO ai_config (
    input_validation_enabled,
    token_limit_enabled,
    max_input_length,
    allowed_topics,
    daily_token_limit
) VALUES (
    false,
    false,
    1000,
    ARRAY['POS', 'Point of Sale', 'Trends', 'Cuaca', 'Weather', 'Penjualan', 'Sales', 'Inventory', 'Stock', 'Product', 'Produk'],
    10000
);