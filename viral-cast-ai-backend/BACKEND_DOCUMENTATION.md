# Viral Cast AI Backend Documentation

## 📋 Overview

Backend Viral Cast AI dibangun menggunakan **Rust** dengan framework **Axum** untuk performa tinggi dan keamanan. Sistem ini menyediakan API RESTful untuk manajemen bisnis, prediksi AI, dan analisis data.

## 🏗️ Arsitektur Backend

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   API Layer     │    │  Business Logic │    │   Data Layer    │
│   (Handlers)    │◄──►│   (Services)    │◄──►│  (Repository)   │
│                 │    │                 │    │                 │
│ • Auth          │    │ • AI Services   │    │ • PostgreSQL    │
│ • Stores        │    │ • RAG System    │    │ • Redis Cache   │
│ • Products      │    │ • Predictions   │    │ • Milvus Vector │
│ • Analytics     │    │ • Weather API   │    │ • File Storage  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🚀 Setup dan Instalasi

### Prerequisites

```bash
# Install Rust (jika belum ada)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install SQLx CLI untuk database migrations
cargo install sqlx-cli --no-default-features --features postgres

# Install Podman untuk container management
# Windows: Download dari https://podman.io/
# Linux: sudo apt install podman
```

### 1. Environment Setup

```bash
cd viral-cast-ai-backend

# Copy dan edit environment file
cp .env.example .env-ai

# Edit .env-ai dengan konfigurasi yang sesuai
```

**Konfigurasi .env-ai:**
```env
# Application Configuration
APP_ENV=development
APP_PORT=12000
CLIENT_ORIGIN=http://localhost:5174
ALLOW_MOCK_DEPENDENCIES=true

# Database Configuration
POSTGRES_SERVICE=vcai-postgres
POSTGRES_HOST=127.0.0.1
POSTGRES_PORT=5432
POSTGRES_USER=vcai_user
POSTGRES_PASSWORD=vcai_password
POSTGRES_DB=viral_cast_ai

# Redis Configuration
REDIS_SERVICE=vcai-redis
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_DB=0

# Vector Database (Milvus)
MILVUS_URI=http://127.0.0.1:19530
MILVUS_TOKEN=
MILVUS_COLLECTION=rag_chunks

# AI Configuration
GROQ_API_KEY=your_groq_api_key_here
GROQ_API_URL=https://api.groq.com/openai/v1/chat/completions
GROQ_MODEL=llama-3.1-8b-instant

# News API
SERPER_API_KEY=your_serper_api_key_here

# JWT Configuration (gunakan yang ada di .env.example)
ACCESS_TOKEN_PRIVATE_KEY=...
ACCESS_TOKEN_PUBLIC_KEY=...
# ... dst
```

### 2. Start Services dengan Podman

```bash
# Start semua services (PostgreSQL, Redis, Milvus, dll)
podman compose up -d

# Cek status services
podman compose ps

# Lihat logs jika ada masalah
podman compose logs -f
```

### 3. Database Setup

```bash
# Set environment variable untuk SQLx
export DATABASE_URL="postgresql://vcai_user:vcai_password@127.0.0.1:5432/viral_cast_ai"

# Jalankan database migrations
sqlx migrate run

# Verify migrations
sqlx migrate info
```

### 4. Build dan Run

```bash
# Development mode dengan auto-reload
cargo watch -x run

# Atau build dan run manual
cargo build --release
cargo run --release

# Run dengan environment file spesifik
cargo run --bin viral_cast_ai_backend
```

## 📡 API Endpoints

