# Viral Cast AI - Technical Architecture Documentation

## 📋 Overview

Dokumen ini menjelaskan arsitektur teknis lengkap dari sistem Viral Cast AI, termasuk desain sistem, alur data, keamanan, dan pertimbangan skalabilitas.

## 🏗️ System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                 VIRAL CAST AI                                   │
│                              System Architecture                                │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client Layer  │    │  Frontend App   │    │  Backend API    │    │  Data Layer     │
│                 │    │                 │    │                 │    │                 │
│ • Web Browser   │◄──►│ • SvelteKit     │◄──►│ • Rust/Axum     │◄──►│ • PostgreSQL    │
│ • Mobile App    │    │ • Tailwind CSS  │    │ • JWT Auth      │    │ • Redis Cache   │
│ • Desktop App   │    │ • TypeScript    │    │ • SQLx ORM      │    │ • Milvus Vector │
│                 │    │ • Vite          │    │ • Tokio Runtime │    │ • File Storage  │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                │                        │
                       ┌─────────────────┐    ┌─────────────────┐
                       │  External APIs  │    │  AI Services    │
                       │                 │    │                 │
                       │ • GROQ AI       │    │ • RAG System    │
                       │ • Serper News   │    │ • Vector Search │
                       │ • BMKG Weather  │    │ • LLM Chat      │
                       │ • Xendit Pay    │    │ • Predictions   │
                       └─────────────────┘    └─────────────────┘
```

### Component Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              BACKEND SERVICES                                   │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   API Gateway   │    │  Core Services  │    │  Data Services  │
│                 │    │                 │    │                 │
│ • Rate Limiting │    │ • Auth Service  │    │ • User Repo     │
│ • CORS Handler  │    │ • Store Service │    │ • Product Repo  │
│ • Request Log   │    │ • Order Service │    │ • Order Repo    │
│ • Error Handler │    │ • AI Service    │    │ • Cache Layer   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            INFRASTRUCTURE                                       │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Databases     │    │   Messaging     │    │   Monitoring    │
│                 │    │                 │    │                 │
│ • PostgreSQL    │    │ • Apache Pulsar │    │ • Logging       │
│ • Redis Cache   │    │ • Event Streams │    │ • Metrics       │
│ • Milvus Vector │    │ • Message Queue │    │ • Health Checks │
│ • MinIO Storage │    │ • Pub/Sub       │    │ • Alerts        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🔄 Data Flow Architecture

### Request Flow

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              REQUEST FLOW                                       │
└─────────────────────────────────────────────────────────────────────────────────┘

1. Client Request
   │
   ├─► Frontend (SvelteKit)
   │   ├─► Route Handler
   │   ├─► API Service Layer
   │   └─► HTTP Request
   │
   ├─► Backend (Rust/Axum)
   │   ├─► Middleware Stack
   │   │   ├─► CORS Handler
   │   │   ├─► Rate Limiter
   │   │   ├─► JWT Validator
   │   │   └─► Request Logger
   │   │
   │   ├─► Route Handler
   │   ├─► Business Logic
   │   ├─► Data Access Layer
   │   └─► Database Query
   │
   └─► Response
       ├─► JSON Serialization
       ├─► HTTP Response
       └─► Frontend Update
```

### Authentication Flow

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           AUTHENTICATION FLOW                                   │
└─────────────────────────────────────────────────────────────────────────────────┘

1. User Login
   │
   ├─► Frontend Form Submission
   │   ├─► Email/Password Validation
   │   └─► POST /api/v1/auth/login
   │
   ├─► Backend Authentication
   │   ├─► User Lookup (PostgreSQL)
   │   ├─► Password Verification (Argon2)
   │   ├─► JWT Token Generation
   │   │   ├─► Access Token (15 min)
   │   │   └─► Refresh Token (7 days)
   │   └─► Token Storage (Redis)
   │
   └─► Frontend Token Management
       ├─► Store in HTTP-only Cookie
       ├─► Update Auth State
       └─► Redirect to Dashboard

