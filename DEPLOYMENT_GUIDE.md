# Viral Cast AI - Deployment Guide

## 📋 Overview

Panduan lengkap untuk deployment dan setup production sistem Viral Cast AI, mencakup environment development, staging, dan production.

## 🚀 Quick Deployment Options

### Option 1: Local Development (Recommended untuk Testing)
```bash
# Clone repository
git clone <repository-url>
cd viral-cast-ai

# Setup environment
cp backend/.env.example backend/.env-ai
cp frontend/.env.example frontend/.env-ai

# Start services dengan Podman
cd backend
podman compose up -d

# Install dan jalankan backend
cargo run

# Install dan jalankan frontend (terminal baru)
cd ../frontend
pnpm install
pnpm dev
```

### Option 2: Production Docker Deployment
```bash
# Build dan deploy dengan Docker/Podman
podman compose -f docker-compose.prod.yml up -d
```

### Option 3: Kubernetes Deployment
```bash
# Deploy ke Kubernetes cluster
kubectl apply -f k8s/
```

## 🛠️ Prerequisites

### System Requirements

#### Minimum Requirements
- **CPU**: 2 cores
- **RAM**: 4GB
- **Storage**: 20GB SSD
- **OS**: Linux (Ubuntu 20.04+), macOS, Windows 10+

#### Recommended Requirements
- **CPU**: 4+ cores
- **RAM**: 8GB+
- **Storage**: 50GB+ SSD
- **OS**: Linux (Ubuntu 22.04 LTS)

### Software Dependencies

#### Development Environment
```bash
# Rust (Backend)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Node.js (Frontend)
curl -fsSL https://fnm.vercel.app/install | bash
fnm install 20
fnm use 20

# pnpm (Package Manager)
npm install -g pnpm

# Podman (Container Runtime)
# Ubuntu/Debian
sudo apt update && sudo apt install -y podman podman-compose

# macOS
brew install podman podman-compose

# Windows
winget install RedHat.Podman
```

#### Production Environment
```bash
# Docker/Podman
# Kubernetes (optional)
# Nginx (Load Balancer)
# PostgreSQL 15+
# Redis 7.0+
```

## 🔧 Environment Configuration

### 1. Backend Environment (.env-ai)

```bash
# Application Settings
APP_ENV=development
APP_PORT=12000
CLIENT_ORIGIN=http://localhost:5174

# Database Configuration
DATABASE_URL=postgresql://vcai_user:vcai_password@localhost:5432/vcai_db
REDIS_URL=redis://localhost:6379

# Vector Database (Milvus)
MILVUS_HOST=localhost
MILVUS_PORT=19530
MILVUS_DB_NAME=vcai_vectors

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-here
JWT_EXPIRES_IN=15m
JWT_REFRESH_EXPIRES_IN=7d

# AI Services
GROQ_API_KEY=your-groq-api-key
GROQ_MODEL=llama-3.1-70b-versatile

# External APIs
SERPER_API_KEY=your-serper-api-key
BMKG_API_URL=https://api.bmkg.go.id

# Payment Gateway
XENDIT_SECRET_KEY=your-xendit-secret-key
XENDIT_WEBHOOK_TOKEN=your-webhook-token

# File Upload
MAX_FILE_SIZE=10485760  # 10MB
UPLOAD_PATH=./uploads

# Logging
RUST_LOG=info
LOG_LEVEL=info
```

### 2. Frontend Environment (.env-ai)

```bash
# API Configuration
API_BASE_URL=http://localhost:12000
FRONTEND_URL=http://localhost:5174

# Environment
NODE_ENV=development

# Optional: Analytics
ANALYTICS_ID=your-analytics-id
```

### 3. Production Environment Variables

```bash
# Production Backend (.env.production)
APP_ENV=production
APP_PORT=4000
CLIENT_ORIGIN=https://your-domain.com

DATABASE_URL=postgresql://user:password@db-host:5432/vcai_prod
REDIS_URL=redis://redis-host:6379

# SSL/TLS
SSL_CERT_PATH=/etc/ssl/certs/cert.pem
SSL_KEY_PATH=/etc/ssl/private/key.pem

# Monitoring
SENTRY_DSN=your-sentry-dsn
LOG_LEVEL=warn

# Production Frontend (.env.production)
API_BASE_URL=https://api.your-domain.com
FRONTEND_URL=https://your-domain.com
NODE_ENV=production
```

