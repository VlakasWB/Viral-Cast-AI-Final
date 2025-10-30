# Viral Cast AI Frontend Documentation

## 📋 Overview

Frontend Viral Cast AI dibangun menggunakan **SvelteKit** dengan **Tailwind CSS** untuk UI yang modern dan responsif. Aplikasi ini menyediakan interface untuk manajemen toko, produk, resep, dan integrasi AI.

## 🏗️ Arsitektur Frontend

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   UI Layer      │    │  State & Logic  │    │  API Services   │
│   (Components)  │◄──►│   (Stores)      │◄──►│   (HTTP)        │
│                 │    │                 │    │                 │
│ • Pages         │    │ • Auth Store    │    │ • Auth API      │
│ • Components    │    │ • Theme Store   │    │ • Products API  │
│ • Layouts       │    │ • Cart Store    │    │ • Orders API    │
│ • Forms         │    │ • i18n Store    │    │ • AI Chat API   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🚀 Setup dan Instalasi

### Prerequisites

```bash
# Install Node.js 20+ (recommended: menggunakan nvm)
# Windows: Download dari https://nodejs.org/
# Linux/Mac: 
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20

# Install pnpm (package manager yang direkomendasikan)
npm install -g pnpm

# Verify installations
node --version  # should be 20+
pnpm --version  # should be latest
```

### 1. Environment Setup

```bash
cd viral-cast-ai-frontend

# Copy dan edit environment file
cp .env.example .env-ai

# Edit .env-ai dengan konfigurasi yang sesuai
```

**Konfigurasi .env-ai:**
```env
# Backend API Configuration
API_BASE_URL=http://localhost:12000

# Frontend Development Server
FRONTEND_URL=http://localhost:5174
NODE_ENV=development

# Optional: untuk production preview
# FRONTEND_URL=http://localhost:4173
```

### 2. Install Dependencies

```bash
# Install semua dependencies
pnpm install

# Atau install dengan frozen lockfile (untuk CI/CD)
pnpm install --frozen-lockfile

# Verify installation
pnpm list
```

### 3. Development Server

```bash
# Start development server dengan hot reload
pnpm run dev

# Start dengan port spesifik
pnpm run dev -- --port 5174

# Start dengan host binding (untuk akses dari device lain)
pnpm run dev -- --host 0.0.0.0

# Open browser otomatis
pnpm run dev -- --open
```

**Development server akan berjalan di:**
- Local: `http://localhost:5174`
- Network: `http://[your-ip]:5174`

### 4. Build dan Preview

```bash
# Build untuk production
pnpm run build

# Preview production build
pnpm run preview

# Preview dengan port spesifik
pnpm run preview -- --port 4173

# Check build output
ls -la build/
```

## 📁 Struktur Proyek

```
viral-cast-ai-frontend/
├── src/
│   ├── app.css                 # Global styles & Tailwind imports
│   ├── app.html                # HTML template
│   ├── lib/                    # Shared utilities & components
│   │   ├── components/         # Reusable UI components
│   │   │   ├── Sidebar.svelte
│   │   │   ├── Header.svelte
│   │   │   ├── ShoppingCart.svelte
│   │   │   └── ui/             # Base UI components
│   │   ├── services/           # API service layers
│   │   │   ├── auth.ts
│   │   │   ├── products.ts
│   │   │   ├── orders.ts
│   │   │   └── recipe.ts
│   │   ├── stores/             # Svelte stores (state management)
│   │   │   ├── auth.ts
│   │   │   ├── theme.ts
│   │   │   └── i18n.ts
│   │   ├── types/              # TypeScript type definitions
│   │   ├── utils/              # Utility functions
│   │   └── styles/             # Additional CSS files
│   │       └── themes.css
│   └── routes/                 # SvelteKit file-based routing
│       ├── +layout.svelte      # Root layout
│       ├── +page.svelte        # Home page
│       ├── (app)/              # Protected app routes
│       │   ├── +layout.svelte  # App layout with sidebar
│       │   ├── settings/       # Settings pages
│       │   ├── master/         # Master data management
│       │   │   ├── products/
│       │   │   ├── ingredients/
│       │   │   └── categories/
│       │   ├── orders/         # Order management
│       │   └── ai-chat/        # AI chat interface
│       └── pages/              # Public pages
│           ├── auth/           # Authentication pages
│           │   ├── login/
│           │   └── register/
│           ├── 404/            # Error pages
│           └── maintenance/
├── static/                     # Static assets
├── tests/                      # Test files
├── package.json               # Dependencies & scripts
├── svelte.config.js           # SvelteKit configuration
├── vite.config.ts             # Vite configuration
├── tsconfig.json              # TypeScript configuration
├── eslint.config.js           # ESLint configuration
├── .prettierrc                # Prettier configuration
└── Dockerfile                 # Container configuration
```