2. Protected Route Access
   │
   ├─► Request with JWT Token
   ├─► Backend Token Validation
   │   ├─► Token Signature Check
   │   ├─► Expiry Validation
   │   └─► User Permission Check
   │
   └─► Allow/Deny Access
```

### AI Service Flow

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              AI SERVICE FLOW                                    │
└─────────────────────────────────────────────────────────────────────────────────┘

1. AI Chat Request
   │
   ├─► Frontend Chat Interface
   │   ├─► User Message Input
   │   └─► POST /api/ai/chat
   │
   ├─► Backend AI Service
   │   ├─► Token Usage Check
   │   ├─► Rate Limiting
   │   ├─► Context Preparation
   │   └─► GROQ API Call
   │
   ├─► RAG Enhancement (Optional)
   │   ├─► Vector Search (Milvus)
   │   ├─► Document Retrieval
   │   └─► Context Augmentation
   │
   └─► Response Generation
       ├─► LLM Response
       ├─► Token Usage Tracking
       └─► Frontend Display

2. Prediction Service
   │
   ├─► Scheduled Job / User Request
   ├─► Data Collection
   │   ├─► Historical Sales Data
   │   ├─► Weather Data (BMKG)
   │   ├─► Market Trends (Serper)
   │   └─► Seasonal Patterns
   │
   ├─► AI Analysis
   │   ├─► Feature Engineering
   │   ├─► Model Inference
   │   └─► Confidence Scoring
   │
   └─► Prediction Storage
       ├─► Database Update
       ├─► Cache Refresh
       └─► Notification Trigger
```

## 🗄️ Database Design

### PostgreSQL Schema

```sql
-- Core Tables
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    role VARCHAR(50) DEFAULT 'user',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    phone VARCHAR(20),
    address TEXT,
    avatar_url VARCHAR(500),
    bio TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE stores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    address TEXT,
    phone VARCHAR(20),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Product Management
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES categories(id),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE ingredient_catalog (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    category_id UUID REFERENCES categories(id),
    unit VARCHAR(50),
    description TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE ingredient_market_prices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ingredient_id UUID REFERENCES ingredient_catalog(id),
    price DECIMAL(10,2) NOT NULL,
    market_location VARCHAR(255),
    date DATE NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Order Management
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID REFERENCES stores(id),
    customer_name VARCHAR(255),
    customer_phone VARCHAR(20),
    total_amount DECIMAL(10,2) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    order_date TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    product_name VARCHAR(255) NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    total_price DECIMAL(10,2) NOT NULL
);

-- Payment Management
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID REFERENCES orders(id),
    amount DECIMAL(10,2) NOT NULL,
    payment_method VARCHAR(50),
    status VARCHAR(50) DEFAULT 'pending',
    external_reference VARCHAR(255),
    payment_date TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- AI & Analytics
CREATE TABLE ai_chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    session_id VARCHAR(255) NOT NULL,
    messages JSONB NOT NULL,
    token_usage INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID REFERENCES stores(id),
    prediction_type VARCHAR(50) NOT NULL, -- 'product', 'ingredient', 'sales'
    target_item VARCHAR(255),
    predicted_value DECIMAL(10,2),
    confidence_score DECIMAL(3,2),
    prediction_date DATE NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### Redis Cache Strategy

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              REDIS CACHE DESIGN                                 │
└─────────────────────────────────────────────────────────────────────────────────┘

Cache Keys Pattern:
├─► auth:tokens:{user_id}           # JWT refresh tokens
├─► auth:sessions:{session_id}      # Active user sessions
├─► user:profile:{user_id}          # User profile data
├─► store:data:{store_id}           # Store information
├─► products:list:{store_id}        # Product listings
├─► orders:stats:{store_id}         # Order statistics
├─► predictions:{store_id}:{type}   # AI predictions
├─► market:prices:{date}            # Market price data
└─► api:rate_limit:{user_id}        # Rate limiting counters

Cache TTL Strategy:
├─► Auth tokens: 7 days
├─► User profiles: 1 hour
├─► Product data: 30 minutes
├─► Statistics: 15 minutes
├─► Predictions: 6 hours
├─► Market prices: 1 day
└─► Rate limits: 1 hour
```