## 🐳 Container Deployment

### Docker Compose Setup

#### Development (docker-compose.yml)
```yaml
version: '3.8'

services:
  # Database Services
  postgres:
    image: postgres:15-alpine
    container_name: vcai-postgres-dev
    environment:
      POSTGRES_DB: vcai_db
      POSTGRES_USER: vcai_user
      POSTGRES_PASSWORD: vcai_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backend/migrations:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U vcai_user -d vcai_db"]
      interval: 30s
      timeout: 10s
      retries: 3

  redis:
    image: redis:7-alpine
    container_name: vcai-redis-dev
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes

  # Vector Database
  milvus:
    image: milvusdb/milvus:v2.3.0
    container_name: vcai-milvus-dev
    ports:
      - "19530:19530"
    environment:
      ETCD_ENDPOINTS: etcd:2379
      MINIO_ADDRESS: minio:9000
    depends_on:
      - etcd
      - minio

  etcd:
    image: quay.io/coreos/etcd:v3.5.0
    container_name: vcai-etcd-dev
    environment:
      - ETCD_AUTO_COMPACTION_MODE=revision
      - ETCD_AUTO_COMPACTION_RETENTION=1000
      - ETCD_QUOTA_BACKEND_BYTES=4294967296
    command: etcd -advertise-client-urls=http://127.0.0.1:2379 -listen-client-urls http://0.0.0.0:2379 --data-dir /etcd

  minio:
    image: minio/minio:latest
    container_name: vcai-minio-dev
    environment:
      MINIO_ACCESS_KEY: minioadmin
      MINIO_SECRET_KEY: minioadmin
    ports:
      - "9000:9000"
      - "9001:9001"
    command: minio server /data --console-address ":9001"

volumes:
  postgres_data:
  redis_data:
```

#### Production (docker-compose.prod.yml)
```yaml
version: '3.8'

services:
  # Load Balancer
  nginx:
    image: nginx:alpine
    container_name: vcai-nginx
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/ssl/certs
    depends_on:
      - backend
      - frontend

  # Backend Service
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    container_name: vcai-backend
    environment:
      - APP_ENV=production
      - DATABASE_URL=${DATABASE_URL}
      - REDIS_URL=${REDIS_URL}
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:4000/api/v1/healthchecker"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Frontend Service
  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    container_name: vcai-frontend
    environment:
      - NODE_ENV=production
      - API_BASE_URL=${API_BASE_URL}
    restart: unless-stopped

  # Database (Production)
  postgres:
    image: postgres:15-alpine
    container_name: vcai-postgres
    environment:
      POSTGRES_DB: ${POSTGRES_DB}
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_prod_data:/var/lib/postgresql/data
      - ./backups:/backups
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    container_name: vcai-redis
    volumes:
      - redis_prod_data:/data
    restart: unless-stopped
    command: redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}

volumes:
  postgres_prod_data:
  redis_prod_data:
```

### Podman Commands

```bash
# Development
podman compose up -d                    # Start all services
podman compose down                     # Stop all services
podman compose logs -f backend          # View backend logs
podman compose restart backend          # Restart backend service

# Production
podman compose -f docker-compose.prod.yml up -d
podman compose -f docker-compose.prod.yml down
```

## ☸️ Kubernetes Deployment

### Namespace Configuration
```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: viral-cast-ai
```

### ConfigMap & Secrets
```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: vcai-config
  namespace: viral-cast-ai
data:
  APP_ENV: "production"
  APP_PORT: "4000"
  FRONTEND_PORT: "5544"
  LOG_LEVEL: "info"

---
# k8s/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: vcai-secrets
  namespace: viral-cast-ai
type: Opaque
stringData:
  DATABASE_URL: "postgresql://user:password@postgres:5432/vcai_db"
  REDIS_URL: "redis://redis:6379"
  JWT_SECRET: "your-jwt-secret"
  GROQ_API_KEY: "your-groq-api-key"
```