### Authentication & Users

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/v1/healthchecker` | Health check endpoint | ❌ |
| `POST` | `/api/v1/auth/register` | Register new user | ❌ |
| `POST` | `/api/v1/auth/login` | User login | ❌ |
| `GET` | `/api/v1/auth/refresh` | Refresh access token | ❌ |
| `POST` | `/api/v1/auth/logout` | User logout | ✅ |
| `GET` | `/api/v1/users/me` | Get current user info | ✅ |

### Profiles Management

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/v1/profiles` | Get my profile | ✅ |
| `POST` | `/api/v1/profiles` | Create profile | ✅ |
| `PUT` | `/api/v1/profiles` | Update profile (full) | ✅ |
| `PATCH` | `/api/v1/profiles` | Update profile (partial) | ✅ |
| `GET` | `/api/v1/profiles/:id` | Get user profile by ID | ✅ |

### Stores Management

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/v1/stores` | Get my store | ✅ |
| `POST` | `/api/v1/stores` | Create new store | ✅ |
| `GET` | `/api/v1/stores/:id` | Get store by ID | ✅ |
| `PUT` | `/api/v1/stores/:id` | Update store (full) | ✅ |
| `PATCH` | `/api/v1/stores/:id` | Update store (partial) | ✅ |

### Store Predictions

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/v1/stores/predictions` | Get product predictions | ✅ |
| `POST` | `/api/v1/stores/predictions` | Generate product predictions | ✅ |
| `GET` | `/api/v1/stores/ingredient-predictions` | Get ingredient predictions | ✅ |
| `POST` | `/api/v1/stores/ingredient-predictions` | Generate ingredient predictions | ✅ |

### Categories Management

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/v1/categories` | Create category | ✅ |
| `GET` | `/api/v1/categories` | Get all categories | ✅ |
| `GET` | `/api/v1/categories/:id` | Get category by ID | ✅ |
| `PATCH` | `/api/v1/categories/:id` | Update category | ✅ |
| `DELETE` | `/api/v1/categories/:id` | Delete category | ✅ |

### Ingredient Management

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/v1/ingredient-catalog` | Create ingredient | ✅ |
| `GET` | `/api/v1/ingredient-catalog` | Get all ingredients | ✅ |
| `GET` | `/api/v1/ingredient-catalog/:id` | Get ingredient by ID | ✅ |
| `PUT` | `/api/v1/ingredient-catalog/:id` | Update ingredient | ✅ |
| `DELETE` | `/api/v1/ingredient-catalog/:id` | Delete ingredient | ✅ |

### Ingredient Market Prices

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/v1/ingredient-market-prices` | Create market price | ✅ |
| `GET` | `/api/v1/ingredient-market-prices` | Get all market prices | ✅ |
| `GET` | `/api/v1/ingredient-market-prices/:id` | Get market price by ID | ✅ |
| `PUT` | `/api/v1/ingredient-market-prices/:id` | Update market price | ✅ |
| `DELETE` | `/api/v1/ingredient-market-prices/:id` | Delete market price | ✅ |

### Ingredient Stock Management

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/v1/ingredient-stock-moves` | Create stock movement | ✅ |
| `GET` | `/api/v1/ingredient-stock-moves` | Get stock movements | ✅ |
| `GET` | `/api/v1/ingredient-stock-moves/:id` | Get stock movement by ID | ✅ |
| `PATCH` | `/api/v1/ingredient-stock-moves/:id` | Update stock movement | ✅ |
| `DELETE` | `/api/v1/ingredient-stock-moves/:id` | Delete stock movement | ✅ |

### Orders Management

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/v1/orders` | Create order | ✅ |
| `GET` | `/api/v1/orders` | Get all orders | ✅ |
| `GET` | `/api/v1/orders/:id` | Get order by ID | ✅ |
| `PUT` | `/api/v1/orders/:id` | Update order | ✅ |
| `DELETE` | `/api/v1/orders/:id` | Delete order | ✅ |
| `PATCH` | `/api/v1/orders/:id/status` | Update order status | ✅ |
| `GET` | `/api/v1/orders/stats` | Get order statistics | ✅ |

### Payments Management

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/v1/payments` | Create payment | ✅ |
| `GET` | `/api/v1/payments` | Get all payments | ✅ |
| `GET` | `/api/v1/payments/:id` | Get payment by ID | ✅ |
| `PUT` | `/api/v1/payments/:id` | Update payment | ✅ |
| `DELETE` | `/api/v1/payments/:id` | Delete payment | ✅ |
| `GET` | `/api/v1/payments/stats` | Get payment statistics | ✅ |
| `GET` | `/api/v1/payments/order/:order_uuid` | Get payments by order | ✅ |