### Milvus Vector Database

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           VECTOR DATABASE DESIGN                                │
└─────────────────────────────────────────────────────────────────────────────────┘

Collection: rag_chunks
├─► id: Primary key (auto-generated)
├─► vector: Dense vector (768 dimensions)
├─► content: Original text content
├─► metadata: JSON metadata
│   ├─► document_id: Source document ID
│   ├─► chunk_index: Position in document
│   ├─► document_type: Type of document
│   └─► created_at: Timestamp
└─► embedding_model: Model used for embedding

Index Configuration:
├─► Index Type: IVF_FLAT
├─► Metric Type: COSINE
├─► nlist: 1024
└─► Search Parameters:
    ├─► nprobe: 10
    └─► top_k: 5
```

## 🔐 Security Architecture

### Authentication & Authorization

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           SECURITY ARCHITECTURE                                 │
└─────────────────────────────────────────────────────────────────────────────────┘

1. Authentication Layer
   ├─► JWT Token-based Authentication
   │   ├─► RS256 Algorithm (RSA + SHA256)
   │   ├─► Access Token: 15 minutes TTL
   │   ├─► Refresh Token: 7 days TTL
   │   └─► Token Rotation on Refresh
   │
   ├─► Password Security
   │   ├─► Argon2id Hashing Algorithm
   │   ├─► Salt: 32 bytes random
   │   ├─► Memory: 64MB
   │   ├─► Iterations: 3
   │   └─► Parallelism: 4 threads
   │
   └─► Session Management
       ├─► Redis-based Session Store
       ├─► Session Invalidation
       └─► Concurrent Session Limits

2. Authorization Layer
   ├─► Role-Based Access Control (RBAC)
   │   ├─► Roles: admin, manager, user
   │   ├─► Permissions: create, read, update, delete
   │   └─► Resource-based Authorization
   │
   ├─► API Route Protection
   │   ├─► JWT Middleware Validation
   │   ├─► Role-based Route Guards
   │   └─► Resource Ownership Checks
   │
   └─► Frontend Route Protection
       ├─► SvelteKit Load Functions
       ├─► Client-side Route Guards
       └─► Conditional UI Rendering

3. Data Protection
   ├─► Input Validation & Sanitization
   │   ├─► Serde Validation (Rust)
   │   ├─► SQL Injection Prevention
   │   ├─► XSS Protection
   │   └─► CSRF Protection
   │
   ├─► Database Security
   │   ├─► Connection Encryption (TLS)
   │   ├─► Prepared Statements (SQLx)
   │   ├─► Database User Permissions
   │   └─► Connection Pooling Limits
   │
   └─► API Security
       ├─► Rate Limiting (Tower middleware)
       ├─► Request Size Limits
       ├─► CORS Configuration
       └─► Security Headers
```

### Security Headers

```rust
// Security headers implementation
pub fn security_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    
    // Prevent XSS attacks
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    
    // HTTPS enforcement
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap()
    );
    
    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
            .parse().unwrap()
    );
    
    headers
}
```

## 📈 Scalability & Performance

### Horizontal Scaling Strategy

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           SCALABILITY ARCHITECTURE                              │
└─────────────────────────────────────────────────────────────────────────────────┘

1. Application Layer Scaling
   ├─► Load Balancer (Nginx/HAProxy)
   │   ├─► Round-robin Distribution
   │   ├─► Health Check Endpoints
   │   └─► SSL Termination
   │
   ├─► Backend Service Instances
   │   ├─► Stateless Design
   │   ├─► Horizontal Pod Autoscaling
   │   ├─► Resource Limits & Requests
   │   └─► Graceful Shutdown Handling
   │
   └─► Frontend CDN Distribution
       ├─► Static Asset Caching
       ├─► Geographic Distribution
       └─► Edge Computing