### Database Deployment
```yaml
# k8s/postgres.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: viral-cast-ai
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        env:
        - name: POSTGRES_DB
          value: "vcai_db"
        - name: POSTGRES_USER
          value: "vcai_user"
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: vcai-secrets
              key: postgres-password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc

---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: viral-cast-ai
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
```

### Application Deployment
```yaml
# k8s/backend.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vcai-backend
  namespace: viral-cast-ai
spec:
  replicas: 3
  selector:
    matchLabels:
      app: vcai-backend
  template:
    metadata:
      labels:
        app: vcai-backend
    spec:
      containers:
      - name: backend
        image: viral-cast-ai-backend:latest
        ports:
        - containerPort: 4000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: vcai-secrets
              key: DATABASE_URL
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/v1/healthchecker
            port: 4000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/v1/healthchecker
            port: 4000
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: vcai-backend-service
  namespace: viral-cast-ai
spec:
  selector:
    app: vcai-backend
  ports:
  - port: 4000
    targetPort: 4000
  type: ClusterIP
```

### Ingress Configuration
```yaml
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: vcai-ingress
  namespace: viral-cast-ai
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - api.your-domain.com
    - your-domain.com
    secretName: vcai-tls
  rules:
  - host: api.your-domain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: vcai-backend-service
            port:
              number: 4000
  - host: your-domain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: vcai-frontend-service
            port:
              number: 5544
```

### Deployment Commands
```bash
# Deploy to Kubernetes
kubectl apply -f k8s/

# Check deployment status
kubectl get pods -n viral-cast-ai
kubectl get services -n viral-cast-ai
kubectl get ingress -n viral-cast-ai

# View logs
kubectl logs -f deployment/vcai-backend -n viral-cast-ai
kubectl logs -f deployment/vcai-frontend -n viral-cast-ai

# Scale deployment
kubectl scale deployment vcai-backend --replicas=5 -n viral-cast-ai

# Update deployment
kubectl set image deployment/vcai-backend backend=viral-cast-ai-backend:v2.0.0 -n viral-cast-ai
```

## 🌐 Production Setup

### 1. Server Preparation

#### Ubuntu 22.04 LTS Setup
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install essential packages
sudo apt install -y curl wget git unzip software-properties-common

# Install Docker/Podman
sudo apt install -y podman podman-compose

# Install Nginx
sudo apt install -y nginx

# Install PostgreSQL client
sudo apt install -y postgresql-client

# Install Node.js (for frontend builds)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
npm install -g pnpm

# Install Rust (for backend builds)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. SSL/TLS Configuration

#### Let's Encrypt Setup
```bash
# Install Certbot
sudo apt install -y certbot python3-certbot-nginx

# Obtain SSL certificate
sudo certbot --nginx -d your-domain.com -d api.your-domain.com

# Auto-renewal setup
sudo crontab -e
# Add: 0 12 * * * /usr/bin/certbot renew --quiet
```

#### Nginx Configuration
```nginx
# /etc/nginx/sites-available/viral-cast-ai
upstream backend {
    server 127.0.0.1:4000;
    server 127.0.0.1:4001;  # Additional backend instances
    server 127.0.0.1:4002;
}

upstream frontend {
    server 127.0.0.1:5544;
    server 127.0.0.1:5545;  # Additional frontend instances
}

# Frontend
server {
    listen 80;
    listen [::]:80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
    add_header Content-Security-Policy "default-src 'self' http: https: data: blob: 'unsafe-inline'" always;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;

    location / {
        proxy_pass http://frontend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}

# Backend API
server {
    listen 80;
    listen [::]:80;
    server_name api.your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name api.your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;

    location / {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # CORS headers
        add_header Access-Control-Allow-Origin "https://your-domain.com" always;
        add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Authorization, Content-Type" always;
        
        if ($request_method = 'OPTIONS') {
            return 204;
        }
    }
}
```

### 3. Database Setup

