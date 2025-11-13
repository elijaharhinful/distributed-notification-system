FROM node:20-alpine AS builder

WORKDIR /app

COPY package*.json ./

# Install ALL dependencies for build
RUN npm ci --silent && \
    npm cache clean --force

COPY . .

# Build application
RUN npm run build

# Production stage
FROM node:20-alpine

WORKDIR /app

RUN addgroup -g 1001 -S nodejs && \
    adduser -S nestjs -u 1001

COPY package*.json ./

# Install only production dependencies
RUN npm ci --silent --only=production && \
    npm cache clean --force

# Copy built files from builder
COPY --from=builder --chown=nestjs:nodejs /app/dist ./dist

USER nestjs

EXPOSE 3002

HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
  CMD node -e "require('http').get('http://localhost:8084/health', (r) => process.exit(r.statusCode === 200 ? 0 : 1))"

CMD ["node", "dist/main.js"]