2. Database Scaling
   ├─► PostgreSQL
   │   ├─► Read Replicas
   │   ├─► Connection Pooling (PgBouncer)
   │   ├─► Query Optimization
   │   └─► Partitioning Strategy
   │
   ├─► Redis Cluster
   │   ├─► Master-Slave Replication
   │   ├─► Sentinel for High Availability
   │   ├─► Cluster Mode for Sharding
   │   └─► Memory Optimization
   │
   └─► Milvus Vector Database
       ├─► Distributed Deployment
       ├─► Index Optimization
       └─► Query Performance Tuning

3. Caching Strategy
   ├─► Multi-level Caching
   │   ├─► Browser Cache (Static Assets)
   │   ├─► CDN Cache (Global Distribution)
   │   ├─► Application Cache (Redis)
   │   └─► Database Query Cache
   │
   ├─► Cache Invalidation
   │   ├─► Time-based Expiration
   │   ├─► Event-driven Invalidation
   │   └─► Cache Warming Strategies
   │
   └─► Cache Patterns
       ├─► Cache-Aside Pattern
       ├─► Write-Through Pattern
       └─► Write-Behind Pattern
```

### Performance Optimization

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         PERFORMANCE OPTIMIZATION                                │
└─────────────────────────────────────────────────────────────────────────────────┘

1. Backend Optimization (Rust)
   ├─► Async/Await with Tokio
   │   ├─► Non-blocking I/O Operations
   │   ├─► Concurrent Request Handling
   │   └─► Efficient Resource Utilization
   │
   ├─► Database Optimization
   │   ├─► Connection Pooling (SQLx)
   │   ├─► Prepared Statement Caching
   │   ├─► Query Optimization
   │   └─► Index Strategy
   │
   ├─► Memory Management
   │   ├─► Zero-copy Serialization
   │   ├─► Efficient Data Structures
   │   └─► Memory Pool Usage
   │
   └─► Compilation Optimization
       ├─► Release Profile Tuning
       ├─► Link-time Optimization (LTO)
       └─► Target CPU Optimization

2. Frontend Optimization (SvelteKit)
   ├─► Bundle Optimization
   │   ├─► Code Splitting
   │   ├─► Tree Shaking
   │   ├─► Dynamic Imports
   │   └─► Chunk Size Optimization
   │
   ├─► Runtime Performance
   │   ├─► Svelte Compilation
   │   ├─► Virtual DOM Elimination
   │   ├─► Reactive Updates
   │   └─► Component Lazy Loading
   │
   ├─► Asset Optimization
   │   ├─► Image Optimization
   │   ├─► Font Loading Strategy
   │   ├─► CSS Purging
   │   └─► Compression (Gzip/Brotli)
   │
   └─► Network Optimization
       ├─► HTTP/2 Server Push
       ├─► Resource Preloading
       ├─► Service Worker Caching
       └─► API Response Caching

3. Monitoring & Profiling
   ├─► Application Metrics
   │   ├─► Response Time Tracking
   │   ├─► Throughput Monitoring
   │   ├─► Error Rate Analysis
   │   └─► Resource Usage Metrics
   │
   ├─► Database Performance
   │   ├─► Query Performance Analysis
   │   ├─► Connection Pool Monitoring
   │   ├─► Index Usage Statistics
   │   └─► Slow Query Logging
   │
   └─► Infrastructure Monitoring
       ├─► CPU & Memory Usage
       ├─► Network I/O Metrics
       ├─► Disk Usage & I/O
       └─► Container Resource Usage
```

## 🔄 CI/CD Pipeline

### Development Workflow

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              CI/CD PIPELINE                                     │
└─────────────────────────────────────────────────────────────────────────────────┘