#### PostgreSQL Production Setup
```bash
# Install PostgreSQL
sudo apt install -y postgresql postgresql-contrib

# Create database and user
sudo -u postgres psql
CREATE DATABASE vcai_prod;
CREATE USER vcai_user WITH ENCRYPTED PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE vcai_prod TO vcai_user;
\q

# Configure PostgreSQL
sudo nano /etc/postgresql/14/main/postgresql.conf
# Uncomment and modify:
# listen_addresses = 'localhost'
# max_connections = 100
# shared_buffers = 256MB

sudo nano /etc/postgresql/14/main/pg_hba.conf
# Add:
# local   vcai_prod   vcai_user   md5

# Restart PostgreSQL
sudo systemctl restart postgresql
sudo systemctl enable postgresql
```

#### Redis Production Setup
```bash
# Install Redis
sudo apt install -y redis-server

# Configure Redis
sudo nano /etc/redis/redis.conf
# Modify:
# bind 127.0.0.1
# requirepass your_secure_password
# maxmemory 256mb
# maxmemory-policy allkeys-lru

# Restart Redis
sudo systemctl restart redis-server
sudo systemctl enable redis-server
```

### 4. Application Deployment

#### Backend Deployment Script
```bash
#!/bin/bash
# deploy-backend.sh

set -e

echo "🚀 Deploying Viral Cast AI Backend..."

# Variables
APP_DIR="/opt/viral-cast-ai"
BACKUP_DIR="/opt/backups"
SERVICE_NAME="vcai-backend"

# Create backup
echo "📦 Creating backup..."
mkdir -p $BACKUP_DIR
sudo systemctl stop $SERVICE_NAME || true
cp -r $APP_DIR $BACKUP_DIR/backup-$(date +%Y%m%d-%H%M%S) || true

# Update code
echo "📥 Updating code..."
cd $APP_DIR
git pull origin main

# Build backend
echo "🔨 Building backend..."
cd backend
cargo build --release

# Update systemd service
echo "⚙️ Updating service..."
sudo cp deploy/vcai-backend.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable $SERVICE_NAME
sudo systemctl start $SERVICE_NAME

# Health check
echo "🏥 Health check..."
sleep 10
if curl -f http://localhost:4000/api/v1/healthchecker; then
    echo "✅ Backend deployment successful!"
else
    echo "❌ Backend deployment failed!"
    exit 1
fi
```

#### Frontend Deployment Script
```bash
#!/bin/bash
# deploy-frontend.sh

set -e

echo "🚀 Deploying Viral Cast AI Frontend..."

# Variables
APP_DIR="/opt/viral-cast-ai"
SERVICE_NAME="vcai-frontend"

# Update code
echo "📥 Updating code..."
cd $APP_DIR
git pull origin main

# Build frontend
echo "🔨 Building frontend..."
cd frontend
pnpm install --frozen-lockfile
pnpm run build

# Update systemd service
echo "⚙️ Updating service..."
sudo cp deploy/vcai-frontend.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable $SERVICE_NAME
sudo systemctl restart $SERVICE_NAME

# Health check
echo "🏥 Health check..."
sleep 5
if curl -f http://localhost:5544; then
    echo "✅ Frontend deployment successful!"
else
    echo "❌ Frontend deployment failed!"
    exit 1
fi
```

#### Systemd Service Files

**Backend Service** (`deploy/vcai-backend.service`):
```ini
[Unit]
Description=Viral Cast AI Backend
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=vcai
Group=vcai
WorkingDirectory=/opt/viral-cast-ai/backend
Environment=RUST_LOG=info
EnvironmentFile=/opt/viral-cast-ai/backend/.env.production
ExecStart=/opt/viral-cast-ai/backend/target/release/viral_cast_ai_backend
Restart=always
RestartSec=10

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/viral-cast-ai/backend/uploads

[Install]
WantedBy=multi-user.target
```

**Frontend Service** (`deploy/vcai-frontend.service`):
```ini
[Unit]
Description=Viral Cast AI Frontend
After=network.target

[Service]
Type=simple
User=vcai
Group=vcai
WorkingDirectory=/opt/viral-cast-ai/frontend
Environment=NODE_ENV=production
EnvironmentFile=/opt/viral-cast-ai/frontend/.env.production
ExecStart=/usr/bin/node build
Restart=always
RestartSec=10

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true

[Install]
WantedBy=multi-user.target
```

