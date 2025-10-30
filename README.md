# Viral Cast AI - Platform Prediksi Bisnis Berbasis AI

![Viral Cast AI](https://img.shields.io/badge/Viral%20Cast%20AI-v1.0.0-blue)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![SvelteKit](https://img.shields.io/badge/SvelteKit-2.0+-green)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15+-blue)
![Redis](https://img.shields.io/badge/Redis-7+-red)

## 📋 Deskripsi Project

Viral Cast AI adalah platform prediksi bisnis berbasis AI yang membantu bisnis dalam:
- **Prediksi Penjualan**: Analisis tren penjualan dan forecasting
- **Manajemen Inventori**: Optimasi stok dan prediksi kebutuhan bahan
- **Analisis Cuaca**: Integrasi data BMKG untuk prediksi berbasis cuaca
- **RAG (Retrieval-Augmented Generation)**: Sistem AI untuk analisis dokumen bisnis
- **Dashboard Analytics**: Visualisasi data dan insights bisnis

## 🏗️ Arsitektur Sistem

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │    Backend      │    │   Database      │
│   (SvelteKit)   │◄──►│   (Rust/Axum)   │◄──►│  (PostgreSQL)   │
│   Port: 5174    │    │   Port: 12000   │    │   Port: 5432    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   Services      │
                    │ • Redis Cache   │
                    │ • Milvus Vector │
                    │ • MinIO Storage │
                    │ • Pulsar Queue  │
                    └─────────────────┘
```

## 🚀 Quick Start

### Prerequisites

Pastikan sistem Anda memiliki:
- **Rust** 1.70+ dengan Cargo
- **Node.js** 18+ dengan pnpm/npm
- **Podman** atau Docker
- **Git**

### 1. Clone Repository

```bash
git clone <repository-url>
cd "Viral Cast AI Final"
```

### 2. Setup Backend

```bash
cd viral-cast-ai-backend

# Copy environment file
cp .env.example .env-ai

# Edit konfigurasi (gunakan .env-ai untuk development)
# Sesuaikan port dan konfigurasi database
```

### 3. Setup Frontend

```bash
cd ../viral-cast-ai-frontend

# Copy environment file
cp .env.example .env

# Install dependencies
pnpm install
# atau
npm install
```

### 4. Jalankan Services dengan Podman

```bash
cd ../viral-cast-ai-backend

# Start semua services (PostgreSQL, Redis, Milvus, dll)
podman compose up -d

# Tunggu hingga semua services ready
podman compose logs -f
```

### 5. Setup Database

```bash
# Install sqlx-cli jika belum ada
cargo install sqlx-cli

# Jalankan migrasi database
sqlx migrate run
```

### 6. Jalankan Aplikasi

**Terminal 1 - Backend:**
```bash
cd viral-cast-ai-backend
cargo run
```

**Terminal 2 - Frontend:**
```bash
cd viral-cast-ai-frontend
pnpm dev
# atau
npm run dev
```

### 7. Akses Aplikasi

- **Frontend**: http://localhost:5174
- **Backend API**: http://localhost:12000
- **API Documentation**: http://localhost:12000/swagger-ui/

## 📁 Struktur Project

```
Viral Cast AI Final/
├── viral-cast-ai-backend/          # Backend Rust/Axum
│   ├── src/
│   │   ├── handlers/               # API handlers
│   │   ├── models/                 # Database models
│   │   ├── services/               # Business logic
│   │   ├── repository/             # Data access layer
│   │   └── routes/                 # API routes
│   ├── migrations/                 # Database migrations
│   ├── compose.yml                 # Podman/Docker services
│   └── Cargo.toml                  # Rust dependencies
│
└── viral-cast-ai-frontend/         # Frontend SvelteKit
    ├── src/
    │   ├── lib/                    # Shared components
    │   └── routes/                 # Pages dan API routes
    ├── static/                     # Static assets
    └── package.json                # Node.js dependencies
```

## ⚙️ Konfigurasi Environment

### Backend (.env-ai)

```env
# Application
APP_ENV=development
APP_PORT=12000
CLIENT_ORIGIN=http://localhost:5174

# Database
POSTGRES_HOST=127.0.0.1
POSTGRES_PORT=5432
POSTGRES_USER=vcai_user
POSTGRES_PASSWORD=vcai_password
POSTGRES_DB=viral_cast_ai

# Redis
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_DB=0

# Vector Database (Milvus)
MILVUS_URI=http://127.0.0.1:19530
MILVUS_COLLECTION=rag_chunks

# AI Configuration
GROQ_API_KEY=your_groq_api_key
GROQ_MODEL=llama-3.1-8b-instant

# News API
SERPER_API_KEY=your_serper_api_key
```

### Frontend (.env)

```env
# Backend API
API_BASE_URL=http://localhost:12000

# Frontend
FRONTEND_URL=http://localhost:5174
NODE_ENV=development
```

## 🔧 Development Workflow

### Backend Development

```bash
# Jalankan dalam mode development dengan auto-reload
cargo watch -x run

# Jalankan tests
cargo test

# Format code
cargo fmt

# Linting
cargo clippy
```

### Frontend Development

```bash
# Development server dengan hot reload
pnpm dev

# Build untuk production
pnpm build

# Preview production build
pnpm preview

# Jalankan tests
pnpm test

# E2E testing dengan Playwright
pnpm test:e2e
```

## 🧪 Testing

### Backend Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --features integration-tests

# Test dengan coverage
cargo tarpaulin --out html
```

### Frontend Testing

```bash
# Unit tests dengan Vitest
pnpm test:unit

# E2E tests dengan Playwright
pnpm test:e2e

# Storybook untuk component testing
pnpm storybook
```

## 📊 Monitoring & Logging

### Backend Logs

```bash
# Lihat logs aplikasi
podman compose logs vcai-app -f

# Lihat logs database
podman compose logs vcai-postgres -f

# Lihat semua logs
podman compose logs -f
```

### Health Checks

- **Backend Health**: http://localhost:12000/health
- **Database**: Automatic health check dalam compose.yml
- **Redis**: Built-in monitoring

## 🚀 Production Deployment

### 1. Build Images

```bash
# Backend
cd viral-cast-ai-backend
podman build -t viral-cast-ai-backend:latest .

# Frontend
cd ../viral-cast-ai-frontend
podman build -t viral-cast-ai-frontend:latest .
```

### 2. Production Environment

```bash
# Set production environment
export APP_ENV=production
export NODE_ENV=production

# Update .env files dengan production values
```

### 3. Deploy dengan Podman

```bash
# Production deployment
podman compose -f compose.yml -f compose.prod.yml up -d
```

## 🔒 Security

### Environment Variables
- **JANGAN** commit file `.env` ke repository
- Gunakan `.env-ai` untuk development
- Gunakan secrets management untuk production

### API Security
- JWT authentication dengan refresh tokens
- CORS configuration
- Rate limiting
- Input validation dengan Validator

### Database Security
- Connection pooling dengan SQLx
- Prepared statements untuk SQL injection prevention
- Database migrations dengan version control

## 🤝 Contributing

1. Fork repository
2. Buat feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push ke branch (`git push origin feature/amazing-feature`)
5. Buat Pull Request

### Code Standards

- **Rust**: Ikuti Rust style guide, gunakan `cargo fmt` dan `cargo clippy`
- **TypeScript**: Ikuti ESLint configuration
- **Commits**: Gunakan conventional commits format

## 📚 API Documentation

API documentation tersedia di:
- **Swagger UI**: http://localhost:12000/swagger-ui/
- **OpenAPI Spec**: http://localhost:12000/api-docs/openapi.json

### Main Endpoints

- `POST /api/auth/login` - User authentication
- `GET /api/profiles` - User profiles
- `GET /api/stores` - Store management
- `POST /api/ai/predictions` - AI predictions
- `GET /api/weather` - Weather data
- `POST /api/rag/upload` - Document upload untuk RAG

## 🛠️ Troubleshooting

### Common Issues

**1. Database Connection Error**
```bash
# Check database status
podman compose ps vcai-postgres

# Restart database
podman compose restart vcai-postgres
```

**2. Port Already in Use**
```bash
# Check port usage
netstat -tulpn | grep :12000

# Kill process using port
kill -9 <PID>
```

**3. Frontend Build Errors**
```bash
# Clear node_modules dan reinstall
rm -rf node_modules package-lock.json
pnpm install
```

**4. Rust Compilation Errors**
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update
```

## 📞 Support

Untuk bantuan dan support:
- **Issues**: Buat issue di repository
- **Documentation**: Lihat folder `docs/` untuk dokumentasi detail
- **API Reference**: Gunakan Swagger UI untuk testing API

## 📄 License

Project ini menggunakan MIT License. Lihat file `LICENSE` untuk detail.

---

**Dibuat dengan ❤️ menggunakan Rust, SvelteKit, dan AI**