1. Development Phase
   ├─► Local Development
   │   ├─► Feature Branch Creation
   │   ├─► Code Implementation
   │   ├─► Unit Testing
   │   └─► Local Integration Testing
   │
   ├─► Code Quality Checks
   │   ├─► Rust: cargo fmt, clippy, audit
   │   ├─► Frontend: ESLint, Prettier, TypeScript
   │   ├─► Security Scanning
   │   └─► Dependency Vulnerability Check
   │
   └─► Pull Request
       ├─► Automated Testing
       ├─► Code Review
       ├─► Integration Tests
       └─► Approval Process

2. CI Pipeline (GitHub Actions)
   ├─► Build Stage
   │   ├─► Backend: Rust Compilation
   │   ├─► Frontend: SvelteKit Build
   │   ├─► Docker Image Building
   │   └─► Artifact Generation
   │
   ├─► Test Stage
   │   ├─► Unit Tests (Backend & Frontend)
   │   ├─► Integration Tests
   │   ├─► E2E Tests (Playwright)
   │   └─► Performance Tests
   │
   ├─► Security Stage
   │   ├─► SAST (Static Analysis)
   │   ├─► Dependency Scanning
   │   ├─► Container Scanning
   │   └─► Secret Detection
   │
   └─► Quality Gate
       ├─► Test Coverage Threshold
       ├─► Code Quality Metrics
       ├─► Security Compliance
       └─► Performance Benchmarks

3. CD Pipeline
   ├─► Staging Deployment
   │   ├─► Database Migration
   │   ├─► Service Deployment
   │   ├─► Smoke Tests
   │   └─► Integration Verification
   │
   ├─► Production Deployment
   │   ├─► Blue-Green Deployment
   │   ├─► Health Check Validation
   │   ├─► Traffic Routing
   │   └─► Rollback Capability
   │
   └─► Post-Deployment
       ├─► Monitoring Setup
       ├─► Alert Configuration
       ├─► Performance Validation
       └─► Documentation Update
```

## 🐳 Container Architecture

### Docker Configuration

```dockerfile
# Backend Dockerfile (Multi-stage)
FROM rust:1.75-alpine AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY src ./src
RUN cargo build --release

FROM alpine:3.18
RUN apk add --no-cache ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/viral_cast_ai_backend .
EXPOSE 4000
CMD ["./viral_cast_ai_backend"]

# Frontend Dockerfile (Multi-stage)
FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN npm install -g pnpm && pnpm install --frozen-lockfile
COPY . .
RUN pnpm run build

FROM node:20-alpine
WORKDIR /app
RUN npm install -g pnpm
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --prod --frozen-lockfile
COPY --from=builder /app/build ./build
EXPOSE 5544
CMD ["node", "build"]
```

### Kubernetes Deployment

```yaml
# Backend Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vcai-backend
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
              key: database-url
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
# Frontend Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vcai-frontend
spec:
  replicas: 2
  selector:
    matchLabels:
      app: vcai-frontend
  template:
    metadata:
      labels:
        app: vcai-frontend
    spec:
      containers:
      - name: frontend
        image: viral-cast-ai-frontend:latest
        ports:
        - containerPort: 5544
        env:
        - name: API_BASE_URL
          value: "http://vcai-backend-service:4000"
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "200m"
```

## 📊 Monitoring & Observability

### Logging Strategy

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           MONITORING ARCHITECTURE                               │
└─────────────────────────────────────────────────────────────────────────────────┘

1. Application Logging
   ├─► Structured Logging (JSON)
   │   ├─► Request/Response Logging
   │   ├─► Error Tracking
   │   ├─► Performance Metrics
   │   └─► Business Event Logging
   │
   ├─► Log Levels
   │   ├─► ERROR: System errors, exceptions
   │   ├─► WARN: Performance issues, deprecations
   │   ├─► INFO: Business events, API calls
   │   └─► DEBUG: Detailed execution flow
   │
   └─► Log Aggregation
       ├─► Centralized Log Collection
       ├─► Log Parsing & Indexing
       ├─► Search & Analytics
       └─► Alert Configuration

2. Metrics Collection
   ├─► Application Metrics
   │   ├─► Request Rate & Latency
   │   ├─► Error Rate & Types
   │   ├─► Database Query Performance
   │   └─► AI Service Usage
   │
   ├─► Infrastructure Metrics
   │   ├─► CPU & Memory Usage
   │   ├─► Network I/O
   │   ├─► Disk Usage & I/O
   │   └─► Container Resource Usage
   │
   └─► Business Metrics
       ├─► User Activity
       ├─► Order Processing
       ├─► Revenue Tracking
       └─► Feature Usage

3. Health Monitoring
   ├─► Health Check Endpoints
   │   ├─► Application Health
   │   ├─► Database Connectivity
   │   ├─► External Service Status
   │   └─► Resource Availability
   │
   ├─► Alerting System
   │   ├─► Threshold-based Alerts
   │   ├─► Anomaly Detection
   │   ├─► Escalation Policies
   │   └─► Notification Channels
   │
   └─► Dashboard & Visualization
       ├─► Real-time Monitoring
       ├─► Historical Trends
       ├─► Performance Analytics
       └─► Business Intelligence
```