## 📊 Monitoring & Maintenance

### 1. Health Monitoring

#### Health Check Script
```bash
#!/bin/bash
# health-check.sh

echo "🏥 Viral Cast AI Health Check"
echo "=============================="

# Check services
services=("vcai-backend" "vcai-frontend" "postgresql" "redis-server" "nginx")
for service in "${services[@]}"; do
    if systemctl is-active --quiet $service; then
        echo "✅ $service: Running"
    else
        echo "❌ $service: Not running"
    fi
done

# Check endpoints
echo ""
echo "🌐 Endpoint Health:"
if curl -f -s http://localhost:4000/api/v1/healthchecker > /dev/null; then
    echo "✅ Backend API: Healthy"
else
    echo "❌ Backend API: Unhealthy"
fi

if curl -f -s http://localhost:5544 > /dev/null; then
    echo "✅ Frontend: Healthy"
else
    echo "❌ Frontend: Unhealthy"
fi

# Check database
echo ""
echo "🗄️ Database Health:"
if pg_isready -h localhost -p 5432 -U vcai_user > /dev/null; then
    echo "✅ PostgreSQL: Connected"
else
    echo "❌ PostgreSQL: Connection failed"
fi

if redis-cli ping > /dev/null 2>&1; then
    echo "✅ Redis: Connected"
else
    echo "❌ Redis: Connection failed"
fi

# Check disk space
echo ""
echo "💾 Disk Usage:"
df -h / | tail -1 | awk '{print "Root: " $5 " used"}'
df -h /opt | tail -1 | awk '{print "App: " $5 " used"}' 2>/dev/null || echo "App: Same as root"

# Check memory
echo ""
echo "🧠 Memory Usage:"
free -h | grep Mem | awk '{print "Memory: " $3 "/" $2 " (" int($3/$2*100) "% used)"}'
```

### 2. Backup Strategy

#### Database Backup Script
```bash
#!/bin/bash
# backup-database.sh

BACKUP_DIR="/opt/backups/database"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="vcai_prod"
DB_USER="vcai_user"

mkdir -p $BACKUP_DIR

echo "🗄️ Creating database backup..."
pg_dump -h localhost -U $DB_USER -d $DB_NAME | gzip > $BACKUP_DIR/vcai_backup_$DATE.sql.gz

# Keep only last 7 days of backups
find $BACKUP_DIR -name "vcai_backup_*.sql.gz" -mtime +7 -delete

echo "✅ Database backup completed: vcai_backup_$DATE.sql.gz"
```

#### Application Backup Script
```bash
#!/bin/bash
# backup-app.sh

BACKUP_DIR="/opt/backups/application"
APP_DIR="/opt/viral-cast-ai"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

echo "📦 Creating application backup..."
tar -czf $BACKUP_DIR/vcai_app_$DATE.tar.gz -C /opt viral-cast-ai --exclude='target' --exclude='node_modules' --exclude='.git'

# Keep only last 3 backups
ls -t $BACKUP_DIR/vcai_app_*.tar.gz | tail -n +4 | xargs rm -f

echo "✅ Application backup completed: vcai_app_$DATE.tar.gz"
```

### 3. Log Management

#### Log Rotation Configuration
```bash
# /etc/logrotate.d/viral-cast-ai
/opt/viral-cast-ai/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 vcai vcai
    postrotate
        systemctl reload vcai-backend
        systemctl reload vcai-frontend
    endscript
}
```

### 4. Automated Maintenance

#### Crontab Setup
```bash
# Edit crontab
sudo crontab -e

# Add maintenance tasks
# Database backup (daily at 2 AM)
0 2 * * * /opt/viral-cast-ai/scripts/backup-database.sh

# Application backup (weekly on Sunday at 3 AM)
0 3 * * 0 /opt/viral-cast-ai/scripts/backup-app.sh

# Health check (every 5 minutes)
*/5 * * * * /opt/viral-cast-ai/scripts/health-check.sh >> /var/log/vcai-health.log

# Log cleanup (monthly)
0 1 1 * * find /opt/viral-cast-ai/logs -name "*.log" -mtime +30 -delete

# SSL certificate renewal check (daily)
0 12 * * * /usr/bin/certbot renew --quiet
```