## 🎨 UI/UX dan Styling

### Tailwind CSS Configuration

Proyek menggunakan **Tailwind CSS v4** dengan konfigurasi modern:

```css
/* src/app.css */
@import 'tailwindcss';
@plugin '@tailwindcss/forms';
@plugin '@tailwindcss/typography';
@import './lib/styles/themes.css';

/* Custom variant untuk dark mode */
@custom-variant dark (&:is(html[data-mode="dark"] *));
```

### Theme System

**Light Mode (Default):**
```css
:root {
  --color-surface: #fff8f3;     /* Background dengan nuansa orange */
  --color-text: #000000;        /* Teks hitam untuk kontras optimal */
  --accent: #ff3e00;            /* Orange Svelte sebagai accent */
  --app-bg: linear-gradient(135deg, #fff8f3 0%, #fff2e8 100%);
}
```

**Dark Mode:**
```css
html[data-mode="dark"] {
  --color-surface: #1a1a1a;
  --color-text: #ffffff;
  --app-bg: linear-gradient(135deg, #1a1a1a 0%, #2d2d2d 100%);
}
```

### Typography

**Font Stack:**
```css
--font-sans: 'Poppins', 'Manrope', 'Urbanist', 'Plus Jakarta Sans', 'Inter', system-ui, sans-serif;
```

**Loaded dari Google Fonts:**
- **Poppins**: 400, 600, 700
- **Manrope**: 400, 600, 700  
- **Urbanist**: 400, 600, 700

### Component Library

**Base Components:**
- `Button.svelte` - Tombol dengan berbagai variant
- `Input.svelte` - Form input dengan validasi
- `Modal.svelte` - Dialog dan popup
- `Card.svelte` - Container dengan shadow dan border
- `Badge.svelte` - Status dan label
- `Dropdown.svelte` - Menu dropdown

**Layout Components:**
- `Sidebar.svelte` - Navigation sidebar dengan collapse
- `Header.svelte` - Top navigation dengan user menu
- `Breadcrumb.svelte` - Navigation breadcrumb

## 🔧 Development Workflow

### Code Quality Tools

**ESLint Configuration:**
```bash
# Lint semua file
pnpm run lint

# Fix auto-fixable issues
pnpm run lint -- --fix

# Lint specific files
pnpm run lint src/routes/+page.svelte
```

**Prettier Formatting:**
```bash
# Format semua file
pnpm run format

# Check formatting tanpa fix
pnpm run format -- --check

# Format specific files
prettier --write src/lib/components/
```

**TypeScript Checking:**
```bash
# Type check
pnpm run check

# Type check dengan watch mode
pnpm run check:watch

# Sync SvelteKit types
pnpm run sync
```

### Testing

**Unit Testing dengan Vitest:**
```bash
# Run unit tests
pnpm run test:unit

# Run dengan watch mode
pnpm run test:unit -- --watch

# Run dengan coverage
pnpm run test:unit -- --coverage

# Run specific test file
pnpm run test:unit src/lib/utils/helpers.test.ts
```

**E2E Testing dengan Playwright:**
```bash
# Install Playwright browsers
npx playwright install

# Run E2E tests
pnpm run test:e2e

# Run dengan UI mode
pnpm run test:e2e -- --ui

# Run specific test
pnpm run test:e2e tests/auth.spec.ts

# Debug mode
pnpm run test:e2e -- --debug
```