#### QRIS Payment Integration (Xendit)

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/v1/payments/qris/sandbox` | Create QRIS payment (sandbox) | ✅ |
| `POST` | `/api/v1/payments/qris/live` | Create QRIS payment (live) | ✅ |
| `GET` | `/api/v1/payments/qris/sandbox/:external_ref/status` | Get QRIS status (sandbox) | ✅ |
| `GET` | `/api/v1/payments/qris/live/:external_ref/status` | Get QRIS status (live) | ✅ |
| `POST` | `/api/v1/payments/xendit/webhook` | Xendit webhook handler | ❌ |

### AI Services

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/ai/chat` | Chat with AI (limited) | ✅ |
| `POST` | `/api/ai/chat/unlimited` | Chat with AI (unlimited) | ✅ |
| `GET` | `/api/ai/config` | Get AI configuration | ✅ |
| `PUT` | `/api/ai/config` | Update AI configuration | ✅ |
| `GET` | `/api/ai/token-usage` | Get token usage | ✅ |
| `GET` | `/api/ai/token-usage/detailed` | Get detailed token usage | ✅ |
| `GET` | `/api/ai/token-usage/history` | Get token usage history | ✅ |
| `GET` | `/api/ai/token-usage/alerts` | Get token monitoring alerts | ✅ |
| `GET` | `/api/ai/input-controls` | Get user input controls | ✅ |
| `PUT` | `/api/ai/input-controls` | Update user input controls | ✅ |

### RAG (Retrieval-Augmented Generation)

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/documents/upload-document` | Ingest text document | ✅ |
| `POST` | `/documents/upload` | Upload document file | ✅ |
| `GET` | `/documents` | List all documents | ✅ |
| `GET` | `/documents/:id/status` | Get document status | ✅ |
| `DELETE` | `/documents/:id` | Delete document | ✅ |
| `POST` | `/query` | Query RAG system | ✅ |
| `POST` | `/answer` | Get answer with RAG + LLM | ✅ |
| `POST` | `/answer/simple` | Simple RAG answer | ✅ |
| `GET` | `/config` | Get RAG configuration | ✅ |
| `PUT` | `/config` | Update RAG configuration | ✅ |

### Image Management

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/v1/images/upload/product` | Upload product image | ✅ |
| `POST` | `/api/v1/images/upload/user` | Upload user image | ✅ |
| `POST` | `/api/v1/images/upload/user/profile-photo` | Upload profile photo | ✅ |
| `POST` | `/api/v1/images/upload/user/background` | Upload background image | ✅ |
| `POST` | `/api/v1/images/upload/store` | Upload store brand image | ✅ |
| `POST` | `/api/v1/images/upload/product/v2` | Upload product image v2 | ✅ |
| `DELETE` | `/api/v1/images/delete/product` | Delete product image | ✅ |

### Roles Management

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/v1/roles` | Get all roles | ✅ |
| `GET` | `/api/v1/roles/:id` | Get role by ID | ✅ |

## 🔧 Development Tools

### Database Migrations

```bash
# Buat migration baru
sqlx migrate add create_new_table

# Jalankan migrations
sqlx migrate run

# Rollback migration terakhir
sqlx migrate revert

# Cek status migrations
sqlx migrate info
```

### Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --features integration-tests

# Run specific test
cargo test test_name

# Run tests dengan output detail
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Linting
cargo clippy

# Check tanpa build
cargo check

# Security audit
cargo audit
```

### Performance Monitoring

```bash
# Build dengan optimasi
cargo build --release

# Profile aplikasi
cargo flamegraph --bin viral_cast_ai_backend

# Memory usage analysis
valgrind --tool=massif target/release/viral_cast_ai_backend
```