## 🚀 Deployment Strategy

### Environment Configuration

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           DEPLOYMENT ENVIRONMENTS                               │
└─────────────────────────────────────────────────────────────────────────────────┘

1. Development Environment
   ├─► Local Development
   │   ├─► Docker Compose Setup
   │   ├─► Hot Reload Enabled
   │   ├─► Debug Logging
   │   └─► Mock External Services
   │
   ├─► Configuration
   │   ├─► .env-ai files
   │   ├─► Development Database
   │   ├─► Local Redis Cache
   │   └─► Test API Keys
   │
   └─► Services
       ├─► Backend: localhost:12000
       ├─► Frontend: localhost:5174
       ├─► PostgreSQL: localhost:5432
       └─► Redis: localhost:6379

2. Staging Environment
   ├─► Pre-production Testing
   │   ├─► Production-like Setup
   │   ├─► Integration Testing
   │   ├─► Performance Testing
   │   └─► Security Testing
   │
   ├─► Configuration
   │   ├─► Staging Database
   │   ├─► Staging API Keys
   │   ├─► SSL Certificates
   │   └─► Monitoring Setup
   │
   └─► Deployment Process
       ├─► Automated Deployment
       ├─► Database Migration
       ├─► Smoke Tests
       └─► Rollback Capability

3. Production Environment
   ├─► High Availability Setup
   │   ├─► Load Balancer
   │   ├─► Multiple Instances
   │   ├─► Database Clustering
   │   └─► CDN Integration
   │
   ├─► Security Configuration
   │   ├─► SSL/TLS Encryption
   │   ├─► Firewall Rules
   │   ├─► Secret Management
   │   └─► Access Controls
   │
   └─► Monitoring & Alerting
       ├─► Real-time Monitoring
       ├─► Performance Metrics
       ├─► Error Tracking
       └─► Business Analytics
