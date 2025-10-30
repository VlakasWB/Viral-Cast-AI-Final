-- Create user_input_controls table for managing user input restrictions
CREATE TABLE IF NOT EXISTS user_input_controls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    max_input_length INTEGER NOT NULL DEFAULT 1000,
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 10,
    blocked_keywords TEXT[] DEFAULT '{}',
    required_keywords TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create user_rate_limits table for tracking user request rates
CREATE TABLE IF NOT EXISTS user_rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_ip VARCHAR(45) NOT NULL, -- Support both IPv4 and IPv6
    minute_window TIMESTAMP WITH TIME ZONE NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_ip, minute_window)
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_user_rate_limits_ip_window ON user_rate_limits(user_ip, minute_window);
CREATE INDEX IF NOT EXISTS idx_user_rate_limits_window ON user_rate_limits(minute_window);

-- Insert default user input control settings with comprehensive business keywords
INSERT INTO user_input_controls (
    max_input_length,
    rate_limit_per_minute,
    blocked_keywords,
    required_keywords
) VALUES (
    2000,
    15,
    ARRAY[
        'politik', 'political', 'agama', 'religion', 'sara', 'pornografi', 'porn',
        'kekerasan', 'violence', 'narkoba', 'drugs', 'gambling', 'judi', 'hack', 'hacking',
        'virus', 'malware', 'illegal', 'ilegal', 'penipuan', 'scam', 'fraud', 'spam',
        'exploit', 'personal data', 'data pribadi', 'password', 'credit card', 'kartu kredit'
    ],
    ARRAY[
        'pos', 'point of sale', 'kasir', 'pembayaran', 'transaksi', 'sales', 'penjualan',
        'business', 'bisnis', 'inventory', 'inventori', 'stok', 'stock', 'produk', 'product',
        'trend', 'trending', 'popular', 'populer', 'tren', 'cuaca', 'weather', 'iklim',
        'climate', 'musim', 'season', 'aplikasi penjualan', 'sales app', 'retail',
        'manajemen', 'management', 'operasional', 'operational', 'keuangan', 'finance',
        'dashboard', 'analytics', 'analitik', 'laporan', 'report', 'customer', 'pelanggan',
        'forecasting', 'ramalan', 'prediksi', 'demand', 'permintaan', 'laba', 'profit',
        'rugi', 'loss', 'margin', 'omzet', 'revenue', 'supplier', 'pemasok', 'order',
        'pesanan', 'invoice', 'faktur', 'purchase', 'pembelian', 'procurement'
    ]
) ON CONFLICT DO NOTHING;