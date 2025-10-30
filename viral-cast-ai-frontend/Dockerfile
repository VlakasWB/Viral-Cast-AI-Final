# Multi-stage build untuk aplikasi SvelteKit
FROM node:20-alpine AS builder

# Set working directory
WORKDIR /app

# Copy package files
COPY package.json pnpm-lock.yaml ./

# Install pnpm
RUN npm install -g pnpm

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy source code
COPY . .

# Set build-time environment variables
ENV API_BASE_URL=http://localhost:12000
ENV FRONTEND_URL=http://localhost:5544
ENV NODE_ENV=production

# Build aplikasi
RUN pnpm run build

# Production stage
FROM node:20-alpine AS runner

# Set working directory
WORKDIR /app

# Install pnpm
RUN npm install -g pnpm

# Copy package files
COPY package.json pnpm-lock.yaml ./

# Install production dependencies only
RUN pnpm install --prod --frozen-lockfile

# Copy built application dari builder stage
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./

# Create non-root user untuk security
RUN addgroup -g 1001 -S nodejs
RUN adduser -S svelte -u 1001

# Change ownership ke user svelte
RUN chown -R svelte:nodejs /app
USER svelte

# Expose port 5544
EXPOSE 5544

# Set environment variables
ENV NODE_ENV=production
ENV PORT=5544
ENV HOST=0.0.0.0

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD node -e "require('http').get('http://localhost:5544', (res) => { process.exit(res.statusCode === 200 ? 0 : 1) })"

# Start aplikasi
CMD ["node", "build"]