## 🔒 Security Features

### Authentication & Authorization
- **JWT Tokens**: Access dan refresh token dengan expiry
- **Password Hashing**: Argon2 untuk hashing password
- **Role-based Access**: Sistem role dan permission
- **CORS Protection**: Konfigurasi CORS untuk frontend

### Data Protection
- **SQL Injection Prevention**: Prepared statements dengan SQLx
- **Input Validation**: Validator crate untuk validasi data
- **Rate Limiting**: Tower middleware untuk rate limiting
- **HTTPS Ready**: TLS support untuk production

### API Security
- **Request Size Limits**: Pembatasan ukuran request
- **File Upload Security**: Validasi tipe dan ukuran file
- **Error Handling**: Structured error response tanpa leak info

## 📊 Monitoring & Logging

### Logging Configuration

```rust
// Konfigurasi logging di main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
    ))
    .with(tracing_subscriber::fmt::layer())
    .init();
```

### Health Checks

```bash
# Basic health check
curl http://localhost:12000/api/v1/healthchecker

# Database health check
curl http://localhost:12000/api/v1/health/database

# Redis health check
curl http://localhost:12000/api/v1/health/redis
```

### Metrics Collection

- **Request Metrics**: Response time, status codes
- **Database Metrics**: Connection pool, query performance
- **AI Usage Metrics**: Token usage, API calls
- **Error Tracking**: Error rates dan types

## 🚀 Production Deployment

### Environment Variables untuk Production

```env
APP_ENV=production
APP_PORT=4000
RUST_LOG=info
DATABASE_MAX_CONNECTIONS=20
REDIS_MAX_CONNECTIONS=10
```

### Docker/Podman Build

```bash
# Build production image
podman build -t viral-cast-ai-backend:latest .

# Run dengan production config
podman run -d \
  --name vcai-backend \
  -p 4000:4000 \
  --env-file .env.production \
  viral-cast-ai-backend:latest
```

### Performance Tuning

```toml
# Cargo.toml optimizations
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## 🐛 Troubleshooting

### Common Issues

**1. Database Connection Error**
```bash
# Check database status
podman compose ps vcai-postgres

# Check logs
podman compose logs vcai-postgres

# Restart database
podman compose restart vcai-postgres
```

**2. Migration Errors**
```bash
# Reset database (DANGER: akan hapus semua data)
sqlx database drop
sqlx database create
sqlx migrate run
```

**3. Redis Connection Issues**
```bash
# Test Redis connection
redis-cli -h 127.0.0.1 -p 6379 ping

# Check Redis logs
podman compose logs vcai-redis
```

**4. Milvus Vector Database Issues**
```bash
# Check Milvus status
curl http://localhost:19530/health

# Restart Milvus stack
podman compose restart vcai-etcd vcai-minio vcai-pulsar vcai-milvus
```

### Debug Mode

```bash
# Run dengan debug logging
RUST_LOG=debug cargo run

# Enable SQL query logging
RUST_LOG=sqlx=debug cargo run

# Full debug mode
RUST_LOG=trace cargo run
```

## 📚 Additional Resources

- **Axum Documentation**: https://docs.rs/axum/
- **SQLx Documentation**: https://docs.rs/sqlx/
- **Tokio Documentation**: https://docs.rs/tokio/
- **Rust Book**: https://doc.rust-lang.org/book/
- **Milvus Documentation**: https://milvus.io/docs/

## 🤝 Contributing

1. Fork repository
2. Buat feature branch
3. Ikuti coding standards (rustfmt + clippy)
4. Tambahkan tests untuk fitur baru
5. Update dokumentasi jika diperlukan
6. Submit pull request

### Code Review Checklist

- [ ] Code formatted dengan `cargo fmt`
- [ ] No clippy warnings
- [ ] Tests pass
- [ ] Documentation updated
- [ ] Security considerations reviewed
- [ ] Performance impact assessed