**Storybook untuk Component Development:**
```bash
# Start Storybook server
pnpm run storybook

# Build Storybook
pnpm run build-storybook

# Storybook akan berjalan di http://localhost:6006
```

### Hot Module Replacement (HMR)

SvelteKit mendukung HMR out-of-the-box:
- **Svelte components**: Auto-reload dengan state preservation
- **CSS changes**: Instant update tanpa page refresh
- **TypeScript**: Fast refresh dengan type checking
- **API routes**: Auto-restart development server

## 🌐 Internationalization (i18n)

### Paraglide-JS Integration

Proyek menggunakan **Paraglide-JS** untuk internationalization:

```typescript
// src/lib/stores/i18n.ts
import { t } from '$lib/paraglide/messages';

// Usage dalam component
<script lang="ts">
  import { t } from '$lib/stores/i18n';
</script>

<h1>{t.welcome()}</h1>
<p>{t.userGreeting({ name: 'John' })}</p>
```

**Supported Languages:**
- **Indonesian (ID)**: Default language
- **English (EN)**: Secondary language

**Message Files:**
```
project.inlang/
├── messages/
│   ├── id.json    # Indonesian messages
│   └── en.json    # English messages
└── settings.json  # Paraglide configuration
```

## 🔐 Authentication & Authorization

### Auth Flow

```typescript
// src/lib/services/auth.ts
export async function login(email: string, password: string) {
  const response = await fetch(`${API_BASE_URL}/api/v1/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password })
  });
  
  if (response.ok) {
    const { access_token, refresh_token } = await response.json();
    // Store tokens secara aman
    localStorage.setItem('access_token', access_token);
    localStorage.setItem('refresh_token', refresh_token);
  }
}
```

### Protected Routes

```typescript
// src/routes/(app)/+layout.server.ts
import { redirect } from '@sveltejs/kit';

export async function load({ cookies, url }) {
  const token = cookies.get('access_token');
  
  if (!token) {
    throw redirect(302, `/pages/auth/login?redirect=${url.pathname}`);
  }
  
  // Verify token dengan backend
  // Return user data jika valid
}
```

### Auth Store

```typescript
// src/lib/stores/auth.ts
import { writable } from 'svelte/store';

interface User {
  id: string;
  email: string;
  name: string;
  role: string;
}

export const user = writable<User | null>(null);
export const isAuthenticated = writable(false);
export const isLoading = writable(false);
```

## 📡 API Integration

### Service Layer Pattern

```typescript
// src/lib/services/base.ts
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:12000';

export async function apiRequest<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const token = localStorage.getItem('access_token');
  
  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...(token && { Authorization: `Bearer ${token}` }),
      ...options.headers
    }
  });
  
  if (!response.ok) {
    throw new Error(`API Error: ${response.status}`);
  }
  
  return response.json();
}
```

### API Services

**Products Service:**
```typescript
// src/lib/services/products.ts
export async function getProducts(params?: ProductQueryParams) {
  return apiRequest<ProductResponse>('/api/v1/products', {
    method: 'GET'
  });
}

export async function createProduct(data: CreateProductRequest) {
  return apiRequest<Product>('/api/v1/products', {
    method: 'POST',
    body: JSON.stringify(data)
  });
}
```

**Orders Service:**
```typescript
// src/lib/services/orders.ts
export async function getOrders() {
  return apiRequest<OrderResponse>('/api/v1/orders');
}

export async function updateOrderStatus(id: string, status: OrderStatus) {
  return apiRequest<Order>(`/api/v1/orders/${id}/status`, {
    method: 'PATCH',
    body: JSON.stringify({ status })
  });
}
```

## 🎯 Key Features Implementation

### 1. Dashboard Analytics

```svelte
<!-- src/routes/(app)/+page.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getOrderStats, getProductStats } from '$lib/services/analytics';
  
  let stats = $state({
    orders: { total: 0, pending: 0, completed: 0 },
    products: { total: 0, lowStock: 0 },
    revenue: { today: 0, thisMonth: 0 }
  });
  
  onMount(async () => {
    const [orderStats, productStats] = await Promise.all([
      getOrderStats(),
      getProductStats()
    ]);
    
    stats = { ...stats, orders: orderStats, products: productStats };
  });