## 🚨 Troubleshooting

### Common Issues

#### 1. Backend Won't Start
```bash
# Check logs
journalctl -u vcai-backend -f

# Common solutions:
# - Check database connection
# - Verify environment variables
# - Check port availability
sudo netstat -tlnp | grep :4000

# Restart services
sudo systemctl restart postgresql
sudo systemctl restart redis-server
sudo systemctl restart vcai-backend
```

#### 2. Frontend Build Errors
```bash
# Clear cache and reinstall
cd /opt/viral-cast-ai/frontend
rm -rf node_modules .svelte-kit
pnpm install
pnpm run build

# Check Node.js version
node --version  # Should be 20+
```

#### 3. Database Connection Issues
```bash
# Test connection
psql -h localhost -U vcai_user -d vcai_prod

# Check PostgreSQL status
sudo systemctl status postgresql

# Check configuration
sudo nano /etc/postgresql/14/main/pg_hba.conf
```

#### 4. SSL Certificate Issues
```bash
# Check certificate status
sudo certbot certificates

# Renew certificate
sudo certbot renew --force-renewal

# Test Nginx configuration
sudo nginx -t
sudo systemctl reload nginx
```

### Performance Optimization

#### 1. Database Optimization
```sql
-- Check slow queries
SELECT query, mean_time, calls 
FROM pg_stat_statements 
ORDER BY mean_time DESC 
LIMIT 10;

-- Analyze table statistics
ANALYZE;

-- Reindex if needed
REINDEX DATABASE vcai_prod;
```

#### 2. Redis Optimization
```bash
# Check Redis memory usage
redis-cli info memory

# Monitor Redis performance
redis-cli monitor

# Optimize configuration
sudo nano /etc/redis/redis.conf
```

#### 3. Application Performance
```bash
# Monitor system resources
htop
iotop
nethogs

# Check application metrics
curl http://localhost:4000/api/v1/metrics
```

## 📈 Scaling Strategies

### Horizontal Scaling

#### 1. Load Balancer Setup
```nginx
# Enhanced Nginx configuration for load balancing
upstream backend_pool {
    least_conn;
    server 10.0.1.10:4000 weight=3;
    server 10.0.1.11:4000 weight=3;
    server 10.0.1.12:4000 weight=2;
    keepalive 32;
}

upstream frontend_pool {
    ip_hash;
    server 10.0.1.20:5544;
    server 10.0.1.21:5544;
    keepalive 16;
}
```

#### 2. Database Scaling
```bash
# PostgreSQL read replicas setup
# Master-slave replication configuration
# Connection pooling with PgBouncer

# Redis cluster setup
redis-cli --cluster create \
  10.0.1.30:7000 10.0.1.31:7000 10.0.1.32:7000 \
  10.0.1.30:7001 10.0.1.31:7001 10.0.1.32:7001 \
  --cluster-replicas 1
```

### Vertical Scaling

#### Resource Optimization
```bash
# Increase system limits
echo "vcai soft nofile 65536" >> /etc/security/limits.conf
echo "vcai hard nofile 65536" >> /etc/security/limits.conf

# Optimize kernel parameters
echo "net.core.somaxconn = 65536" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 65536" >> /etc/sysctl.conf
sysctl -p
```

## 🔐 Security Hardening

### System Security
```bash
# Firewall setup
sudo ufw enable
sudo ufw allow ssh
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw deny 4000/tcp  # Block direct backend access
sudo ufw deny 5544/tcp  # Block direct frontend access

# Fail2ban setup
sudo apt install -y fail2ban
sudo systemctl enable fail2ban
```

### Application Security
```bash
# Regular security updates
sudo apt update && sudo apt upgrade -y

# Dependency vulnerability scanning
cd /opt/viral-cast-ai/backend
cargo audit

cd /opt/viral-cast-ai/frontend
pnpm audit
```

---

**Panduan deployment ini mencakup semua aspek yang diperlukan untuk menjalankan Viral Cast AI dalam environment production yang aman dan scalable.**