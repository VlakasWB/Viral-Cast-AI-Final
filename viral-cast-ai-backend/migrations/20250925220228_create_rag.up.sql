-- Enable vector extension for PostgreSQL (if using pgvector)
-- CREATE EXTENSION IF NOT EXISTS vector;

-- Create document categories table
CREATE TABLE IF NOT EXISTS document_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    color VARCHAR(7) NOT NULL DEFAULT '#007bff', -- Hex color
    icon VARCHAR(50),
    allowed_file_types TEXT[] DEFAULT '{}',
    max_file_size_mb INTEGER DEFAULT 50,
    auto_tags TEXT[] DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create document tags table
CREATE TABLE IF NOT EXISTS document_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    color VARCHAR(7),
    usage_count BIGINT DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create RAG configuration table
CREATE TABLE IF NOT EXISTS rag_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chunk_size INTEGER NOT NULL DEFAULT 1000,
    chunk_overlap INTEGER NOT NULL DEFAULT 200,
    embedding_model VARCHAR(100) NOT NULL DEFAULT 'text-embedding-ada-002',
    embedding_dimensions INTEGER NOT NULL DEFAULT 1536,
    similarity_threshold REAL NOT NULL DEFAULT 0.7,
    max_results INTEGER NOT NULL DEFAULT 10,
    enable_reranking BOOLEAN DEFAULT false,
    reranking_model VARCHAR(100),
    supported_file_types TEXT[] DEFAULT ARRAY['pdf', 'txt', 'docx', 'md', 'csv'],
    max_file_size_mb INTEGER DEFAULT 50,
    max_documents_per_user INTEGER DEFAULT 100,
    retention_days INTEGER DEFAULT 365,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create documents table
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL,
    tags TEXT[] DEFAULT '{}',
    file_path VARCHAR(500) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    file_type VARCHAR(10) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'processing', -- processing, ready, error, deleted
    error_message TEXT,
    chunk_count INTEGER DEFAULT 0,
    processing_progress INTEGER DEFAULT 0, -- 0-100
    current_processing_step VARCHAR(100),
    uploaded_by UUID, -- Reference to users table
    access_count BIGINT DEFAULT 0,
    last_accessed TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    FOREIGN KEY (category) REFERENCES document_categories(name) ON UPDATE CASCADE
);

-- Create document chunks table for vector storage
CREATE TABLE IF NOT EXISTS document_chunks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    content_hash VARCHAR(64) NOT NULL, -- SHA-256 hash for deduplication
    embedding REAL[] NOT NULL, -- Vector embedding (use vector type if pgvector is available)
    page_number INTEGER,
    start_char INTEGER,
    end_char INTEGER,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    UNIQUE(document_id, chunk_index)
);

-- Create document processing jobs table
CREATE TABLE IF NOT EXISTS document_processing_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL,
    job_type VARCHAR(50) NOT NULL, -- extract_text, generate_embeddings, reprocess
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, processing, completed, failed
    priority INTEGER DEFAULT 5, -- 1-10, higher is more priority
    progress_percentage INTEGER DEFAULT 0,
    current_step VARCHAR(100) DEFAULT 'Initializing',
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

-- Create document access log table
CREATE TABLE IF NOT EXISTS document_access_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL,
    user_id UUID,
    user_ip INET,
    query TEXT NOT NULL,
    similarity_score REAL NOT NULL,
    response_time_ms BIGINT NOT NULL,
    chunks_retrieved INTEGER NOT NULL,
    accessed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

-- Create RAG query history table
CREATE TABLE IF NOT EXISTS rag_query_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    user_ip INET,
    query TEXT NOT NULL,
    query_hash VARCHAR(64) NOT NULL, -- SHA-256 hash of query
    category_filter VARCHAR(100),
    document_ids_filter UUID[],
    max_results INTEGER NOT NULL,
    similarity_threshold REAL NOT NULL,
    results_count INTEGER NOT NULL,
    top_similarity_score REAL,
    response_time_ms BIGINT NOT NULL,
    user_feedback VARCHAR(20), -- good, bad, neutral
    feedback_comment TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create document similarity cache table
CREATE TABLE IF NOT EXISTS document_similarity_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query_hash VARCHAR(64) NOT NULL UNIQUE,
    document_ids UUID[] NOT NULL,
    similarity_scores REAL[] NOT NULL,
    chunk_ids UUID[] NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_documents_category ON documents(category);
CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(status);
CREATE INDEX IF NOT EXISTS idx_documents_uploaded_by ON documents(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at);
CREATE INDEX IF NOT EXISTS idx_documents_tags ON documents USING GIN(tags);