</script>

<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
  <div class="bg-white rounded-xl p-6 shadow-sm">
    <h3 class="text-lg font-semibold">Total Orders</h3>
    <p class="text-3xl font-bold text-blue-600">{stats.orders.total}</p>
  </div>
  <!-- More stat cards... -->
</div>
```

### 2. Product Management

```svelte
<!-- src/routes/(app)/master/products/+page.svelte -->
<script lang="ts">
  import { getProducts, deleteProduct } from '$lib/services/products';
  import DataTable from '$lib/components/DataTable.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  
  let products = $state([]);
  let loading = $state(false);
  
  async function loadProducts() {
    loading = true;
    try {
      const response = await getProducts();
      products = response.data;
    } finally {
      loading = false;
    }
  }
  
  async function handleDelete(id: string) {
    if (confirm('Yakin ingin menghapus produk ini?')) {
      await deleteProduct(id);
      await loadProducts();
    }
  }
  
  onMount(loadProducts);
</script>

<div class="space-y-6">
  <div class="flex justify-between items-center">
    <h1 class="text-2xl font-bold">Products</h1>
    <Button href="/master/products/create">Add Product</Button>
  </div>
  
  <DataTable 
    data={products} 
    columns={[
      { key: 'name', label: 'Name' },
      { key: 'category', label: 'Category' },
      { key: 'price', label: 'Price', format: 'currency' },
      { key: 'stock', label: 'Stock' }
    ]}
    actions={[
      { label: 'Edit', href: (item) => `/master/products/${item.id}/edit` },
      { label: 'Delete', onClick: (item) => handleDelete(item.id), variant: 'danger' }
    ]}
    {loading}
  />
</div>
```

### 3. AI Chat Integration

```svelte
<!-- src/routes/(app)/ai-chat/+page.svelte -->
<script lang="ts">
  import { chatWithAI } from '$lib/services/ai';
  import ChatMessage from '$lib/components/ChatMessage.svelte';
  
  let messages = $state([]);
  let input = $state('');
  let loading = $state(false);
  
  async function sendMessage() {
    if (!input.trim() || loading) return;
    
    const userMessage = { role: 'user', content: input };
    messages = [...messages, userMessage];
    
    const currentInput = input;
    input = '';
    loading = true;
    
    try {
      const response = await chatWithAI(currentInput);
      const aiMessage = { role: 'assistant', content: response.message };
      messages = [...messages, aiMessage];
    } catch (error) {
      console.error('Chat error:', error);
      const errorMessage = { 
        role: 'assistant', 
        content: 'Maaf, terjadi kesalahan. Silakan coba lagi.' 
      };
      messages = [...messages, errorMessage];
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex-1 overflow-y-auto p-4 space-y-4">
    {#each messages as message}
      <ChatMessage {message} />
    {/each}
    
    {#if loading}
      <div class="flex items-center space-x-2">
        <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
        <span class="text-sm text-gray-500">AI sedang mengetik...</span>
      </div>
    {/if}
  </div>
  
  <form onsubmit={sendMessage} class="p-4 border-t">
    <div class="flex space-x-2">
      <input
        bind:value={input}
        placeholder="Ketik pesan Anda..."
        class="flex-1 px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        disabled={loading}
      />
      <Button type="submit" disabled={loading || !input.trim()}>
        Send
      </Button>
    </div>
  </form>
</div>
```

## 🚀 Production Deployment

### Build Optimization

```bash
# Production build dengan optimizations
pnpm run build

# Analyze bundle size
npx vite-bundle-analyzer build/

# Check build output
ls -la build/
```

### Environment Variables untuk Production

```env
# .env.production
API_BASE_URL=https://api.viralcast.ai
FRONTEND_URL=https://app.viralcast.ai
NODE_ENV=production

# Optional: Analytics & monitoring
VITE_ANALYTICS_ID=your_analytics_id
VITE_SENTRY_DSN=your_sentry_dsn
```

### Docker Deployment

```dockerfile
# Multi-stage build untuk optimasi ukuran
FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN npm install -g pnpm && pnpm install --frozen-lockfile
COPY . .
RUN pnpm run build

FROM node:20-alpine AS runner
WORKDIR /app
RUN npm install -g pnpm
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --prod --frozen-lockfile
COPY --from=builder /app/build ./build
EXPOSE 5544
CMD ["node", "build"]
```

**Build dan Run:**
```bash
# Build image
podman build -t viral-cast-ai-frontend:latest .

# Run container
podman run -d \
  --name vcai-frontend \
  -p 5544:5544 \
  --env-file .env.production \
  viral-cast-ai-frontend:latest
```

### Performance Optimization

**Vite Configuration:**
```typescript
// vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['svelte', '@sveltejs/kit'],
          ui: ['tailwindcss'],
          utils: ['date-fns', 'lodash-es']
        }
      }
    },
    chunkSizeWarningLimit: 1000
  },
  optimizeDeps: {
    include: ['date-fns', 'lodash-es']
  }
});
```

**Code Splitting:**
```typescript
// Lazy load components
const LazyComponent = lazy(() => import('$lib/components/HeavyComponent.svelte'));