```

## 🔧 Development Best Practices

### Code Organization

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           DEVELOPMENT BEST PRACTICES                            │
└─────────────────────────────────────────────────────────────────────────────────┘

1. Backend (Rust) Best Practices
   ├─► Project Structure
   │   ├─► Domain-driven Design
   │   ├─► Layered Architecture
   │   ├─► Dependency Injection
   │   └─► Error Handling Strategy
   │
   ├─► Code Quality
   │   ├─► Rust Idioms & Patterns
   │   ├─► Memory Safety
   │   ├─► Concurrency Safety
   │   └─► Performance Optimization
   │
   └─► Testing Strategy
       ├─► Unit Tests (cargo test)
       ├─► Integration Tests
       ├─► Property-based Testing
       └─► Benchmark Tests

2. Frontend (SvelteKit) Best Practices
   ├─► Component Design
   │   ├─► Single Responsibility
   │   ├─► Reusable Components
   │   ├─► Props Validation
   │   └─► Event Handling
   │
   ├─► State Management
   │   ├─► Svelte Stores
   │   ├─► Reactive Declarations
   │   ├─► Context API
   │   └─► Local Component State
   │
   └─► Performance
       ├─► Code Splitting
       ├─► Lazy Loading
       ├─► Bundle Optimization
       └─► Caching Strategy

3. Database Best Practices
   ├─► Schema Design
   │   ├─► Normalization
   │   ├─► Index Strategy
   │   ├─► Constraint Definition
   │   └─► Migration Management
   │
   ├─► Query Optimization
   │   ├─► Prepared Statements
   │   ├─► Query Planning
   │   ├─► Connection Pooling
   │   └─► Transaction Management
   │
   └─► Data Security
       ├─► Access Controls
       ├─► Data Encryption
       ├─► Audit Logging
       └─► Backup Strategy
```

## 📚 Technology Stack Summary

### Backend Technologies
- **Language**: Rust 1.75+
- **Framework**: Axum (async web framework)
- **Runtime**: Tokio (async runtime)
- **Database ORM**: SQLx (compile-time checked queries)
- **Authentication**: JWT with RS256
- **Password Hashing**: Argon2id
- **Serialization**: Serde (JSON)
- **HTTP Client**: Reqwest
- **Logging**: Tracing + Tracing-subscriber
- **Testing**: Built-in test framework + Tokio-test

### Frontend Technologies
- **Framework**: SvelteKit 2.0+
- **Language**: TypeScript 5.0+
- **Build Tool**: Vite 7.0+
- **CSS Framework**: Tailwind CSS 4.0+
- **UI Components**: Custom component library
- **State Management**: Svelte stores
- **HTTP Client**: Fetch API
- **Testing**: Vitest + Playwright
- **Linting**: ESLint + Prettier
- **Package Manager**: pnpm

### Database & Storage
- **Primary Database**: PostgreSQL 15+
- **Cache**: Redis 7.0+
- **Vector Database**: Milvus 2.3+
- **Object Storage**: MinIO
- **Message Queue**: Apache Pulsar
- **Key-Value Store**: etcd

### Infrastructure & DevOps
- **Containerization**: Docker + Podman
- **Orchestration**: Kubernetes (optional)
- **CI/CD**: GitHub Actions
- **Monitoring**: Custom metrics + logging
- **Load Balancer**: Nginx/HAProxy
- **SSL/TLS**: Let's Encrypt
- **CDN**: CloudFlare (recommended)

### External Services
- **AI/LLM**: GROQ API (Llama models)
- **News API**: Serper.dev
- **Weather API**: BMKG Indonesia
- **Payment Gateway**: Xendit (QRIS)
- **Email Service**: SMTP (configurable)

## 🔮 Future Architecture Considerations

### Scalability Roadmap
1. **Microservices Migration**: Split monolith into domain services
2. **Event-Driven Architecture**: Implement event sourcing
3. **CQRS Pattern**: Separate read/write operations
4. **GraphQL API**: Flexible data fetching
5. **Real-time Features**: WebSocket integration
6. **Mobile Apps**: React Native/Flutter
7. **Edge Computing**: CDN-based processing
8. **Machine Learning**: On-premise ML models

### Technology Evolution
1. **Rust Ecosystem**: Leverage new crates and features
2. **SvelteKit 3.0**: Upgrade to latest version
3. **Database Sharding**: Horizontal database scaling
4. **Kubernetes Native**: Full cloud-native deployment
5. **Observability**: OpenTelemetry integration
6. **Security**: Zero-trust architecture
7. **Performance**: WebAssembly integration
8. **AI Integration**: Local LLM deployment

---

**Dokumen ini akan terus diperbarui seiring dengan evolusi arsitektur sistem Viral Cast AI.**