CREATE INDEX IF NOT EXISTS idx_document_chunks_document_id ON document_chunks(document_id);
CREATE INDEX IF NOT EXISTS idx_document_chunks_content_hash ON document_chunks(content_hash);
-- CREATE INDEX IF NOT EXISTS idx_document_chunks_embedding ON document_chunks USING ivfflat (embedding vector_cosine_ops); -- If using pgvector

CREATE INDEX IF NOT EXISTS idx_processing_jobs_document_id ON document_processing_jobs(document_id);
CREATE INDEX IF NOT EXISTS idx_processing_jobs_status ON document_processing_jobs(status);
CREATE INDEX IF NOT EXISTS idx_processing_jobs_priority ON document_processing_jobs(priority DESC);
CREATE INDEX IF NOT EXISTS idx_processing_jobs_created_at ON document_processing_jobs(created_at);

CREATE INDEX IF NOT EXISTS idx_access_logs_document_id ON document_access_logs(document_id);
CREATE INDEX IF NOT EXISTS idx_access_logs_user_id ON document_access_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_access_logs_accessed_at ON document_access_logs(accessed_at);

CREATE INDEX IF NOT EXISTS idx_query_history_user_id ON rag_query_history(user_id);
CREATE INDEX IF NOT EXISTS idx_query_history_query_hash ON rag_query_history(query_hash);
CREATE INDEX IF NOT EXISTS idx_query_history_created_at ON rag_query_history(created_at);

CREATE INDEX IF NOT EXISTS idx_similarity_cache_query_hash ON document_similarity_cache(query_hash);
CREATE INDEX IF NOT EXISTS idx_similarity_cache_expires_at ON document_similarity_cache(expires_at);

-- Insert default document categories
INSERT INTO document_categories (name, description, color, icon, allowed_file_types, auto_tags) VALUES
('POS', 'Point of Sale related documents', '#28a745', 'cash-register', ARRAY['pdf', 'txt', 'docx', 'csv'], ARRAY['pos', 'kasir', 'penjualan']),
('Trends', 'Market trends and analysis documents', '#17a2b8', 'trending-up', ARRAY['pdf', 'txt', 'docx', 'csv', 'xlsx'], ARRAY['trend', 'analisis', 'pasar']),
('Weather', 'Weather and climate related documents', '#ffc107', 'cloud-sun', ARRAY['pdf', 'txt', 'docx', 'csv'], ARRAY['cuaca', 'iklim', 'musim']),
('Sales', 'Sales and revenue documents', '#007bff', 'chart-line', ARRAY['pdf', 'txt', 'docx', 'csv', 'xlsx'], ARRAY['penjualan', 'revenue', 'omzet']),
('Inventory', 'Inventory and stock management', '#6f42c1', 'boxes', ARRAY['pdf', 'txt', 'docx', 'csv', 'xlsx'], ARRAY['inventory', 'stok', 'gudang']),
('Finance', 'Financial reports and analysis', '#fd7e14', 'calculator', ARRAY['pdf', 'txt', 'docx', 'csv', 'xlsx'], ARRAY['keuangan', 'laporan', 'finance']),
('General', 'General business documents', '#6c757d', 'file-text', ARRAY['pdf', 'txt', 'docx', 'md'], ARRAY['bisnis', 'umum'])
ON CONFLICT (name) DO NOTHING;

-- Insert default RAG configuration
INSERT INTO rag_configurations (
    chunk_size,
    chunk_overlap,
    embedding_model,
    embedding_dimensions,
    similarity_threshold,
    max_results,
    enable_reranking,
    supported_file_types,
    max_file_size_mb,
    max_documents_per_user,
    retention_days
) VALUES (
    1000,
    200,
    'text-embedding-ada-002',
    1536,
    0.7,
    10,
    false,
    ARRAY['pdf', 'txt', 'docx', 'md', 'csv'],
    50,
    100,
    365
) ON CONFLICT DO NOTHING;

-- Insert common document tags
INSERT INTO document_tags (name, description, color) VALUES
('manual', 'User manuals and guides', '#007bff'),
('report', 'Reports and analytics', '#28a745'),
('policy', 'Policies and procedures', '#dc3545'),
('training', 'Training materials', '#ffc107'),
('reference', 'Reference documents', '#6c757d'),
('template', 'Document templates', '#17a2b8'),
('specification', 'Technical specifications', '#6f42c1'),
('contract', 'Contracts and agreements', '#fd7e14')
ON CONFLICT (name) DO NOTHING;