// Dynamic imports untuk routes
const routes = {
  '/admin': () => import('./routes/admin/+page.svelte'),
  '/reports': () => import('./routes/reports/+page.svelte')
};
```

## 🐛 Troubleshooting

### Common Issues

**1. Development Server Issues**
```bash
# Clear node_modules dan reinstall
rm -rf node_modules pnpm-lock.yaml
pnpm install

# Clear SvelteKit cache
rm -rf .svelte-kit
pnpm run dev

# Check port conflicts
lsof -i :5174
```

**2. Build Errors**
```bash
# Type check errors
pnpm run check

# Clear build cache
rm -rf build .svelte-kit
pnpm run build

# Check for missing dependencies
pnpm audit
```

**3. API Connection Issues**
```bash
# Check backend status
curl http://localhost:12000/api/v1/healthchecker

# Verify environment variables
echo $API_BASE_URL

# Check CORS configuration di backend
```

**4. Styling Issues**
```bash
# Rebuild Tailwind
rm -rf .svelte-kit
pnpm run dev

# Check CSS imports
grep -r "@import" src/

# Verify theme variables
grep -r "--color" src/app.css
```

### Debug Mode

```bash
# Enable debug logging
DEBUG=vite:* pnpm run dev

# SvelteKit debug mode
VITE_LOG_LEVEL=info pnpm run dev

# Network debugging
VITE_DEBUG_API=true pnpm run dev
```

### Performance Debugging

```bash
# Bundle analyzer
npx vite-bundle-analyzer

# Lighthouse audit
npx lighthouse http://localhost:5174

# Memory usage
node --inspect-brk node_modules/.bin/vite dev
```

## 📚 Additional Resources

- **SvelteKit Documentation**: https://kit.svelte.dev/docs
- **Svelte Documentation**: https://svelte.dev/docs
- **Tailwind CSS**: https://tailwindcss.com/docs
- **Vite Documentation**: https://vitejs.dev/guide/
- **TypeScript Handbook**: https://www.typescriptlang.org/docs/
- **Playwright Testing**: https://playwright.dev/docs/intro
- **Vitest Testing**: https://vitest.dev/guide/

## 🤝 Contributing

### Development Guidelines

1. **Code Style**: Ikuti ESLint dan Prettier configuration
2. **Component Structure**: Gunakan composition pattern
3. **Type Safety**: Selalu gunakan TypeScript types
4. **Testing**: Tambahkan unit tests untuk utility functions
5. **Documentation**: Update dokumentasi untuk fitur baru

### Git Workflow

```bash
# Create feature branch
git checkout -b feature/new-component

# Commit dengan conventional format
git commit -m "feat: add new product card component"

# Push dan create PR
git push origin feature/new-component
```

### Code Review Checklist

- [ ] Code formatted dengan Prettier
- [ ] No ESLint warnings
- [ ] TypeScript types defined
- [ ] Components tested
- [ ] Responsive design implemented
- [ ] Accessibility considerations
- [ ] Performance impact assessed